extern crate btmgmt;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: get_connections_info <address>");
    }

    let btmgmt = btmgmt::BTMgmt::new().expect("error opening bt mgmt socket");
    let address = btmgmt::address::Address::from_string(
        args[1].as_str(),
        btmgmt::address::AddressType::LeRandom,
    ).unwrap();
    let ci = btmgmt.get_connection_info(0, &address).unwrap();
    println!("{:?}", ci);
}
