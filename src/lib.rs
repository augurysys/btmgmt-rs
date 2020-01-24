extern crate hex;
extern crate libc;

pub mod address;
mod cmd;
mod error;
mod get_connection_info_cmd;
mod get_connections_cmd;
mod add_device_cmd;

use cmd::Command;
use error::Error;
use get_connection_info_cmd::GetConnectionInfoCommand;
use get_connections_cmd::GetConnectionsCommand;
use add_device_cmd::AddDeviceCommand;
use std::time;
use std::sync::mpsc;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::mem;

const COMMAND_RESPONSE_EVENT: u8 = 0x01;
const COMMAND_STATUS_EVENT: u8 = 0x02;

pub const BTMGMT_EVENT_CODE_DEVICE_CONNECTED: u16       = 0x000b;
pub const BTMGMT_EVENT_CODE_DEVICE_DISCONNECTED: u16    = 0x000c;
pub const BTMGMT_EVENT_DEVICE_DISCONNECTED_REASON_UNSPECIFIED: u8          = 0x00;
pub const BTMGMT_EVENT_DEVICE_DISCONNECTED_REASON_CONNECTION_TIMEOUT: u8   = 0x01;
pub const BTMGMT_EVENT_DEVICE_DISCONNECTED_REASON_TERMINATE_LOCAL: u8      = 0x02;
pub const BTMGMT_EVENT_DEVICE_DISCONNECTED_REASON_TERMINATE_REMOTE: u8     = 0x03;
pub const BTMGMT_EVENT_DEVICE_DISCONNECTED_REASON_AUTH_FAILURE: u8         = 0x04;

const BTPROTO_HCI: i32 = 1;
const HCI_DEV_NONE: u16 = 0xffff;
const HCI_CHANNEL_CONTROL: u16 = 3;

#[repr(C)]
struct SockAddrHci {
    hci_family: libc::sa_family_t,
    hci_dev: u16,
    hci_channel: u16,
}

pub struct BtmgmtEventPacketStructure {
    pub event_code: u16,                    //byte loc 00-01
    pub controller_index: u16,              //byte loc 02-03
    pub param_lenght: u16,                  //byte loc 04-05
    pub device_address: address::Address,   //byte loc 06-11 + 12
    pub disconnect_reason: u8,              //byte loc 13
}

pub struct BTMgmt {
    pub fd: i32,
}

impl BTMgmt {
    pub fn new() -> Result<BTMgmt, Error> {
        let btmgmt = BTMgmt {
            fd: unsafe {
                libc::socket(
                    libc::PF_BLUETOOTH,
                    libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                    BTPROTO_HCI,
                )
            },
        };

        if btmgmt.fd < 0 {
            return Err(Error::SocketError);
        }

        let addr = SockAddrHci {
            hci_family: libc::AF_BLUETOOTH as u16,
            hci_dev: HCI_DEV_NONE,
            hci_channel: HCI_CHANNEL_CONTROL,
        };

        if unsafe {
            let addr_ptr = Box::into_raw(Box::new(addr));
            let ret_val = libc::bind(
                btmgmt.fd,
                addr_ptr as *const libc::sockaddr,
                std::mem::size_of::<SockAddrHci>() as u32,
            );
            Box::from_raw(addr_ptr);
            ret_val
        } < 0
        {
            return Err(Error::BindError);
        }

        Ok(btmgmt)
    }

    pub fn get_connections(&self, ctrl_index: u16) -> Result<Vec<address::Address>, Error> {
        let mut cmd = GetConnectionsCommand::new(ctrl_index, time::Duration::from_secs(1));
        self.write_command(&mut cmd)?;

        cmd.result()
    }

    pub fn get_connection_info(
        &self,
        ctrl_index: u16,
        address: &address::Address,
    ) -> Result<get_connection_info_cmd::ConnectionInfo, Error> {
        let mut cmd =
            GetConnectionInfoCommand::new(ctrl_index, &address, time::Duration::from_secs(1));
        self.write_command(&mut cmd)?;

        cmd.result()
    }

    pub fn add_device(
        &self,
        ctrl_index: u16,
        address: &address::Address,
    ) -> Result<address::Address, Error> {
        let mut cmd =
            AddDeviceCommand::new(ctrl_index, &address, time::Duration::from_secs(1));
        self.write_command(&mut cmd)?;

        cmd.result()
    }

    fn write_command(&self, cmd: &mut Command) -> Result<(), Error> {
        let mut fds = vec![libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN | libc::POLLHUP | libc::POLLERR,
            revents: 0,
        }];

        unsafe {
            let cmd_ptr = Box::into_raw(cmd.to_bytes().into_boxed_slice());
            libc::write(
                self.fd,
                cmd_ptr as *mut libc::c_void,
                cmd.size(),
            );
            Box::from_raw(cmd_ptr)
        };

