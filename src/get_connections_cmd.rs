use address::{Address, AddressType};
use cmd::Command;
use error::Error;
use std::time;

const GET_CONNECTIONS_OPCODE: u16 = 0x0015;

pub struct GetConnectionsCommand {
    cmd_code: u16,
    ctrl_index: u16,
    param_length: u16,
    params: Vec<u8>,
    response: Vec<u8>,
    timeout: time::Duration,
}

impl GetConnectionsCommand {
    pub fn new(ctrl_index: u16, timeout: time::Duration) -> GetConnectionsCommand {
        GetConnectionsCommand {
            cmd_code: GET_CONNECTIONS_OPCODE,
            ctrl_index,
            param_length: 0,
            params: Vec::new(),
            response: Vec::new(),
            timeout,
        }
    }
}

impl GetConnectionsCommand {
    pub fn result(&self) -> Result<Vec<Address>, Error> {
        if self.response.is_empty() {
            return Err(Error::NoResponse);
        }

        if let Some(err) = Error::from_status(self.response[8]) {
            return Err(err);
        }

        let parameters = &self.response[11..self.response.len()];
        let count: u16 = u16::from(self.response[9]) | (u16::from(self.response[10]) << 8);

        let mut addresses = vec![];
        for i in 0..count {
            let mut address: [u8; 6] = Default::default();
            address.copy_from_slice(&parameters[i as usize * 7..i as usize * 7 + 6]);
            let at = parameters[i as usize * 7 + 6];

            let address = Address {
                address,
                address_type: AddressType::from_byte(at),
            };

            addresses.push(address);
        }

        Ok(addresses)
    }
}

impl Command for GetConnectionsCommand {
    fn get_cmd_code(&self) -> u16 {
        self.cmd_code
    }
    fn get_ctrl_index(&self) -> u16 {
        self.ctrl_index
    }
    fn get_param_length(&self) -> u16 {
        self.param_length
    }
    fn get_params(&self) -> Vec<u8> {
        self.params.clone()
    }
    fn get_timeout(&self) -> time::Duration {
        self.timeout
    }
    fn store_response(&mut self, data: Vec<u8>) {
        self.response = data;
    }
    fn is_response(&self, data: &[u8]) -> bool {
        self.cmd_code == u16::from(data[6]) | (u16::from(data[7]) << 8)
    }
}
