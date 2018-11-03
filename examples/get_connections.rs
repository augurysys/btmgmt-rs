extern crate btmgmt;

fn main() {
    let btmgmt = btmgmt::BTMgmt::new().expect("error opening bt mgmt socket");
    println!("{:?}", btmgmt.get_connections().unwrap());
}
