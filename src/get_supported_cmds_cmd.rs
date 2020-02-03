use cmd::Command;
use error::Error;
use std::time;

pub const GET_SUPPORTED_CMDS_CMD_OPCODE: u16 = 0x0002;

pub struct SupportedCmdsResult {
    cmds: Vec<u16>,
    events: Vec<u16>,
}

impl SupportedCmdsResult {
    pub fn new() -> SupportedCmdsResult {
        SupportedCmdsResult {
            cmds: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn add_cmd(&mut self, cmd: u16) {
        self.cmds.push(cmd);
    }

    pub fn add_event(&mut self, cmd: u16) {
        self.events.push(cmd);
    }

    pub fn is_cmd_supported(&self, cmd: u16) -> bool {
        self.cmds.contains(&cmd)
    }

    pub fn is_event_supported(&self, event: u16) -> bool {
        self.events.contains(&event)
    }
}

pub struct GetSupportedCmdsCommand {
    cmd_code: u16,
    ctrl_index: u16,
    param_length: u16,
    params: Vec<u8>,
    response: Vec<u8>,
    timeout: time::Duration,
}

impl GetSupportedCmdsCommand {
    pub fn new(timeout: time::Duration) -> GetSupportedCmdsCommand {
        GetSupportedCmdsCommand {
            cmd_code: GET_SUPPORTED_CMDS_CMD_OPCODE,
            ctrl_index: 0xFFFF,
            param_length: 0,
            params: Vec::new(),
            response: Vec::new(),
            timeout,
        }
    }
}

impl GetSupportedCmdsCommand {
    pub fn result(&self) -> Result<SupportedCmdsResult, Error> {
        if self.response.is_empty() {
            return Err(Error::NoResponse);
        }

        if let Some(err) = Error::from_status(self.response[8]) {
            return Err(err);
        }

        let parameters = &self.response[13..self.response.len()];
        let num_cmds: u16 = u16::from(self.response[9]) | (u16::from(self.response[10]) << 8);
        let num_events: u16 = u16::from(self.response[11]) | (u16::from(self.response[12]) << 8);

        let mut res = SupportedCmdsResult::new();
        for i in 0..num_cmds {
            let cmd: u16 = u16::from(parameters[i as usize * 2]) | (u16::from(parameters[i as usize * 2 + 1]) << 8);
            res.cmds.push(cmd)
        }
        for i in num_cmds..num_cmds+num_events {
            let event: u16 = u16::from(parameters[i as usize * 2]) | (u16::from(parameters[i as usize * 2 + 1]) << 8);
            res.events.push(event)
        }

        Ok(res)
    }
}

impl Command for GetSupportedCmdsCommand {
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
