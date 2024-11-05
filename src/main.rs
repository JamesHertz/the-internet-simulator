#![feature(iter_map_windows, iter_next_chunk)]
#![allow(unused)]

mod devices;
mod links;
mod protocols;
mod simulator;

use devices::{
    switch::{self, Layer2Switch},
    ProgrammableDevice,
};
use protocols::ethernet::{self, EthernetFrame, FrameProtocol, MacAddress};
use simulator::{InterfaceSpec, Simulator};
use std::env;

fn main() {
    init_log();

    let addresses: Vec<_> = [
        "11:11:11:11:11:11",
        "22:22:22:22:22:22",
        "33:33:33:33:33:33",
        "44:44:44:44:44:44",
    ]
    .iter()
    .map(|value| MacAddress::from(value).unwrap())
    .collect();

    let switch_addr = addresses[0];
    let source = addresses[1];
    let destin = addresses[2];

    let mut sim = Simulator::new();
    sim.add_device(Layer2Switch::new(switch_addr, 3));

    for i in 0..3 {
        let addr = addresses[i + 1];
        let device = ProgrammableDevice::new(addr, 1, move |addr, module| {
            log::debug!("Device {addr} running...");

            if addr == source {
                let frame = EthernetFrame {
                    source,
                    destin,
                    protocol: FrameProtocol::Ipv4, // just a joke
                    data: Box::from("Hello, world".as_bytes()),
                };

                let interface = module.get_interface(0).unwrap();
                interface.send(frame.to_bytes().as_ref())
            }

            let mut replied = false;

            loop {
                let msg = module.wait_for_msg();
                let frame = match EthernetFrame::from_raw_bytes(msg.data.as_ref()) {
                    Err(error) => {
                        log::error!("Device {addr}: Error parsing ethernet frame: {error:?}");
                        continue;
                    }
                    Ok(frame) => frame,
                };

                if frame.destin == addr {
                    log::info!(
                        "Device {addr}: Received a ethernet frame for me from {} with msg: '{}'",
                        frame.source,
                        String::from_utf8(frame.data.to_vec()).unwrap()
                    );

                    if !replied {
                        let frame = EthernetFrame {
                            source: addr,
                            destin: frame.source,
                            protocol: FrameProtocol::Ipv4, // just joking
                            data: format!("Hi {}", frame.source).into_bytes().into(),
                        };

                        let interface = module.get_interface(0).unwrap();
                        interface.send(frame.to_bytes().as_ref());

                        replied = true;
                    }
                } else {
                    log::debug!(
                        "Device {addr}: Received an ethernet frame from {} to {}. Dropping...",
                        frame.source,
                        frame.destin
                    )
                }
            }
        });
        sim.add_device(device);
        sim.add_link(
            InterfaceSpec::new(switch_addr, i as u32),
            InterfaceSpec::new(addr, 0),
        );
    }

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
