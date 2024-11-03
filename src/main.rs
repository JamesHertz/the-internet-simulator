#![feature(iter_map_windows, iter_next_chunk)]
#![allow(unused)]

mod devices;
mod links;
mod protocols;
mod simulator;

use simulator::Simulator;
use std::env;

fn main() {
    init_log();
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
