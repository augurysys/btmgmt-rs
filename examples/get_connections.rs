extern crate btmgmt;

fn main() {
    let btmgmt = btmgmt::BTMgmt::new().expect("error opening bt mgmt socket");
    let addresses = btmgmt.get_connections(0).unwrap();
    for a in addresses {
        println!("{}", a.to_string());
    }
}