        let start = time::SystemTime::now();
        loop {
            let r = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::c_ulong, 1) };
            if r > 0 && fds[0].revents > 0 {
                if fds[0].revents & libc::POLLIN > 0 {
                    let mut buffer: [u8; 1024] = [0; 1024];
                    let bytes = unsafe {
                        libc::read(self.fd, buffer.as_mut_ptr() as *mut libc::c_void, 1024)
                    };

                    if bytes <= 0 {
                        return Err(error::Error::UnknownError);
                    }

                    if (buffer[0] == COMMAND_RESPONSE_EVENT || buffer[0] == COMMAND_STATUS_EVENT)
                        && cmd.is_response(&buffer[0..buffer.len()])
                    {
                        let mut v = Vec::new();
                        v.extend_from_slice(&buffer);
                        cmd.store_response(v);
                        return Ok(());
                    }
                } else {
                    return Err(error::Error::UnknownError);
                }
            }

            // check command timeout
            match start.elapsed() {
                Ok(elapsed) => {
                    if elapsed > cmd.get_timeout() {
                        return Err(error::Error::Timeout);
                    }
                }

                Err(_) => {
                    return Err(error::Error::UnknownError);
                }
            }
        }
    }
}

impl Drop for BTMgmt {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}


pub struct BTMgmtEventListener {
    pub fd: i32,
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl BTMgmtEventListener {
    pub fn new(
        event_tx: mpsc::SyncSender<Box<BtmgmtEventPacketStructure>>,
    ) -> Result<BTMgmtEventListener, Error> {
        
        let mut btmgmteventlistener = BTMgmtEventListener {
            fd: unsafe {
                libc::socket(
                    libc::PF_BLUETOOTH,
                    libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                    BTPROTO_HCI,
                )
            },
            running: Arc::new(AtomicBool::new(false)),
            handle: None,  
        };

        if btmgmteventlistener.fd < 0 {
            return Err(Error::SocketError);
        }

        let addr = SockAddrHci {
            hci_family: libc::AF_BLUETOOTH as u16,
            hci_dev: HCI_DEV_NONE,
            hci_channel: HCI_CHANNEL_CONTROL,
        };

        if unsafe {
            let addr_ptr = Box::into_raw(Box::new(addr));
            let ret_val = libc::bind(
                btmgmteventlistener.fd,
                addr_ptr as *const libc::sockaddr,
                std::mem::size_of::<SockAddrHci>() as u32,
            );
            Box::from_raw(addr_ptr);
            ret_val
        } < 0
        {
            return Err(Error::BindError);
        }

        btmgmteventlistener.run(event_tx);
        Ok(btmgmteventlistener)
    }


    fn run(&mut self, event_tx: mpsc::SyncSender<Box<BtmgmtEventPacketStructure>>) {
        let mut fds = vec![libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN | libc::POLLHUP | libc::POLLERR,
            revents: 0,
        }];

        self.running.store(true, Ordering::Relaxed);
        let running = self.running.clone();
        let fd = self.fd.clone();

        let handle = std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let r = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::c_ulong, 1) };
                if r > 0 && fds[0].revents > 0 {
                    if fds[0].revents & libc::POLLIN > 0 {
                        let mut buffer: [u8; 128] = [0; 128];
                        let bytes = unsafe {
                            libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, 128)
                        };
                        
                        if bytes as usize == mem::size_of::<BtmgmtEventPacketStructure>() {
                            
                            let mut address: [u8; 6] = Default::default(); 
                            address.copy_from_slice(&buffer[6..12]);

                            let btmgmtstr = BtmgmtEventPacketStructure {
                                event_code:          u16::from(buffer[0]) | u16::from(buffer[1]) << 8,
                                controller_index:    u16::from(buffer[2]) | u16::from(buffer[3]) << 8,
                                param_lenght:        u16::from(buffer[4]) | u16::from(buffer[5]) << 8,
                                device_address:      address::Address::from_bytes(address, buffer[12]),
                                disconnect_reason:   buffer[13],
                            };

                            if btmgmtstr.event_code == BTMGMT_EVENT_CODE_DEVICE_DISCONNECTED && 
                                btmgmtstr.disconnect_reason == BTMGMT_EVENT_DEVICE_DISCONNECTED_REASON_AUTH_FAILURE {
                                    let event_data = Box::new(btmgmtstr);
                                    match event_tx.send(event_data) {
                                        Ok(()) => {}
                                        Err(_err) => return,                                    
                                    }                                    
                            }                                
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
    }
}


impl Drop for BTMgmtEventListener {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.handle.take().unwrap().join().unwrap();
        unsafe {
            libc::close(self.fd);
        }
    }
}
