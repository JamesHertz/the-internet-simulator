#![allow(unused)]
use crate::links::{LinkData, LinkEnd, LinkError};
use std::fmt::Display;

#[derive(Clone, Copy)]
pub struct MacAddress([u8; 6]);

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = &self.0;
        write!(
            f,
            "{:#02x}:{:#02x}:{:#02x}:{:#02x}:{:#02x}:{:#02x}",
            address[0], address[1], address[2], address[3], address[4], address[5]
        )
    }
}

pub struct Interface {
    interface_id: u32,
    connection: Option<LinkEnd>,
}

struct Module {
    interfaces: Vec<Interface>,
    interface_nr: u32,
    // TODO: add more things here c:
}

impl Module {
    pub fn new(interface_nr: u32) -> Self {
        assert!(interface_nr > 0, "Provided 0 as inteface_nr for Module");

        Self {
            interface_nr,
            interfaces: (0..interface_nr)
                .map(|interface_id| Interface {
                    interface_id,
                    connection: None,
                })
                .collect(),
        }
    }

    pub fn get_interface_nr(&self) -> u32 {
        self.interface_nr
    }

    pub fn attach_link(&mut self, interface_id: u32, link_end: LinkEnd) {
        assert!(
            interface_id < self.interface_nr,
            "Invalid interface_id: {interface_id} for module with {} interfaces",
            self.interface_nr
        );

        let interface = &mut self.interfaces[interface_id as usize];
        match interface.connection {
            // TODO: attach a handler function to link_end
            None => interface.connection = Some(link_end),
            Some(_) => {
                // TODO: think if you want to panic or send a result error
                panic!("Attaching link to interface {interface_id} which already has a connection")
            }
        }
    }

    fn wait_for_msg(&mut self) -> Result<LinkData, LinkError> {
        todo!()
    }
}

trait Device {
    fn get_mac_address() -> MacAddress;
    fn run(self);
}
