pub mod switch;
use crate::{
    links::{LinkData, LinkEnd, LinkError},
    protocols::ethernet::MacAddress,
};

use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

pub struct Interface {
    interface_id: u32,
    connection: Option<LinkEnd>,
}

pub struct Module {
    interfaces: Vec<Interface>,
    interface_nr: u32,
    msg_buffer: Arc<(Mutex<VecDeque<WireMsg>>, Condvar)>, // TODO: add more things here c:
}

pub struct WireMsg {
    pub data: LinkData,
    pub interface_id: u32,
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

impl Interface {
    fn is_up(&self) -> bool {
        // FIXME: start to consider the fact the other end might not have a handler
        self.connection.is_some()
    }

    fn send(&self, data: &[u8]) {
        // TODO:
        //  - start returing a Result<..>
        //  - start sending Arc<..> to avoid copying c:
        assert!(
            self.is_up(),
            "Sending data through interface {} that is down!",
            self.interface_id
        );
        let connection = self.connection.as_ref().unwrap();
        connection.send(data).unwrap();
    }
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
            msg_buffer: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
        }
    }

    pub fn get_interface_nr(&self) -> u32 {
        self.interface_nr
    }

    pub fn get_interface(&self, interface_id: u32) -> &Interface {
        assert!(self.interface_nr > interface_id);
        &self.interfaces[interface_id as usize]
    }

    pub fn attach_link(&mut self, interface_id: u32, link_end: LinkEnd) {
        assert!(
            interface_id < self.interface_nr,
            "Invalid interface_id: {interface_id} for module with {} interfaces",
            self.interface_nr
        );

        let interface = &mut self.interfaces[interface_id as usize];
        match interface.connection {
            None => {
                let msg_buffer = Arc::clone(&self.msg_buffer);
                link_end.attach_receiver(move |data| {
                    let (lock, condvar) = &*msg_buffer;
                    let mut buffer = lock.lock().unwrap();
                    buffer.push_back(WireMsg { interface_id, data });
                    condvar.notify_one() // there should only be one thread waiting for this
                });
                interface.connection = Some(link_end);
            }
            Some(_) => {
                // TODO: think if you want to panic or send a result error
                panic!("Attaching link to interface {interface_id} which already has a connection")
            }
        }
    }

    pub fn interfaces(&self) -> impl Iterator<Item = &Interface> {
        self.interfaces.iter()
    }

    pub fn wait_for_msg(&mut self) -> WireMsg {
        let (lock, condvar) = &*self.msg_buffer;

        let mut buffer = condvar
            .wait_while(lock.lock().unwrap(), |buffer| buffer.is_empty())
            .unwrap();

        buffer.pop_front().unwrap()
    }
}

pub struct ProgrammableDevice<F> {
    program: F,
    module: Module,
    address: MacAddress,
}

impl<F: FnMut(MacAddress, &mut Module)> ProgrammableDevice<F> {
    pub fn new(address: MacAddress, interface_nr: u32, program: F) -> Self {
        Self {
            program,
            address,
            module: Module::new(interface_nr),
        }
    }
}

impl<F: FnMut(MacAddress, &mut Module) + Send> Device for ProgrammableDevice<F> {
    fn get_mac_address(&self) -> MacAddress {
        self.address
    }

    fn get_module(&mut self) -> &mut Module {
        &mut self.module
    }

    fn run(&mut self) {
        (self.program)(self.address, &mut self.module);
    }
}
