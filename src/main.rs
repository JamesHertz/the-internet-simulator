#![feature(iter_map_windows, iter_next_chunk)]
#![allow(unused)]

mod devices;
mod links;
mod protocols;
mod simulator;

use devices::{switch::Layer2Switch, ProgrammableDevice};
use protocols::ethernet::MacAddress;
use simulator::Simulator;
use std::env;

fn main() {
    init_log();

    let addresses: Vec<MacAddress> = [
        "11:11:11:11:11:11",
        "22:22:22:22:22:22",
        "33:33:33:33:33:33",
    ]
    .iter()
    .map(|value| MacAddress::from(value).unwrap())
    .collect();

    let switch = Layer2Switch::new(addresses[0], 2);
    //let device_1 = ProgrammableDevice::new(addresses[1], 1, move |module, address| {
    //    let target = addresses[3];
    //});
    //
    let sim = Simulator::new();
    sim.run();
}

pub fn init_log() {
    // TODO: ADD TIME TO LOG LINES
    if env::var_os("RUST_LOG").is_none() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }
    pretty_env_logger::init();
}
