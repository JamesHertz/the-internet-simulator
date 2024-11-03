use crate::{
    links::{LinkData, LinkEnd, LinkError},
    protocols::ethernet::MacAddress,
};

pub struct Interface {
    interface_id: u32,
    connection: Option<LinkEnd>,
}

pub struct Module {
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

pub trait Device {
    fn get_mac_address(&self) -> MacAddress;
    fn get_module(&mut self) -> &mut Module;
    fn run(&mut self);

    // default methods
    //fn attach_link(&mut self, interface_id: u32, link_end: LinkEnd) {
    //    self.get_module().attach_link(interface_id, link_end)
    //}
}
