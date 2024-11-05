use super::{Device, Module};
use crate::protocols::ethernet::{self, EthernetFrame, MacAddress};
use std::collections::HashMap;

pub struct Layer2Switch {
    address: MacAddress,
    module: Module,
    // TODO: limit the size of this and start using an array
    learn_table: HashMap<MacAddress, u32>,
}

impl Layer2Switch {
    pub fn new(address: MacAddress, interface_nr: u32) -> Self {
        Self {
            address,
            module: Module::new(interface_nr),
            learn_table: HashMap::new(),
        }
    }
}

impl Device for Layer2Switch {
    fn get_mac_address(&self) -> MacAddress {
        self.address
    }

    fn get_module(&mut self) -> &mut Module {
        &mut self.module
    }

    fn run(&mut self) {
        log::debug!("Layer2Switch {} running...", self.address);
        loop {
            let msg = self.module.wait_for_msg();
            // TODO: Do not parse the frame just check the first 6 bytes c:
            let frame = match EthernetFrame::from_raw_bytes(msg.data.as_ref()) {
                Ok(frame) => frame,
                Err(err) => {
                    log::error!(
                        "Layer2Switch {}: parsing ethernet frame from interface {}: {err:?}",
                        self.address,
                        msg.interface_id
                    );
                    continue;
                }
            };

            log::debug!("received ethernet frame {frame:?}");

            if frame.source != ethernet::ETHERNET_BROADCAST_MAC_ADDR {
                self.learn_table.insert(frame.source, msg.interface_id);
            }

            match self.learn_table.get(&frame.destin).copied() {
                Some(interface_id) if msg.interface_id != interface_id => {
                    log::debug!("Sending frame to interface {interface_id}");
                    self.module
                        .get_interface(interface_id)
                        .unwrap()
                        .send(msg.data.as_ref());
                }
                None => {
                    log::debug!("Broadcasting frame...");
                    self.module.interfaces().for_each(|interface| {
                        if interface.interface_id != msg.interface_id && interface.is_up() {
                            interface.send(&msg.data)
                        }
                    });
                }
                _ => {
                    log::warn!("Dropping frame: {frame:?}")
                }
            }
        }
    }
}
