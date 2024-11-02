use crate::{
    devices::{Device, MacAddress, Module},
    links::{self, Link, LinkEnd},
};
use std::{collections::HashMap, thread};

pub struct Simulator {
    devices: HashMap<MacAddress, Box<dyn Device + Send>>,
    links: Vec<(InterfaceSpec, InterfaceSpec)>,
}

pub struct InterfaceSpec {
    mac_address: MacAddress,
    interface_id: u32,
}

impl InterfaceSpec {
    pub fn new(mac_address: MacAddress, interface_id: u32) -> Self {
        Self {
            mac_address,
            interface_id,
        }
    }
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            links: Vec::new(),
        }
    }

    pub fn add_device<T>(&mut self, device: T)
    where
        T: Device + Send + 'static,
    {
        let mac = device.get_mac_address();
        assert!(
            self.devices.insert(mac, Box::new(device)).is_none(),
            "Mac address '{mac}' already assigned!"
        );
    }

    // TODO: add more things
    pub fn add_link(&mut self, source: InterfaceSpec, destin: InterfaceSpec) {
        self.links.push((source, destin))
    }

    fn create_network(&mut self) {
        for (src, dst) in self.links.iter() {
            let (end_1, end_2) = links::create_link();
            let device_1 = self
                .devices
                .get_mut(&src.mac_address)
                .expect("Failed to find device 1");

            device_1.get_module().attach_link(src.interface_id, end_1);

            let device_2 = self
                .devices
                .get_mut(&dst.mac_address)
                .expect("Failed to find device 2");

            device_2.get_module().attach_link(src.interface_id, end_2);
        }
    }

    pub fn run(mut self) {
        self.create_network();

        let mut handlers = Vec::new();

        for mut device in self.devices.into_values() {
            handlers.push(thread::spawn(move || device.run()));
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }
}
