use hex;

#[derive(Debug, Clone)]
pub enum AddressType {
    BrEdr,
    LePublic,
    LeRandom,
    Unknown,
}

impl AddressType {
    pub fn from_byte(value: u8) -> AddressType {
        match value {
            0 => AddressType::BrEdr,
            1 => AddressType::LePublic,
            2 => AddressType::LeRandom,
            _ => AddressType::Unknown,
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            AddressType::BrEdr => 0,
            AddressType::LePublic => 1,
            AddressType::LeRandom => 2,
            AddressType::Unknown => 0xff,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Address {
    pub address: [u8; 6],
    pub address_type: AddressType,
}

impl Address {
    pub fn from_bytes(address: [u8; 6], address_type: u8) -> Address {
        Address {
            address,
            address_type: AddressType::from_byte(address_type),
        }
    }

    pub fn from_string(address: &str, address_type: AddressType) -> Option<Address> {
        let parts = address.split(':').collect::<Vec<&str>>();
        if parts.len() != 6 {
            return None;
        }

        let mut bytes: [u8; 6] = Default::default();

        for i in 0..6 {
            if let Ok(byte) = hex::decode(parts[i]) {
                if byte.len() != 1 {
                    return None;
                }

                bytes[5 - i] = byte[0];
            } else {
                return None;
            }
        }

        Some(Address {
            address: bytes,
            address_type,
        })
    }

    pub fn to_string(&self) -> String {
        let mut a0: [u8; 1] = Default::default();
        let mut a1: [u8; 1] = Default::default();
        let mut a2: [u8; 1] = Default::default();
        let mut a3: [u8; 1] = Default::default();
        let mut a4: [u8; 1] = Default::default();
        let mut a5: [u8; 1] = Default::default();

        a0.copy_from_slice(&self.address[5..6]);
        a1.copy_from_slice(&self.address[4..5]);
        a2.copy_from_slice(&self.address[3..4]);
        a3.copy_from_slice(&self.address[2..3]);
        a4.copy_from_slice(&self.address[1..2]);
        a5.copy_from_slice(&self.address[0..1]);

        format!(
            "{}:{}:{}:{}:{}:{}",
            hex::encode_upper(a0),
            hex::encode_upper(a1),
            hex::encode_upper(a2),
            hex::encode_upper(a3),
            hex::encode_upper(a4),
            hex::encode_upper(a5)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_string() {
        let a = Address::from_string("AB:BC:CD:DE:EF:F1", AddressType::LeRandom).unwrap();
        assert_eq!("AB:BC:CD:DE:EF:F1", a.to_string());
    }
}
