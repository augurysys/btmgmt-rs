extern crate hex;
extern crate libc;

pub mod address;
mod cmd;
mod error;
mod get_connection_info_cmd;
mod get_connections_cmd;

use cmd::Command;
use error::Error;
use get_connection_info_cmd::GetConnectionInfoCommand;
use get_connections_cmd::GetConnectionsCommand;

const COMMAND_RESPONSE_EVENT: u8 = 0x01;
const COMMAND_STATUS_EVENT: u8 = 0x02;

const BTPROTO_HCI: i32 = 1;
const HCI_DEV_NONE: u16 = 0xffff;
const HCI_CHANNEL_CONTROL: u16 = 3;

#[repr(C)]
struct SockAddrHci {
    hci_family: libc::sa_family_t,
    hci_dev: u16,
    hci_channel: u16,
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
            libc::bind(
                btmgmt.fd,
                Box::into_raw(Box::new(addr)) as *const libc::sockaddr,
                std::mem::size_of::<SockAddrHci>() as u32,
            )
        } < 0
        {
            return Err(Error::BindError);
        }

        Ok(btmgmt)
    }

    pub fn get_connections(&self, ctrl_index: u16) -> Result<Vec<address::Address>, Error> {
        let mut cmd = GetConnectionsCommand::new(ctrl_index);
        self.write_command(&mut cmd);

        cmd.result()
    }

    pub fn get_connection_info(
        &self,
        ctrl_index: u16,
        address: &address::Address,
    ) -> Result<get_connection_info_cmd::ConnectionInfo, Error> {
        let mut cmd = GetConnectionInfoCommand::new(ctrl_index, &address);
        self.write_command(&mut cmd);

        cmd.result()
    }

    fn write_command(&self, cmd: &mut Command) {
        let mut fds = vec![libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN | libc::POLLHUP | libc::POLLERR,
            revents: 0,
        }];

        unsafe {
            libc::write(
                self.fd,
                Box::into_raw(cmd.to_bytes().into_boxed_slice()) as *mut libc::c_void,
                cmd.size(),
            )
        };

        loop {
            let r = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::c_ulong, 1) };
            if r > 0 && fds[0].revents > 0 {
                if fds[0].revents & libc::POLLIN > 0 {
                    let mut buffer: [u8; 1024] = [0; 1024];
                    let bytes = unsafe {
                        libc::read(self.fd, buffer.as_mut_ptr() as *mut libc::c_void, 1024)
                    };

                    if bytes <= 0 {
                        return;
                    }

                    if (buffer[0] == COMMAND_RESPONSE_EVENT || buffer[0] == COMMAND_STATUS_EVENT)
                        && cmd.is_response(&buffer[0..buffer.len()])
                    {
                        let mut v = Vec::new();
                        v.extend_from_slice(&buffer);
                        cmd.store_response(v);
                        break;
                    }
                } else {
                    return;
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
