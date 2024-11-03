#![feature(iter_map_windows, iter_next_chunk)]
#![allow(unused)]

use simulator::Simulator;

mod devices;
mod links;
mod protocols;
mod simulator;

fn main() {
    let sim = Simulator::new();
    sim.run();
}
