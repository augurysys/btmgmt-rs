use cmd::Command;
use error::Error;

use std::time;

pub const SET_SCAN_PARAMS_OPCODE: u16 = 0x002C;

pub struct SetScanParamsCommand {
    cmd_code: u16,
    ctrl_index: u16,
    param_length: u16,
    params: Vec<u8>,
    response: Vec<u8>,
    timeout: time::Duration,
}

impl SetScanParamsCommand {
    pub fn new(
        ctrl_index: u16,        
        interval: u16,
        window: u16,
        timeout: time::Duration,
    ) -> SetScanParamsCommand {
        let mut c = SetScanParamsCommand {
            cmd_code: SET_SCAN_PARAMS_OPCODE,
            ctrl_index,
            param_length: 4,
            params: Vec::new(),
            response: Vec::new(),
            timeout,
        };

        c.params.push((interval & 0xff) as u8);
        c.params.push((interval >> 8 & 0xff) as u8);
        c.params.push((window & 0xff) as u8);
        c.params.push((window >> 8 & 0xff) as u8);

        c
    }
}

impl SetScanParamsCommand {
    pub fn result(&self) -> Result<u8, Error> {
        if self.response.is_empty() {
            return Err(Error::NoResponse);
        }

        if let Some(err) = Error::from_status(self.response[8]) {
            return Err(err);
        }

        Ok(0)
    }
}

impl Command for SetScanParamsCommand {
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

        true
    }
}

