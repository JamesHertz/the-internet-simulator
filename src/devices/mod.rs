use crate::links::{LinkData, LinkEnd, LinkError};
use std::{convert::Infallible, fmt::Display, num::ParseIntError, string::ParseError};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct MacAddress([u8; 6]);

#[derive(Debug)]
pub struct InvalidMacAddress;

impl From<ParseIntError> for InvalidMacAddress {
    fn from(value: ParseIntError) -> Self {
        InvalidMacAddress
    }
}

impl MacAddress {
    const MAC_ADDRESS_BYTE_LEN: usize = 6;
    const MAC_ADDRESS_STR_LEN: usize = 3 * Self::MAC_ADDRESS_BYTE_LEN - 1;
    pub fn build(bytes: &[u8]) -> Result<Self, InvalidMacAddress> {
        if bytes.len() == Self::MAC_ADDRESS_BYTE_LEN {
            Ok(Self(bytes.try_into().unwrap()))
        } else {
            Err(InvalidMacAddress)
        }
    }

    pub fn from(data: &str) -> Result<Self, InvalidMacAddress> {
        if data.len() != Self::MAC_ADDRESS_STR_LEN {
            Err(InvalidMacAddress)
        } else {
            let bytes: Result<Vec<_>, InvalidMacAddress> = data
                .split(':')
                .map(|value| Ok(u8::from_str_radix(value, 16)?))
                .collect();
            Self::build(&bytes?)
        }
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = &self.0;
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            address[0], address[1], address[2], address[3], address[4], address[5]
        )
    }
}

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

#[cfg(test)]
mod test {
    use super::MacAddress;

    struct TestEntry {
        address: &'static [u8],
        expected_fmt: &'static str,
    }

    const TEST_ENTRIES: &[TestEntry] = &[
        TestEntry {
            address: &[0; 6],
            expected_fmt: "00:00:00:00:00:00",
        },
        TestEntry {
            address: &[255; 6],
            expected_fmt: "FF:FF:FF:FF:FF:FF",
        },
        TestEntry {
            address: &[16; 6],
            expected_fmt: "10:10:10:10:10:10",
        },
        TestEntry {
            address: &[255, 0, 32, 11, 0, 254],
            expected_fmt: "FF:00:20:0B:00:FE",
        },
    ];

    #[test]
    fn creation() {
        let invalids = [&[0][..], &[0; 10], &[1, 2, 3]];
        for invalid in invalids {
            assert!(MacAddress::build(invalid).is_err())
        }
    }

    #[test]
    fn mac_address_format() {
        for entry in TEST_ENTRIES {
            let address = MacAddress::build(entry.address);
            assert!(address.is_ok());

            let address = address.unwrap();
            assert_eq!(address.to_string(), entry.expected_fmt)
        }
    }

    #[test]
    fn from_string() {
        for entry in TEST_ENTRIES {
            let address = MacAddress::from(entry.expected_fmt);
            assert!(address.is_ok(), "Failed to parse '{}'", entry.expected_fmt);

            let address = address.unwrap();
            let expected = MacAddress::build(entry.address).unwrap();

            assert_eq!(expected, address);
        }
    }
}
