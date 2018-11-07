pub trait Command {
    fn get_cmd_code(&self) -> u16;
    fn get_ctrl_index(&self) -> u16;
    fn get_param_length(&self) -> u16;
    fn get_params(&self) -> Vec<u8>;
    fn is_response(&self, &[u8]) -> bool;
    fn store_response(&mut self, Vec<u8>);

    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![
            (self.get_cmd_code() & 0xff) as u8,
            ((self.get_cmd_code() >> 8) & 0xff) as u8,
            (self.get_ctrl_index() & 0xff) as u8,
            ((self.get_ctrl_index() >> 8) & 0xff) as u8,
            (self.get_param_length() & 0xff) as u8,
            ((self.get_param_length() >> 8) & 0xff) as u8,
        ];

        v.append(&mut self.get_params().clone());
        v
    }

    fn size(&self) -> usize {
        6 + self.get_params().len()
    }
}
