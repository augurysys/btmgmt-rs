use address::{Address, AddressType};
use cmd::Command;
use error::Error;

use std::time;

const GET_CONNECTION_INFO_OPCODE: u16 = 0x0031;

pub struct GetConnectionInfoCommand {
    cmd_code: u16,
    ctrl_index: u16,
    param_length: u16,
    params: Vec<u8>,
    address: Address,
    response: Vec<u8>,
    timeout: time::Duration,
}

impl GetConnectionInfoCommand {
    pub fn new(ctrl_index: u16, address: &Address, to: time::Duration) -> GetConnectionInfoCommand {
        let mut c = GetConnectionInfoCommand {
            cmd_code: GET_CONNECTION_INFO_OPCODE,
            ctrl_index,
            param_length: 7,
            params: Vec::new(),
            address: address.clone(),
            response: Vec::new(),
            timeout: to,
        };

        c.params.extend_from_slice(&address.address);
        c.params.push(match address.address_type {
            AddressType::BrEdr => 0,
            AddressType::LePublic => 1,
            AddressType::LeRandom => 2,
            AddressType::Unknown => 0,
        });

        c
    }
}

impl GetConnectionInfoCommand {
    pub fn result(&self) -> Result<ConnectionInfo, Error> {
        if self.response.is_empty() {
            return Err(Error::NoResponse);
        }

        if let Some(err) = Error::from_status(self.response[8]) {
            return Err(err);
        }

        let parameters = &self.response[9..self.response.len()];

        let mut address: [u8; 6] = Default::default();
        address.copy_from_slice(&parameters[0..6]);
        let address_type = parameters[6];
        let rssi = {
            if parameters[7] & 0x80 == 0 {
                parameters[7] as i8
            } else {
                !((parameters[7] as i8) - 0x01) * -1
            }
        };

        let tx_power = parameters[8];
        let max_tx_power = parameters[9];

        Ok(ConnectionInfo {
            address: Address::from_bytes(address, address_type),
            rssi,
            tx_power,
            max_tx_power,
        })
    }
}

impl Command for GetConnectionInfoCommand {
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
        if self.cmd_code != u16::from(data[6]) | (u16::from(data[7]) << 8) {
            return false;
        }

        if let Some(_err) = Error::from_status(data[8]) {
            return true;
        }

        let address = &data[9..15];
        let address_type = data[15];

        if address != &self.address.address[0..self.address.address.len()] {
            return false;
        }

        if address_type != self.address.address_type.to_byte() {
            return false;
        }

        true
    }
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub address: Address,
    pub rssi: i8,
    pub tx_power: u8,
    pub max_tx_power: u8,
}
