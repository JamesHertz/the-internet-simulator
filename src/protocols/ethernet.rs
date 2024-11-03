use crate::links::{LinkData, LinkEnd, LinkError};
use std::{
    convert::{From, Infallible, TryFrom},
    fmt::Display,
    io::Write,
    num::ParseIntError,
};

use super::{ParseError, Parser};

pub const ETHERNET_CRC_SIZE: usize = 4;
pub const ETHERNET_MAC_ADDR_SIZE: usize = 6;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct MacAddress([u8; ETHERNET_MAC_ADDR_SIZE]);

#[derive(Debug)]
pub struct InvalidMacAddress;

impl From<ParseIntError> for InvalidMacAddress {
    fn from(value: ParseIntError) -> Self {
        InvalidMacAddress
    }
}

impl MacAddress {
    const MAC_ADDRESS_STR_LEN: usize = 3 * ETHERNET_MAC_ADDR_SIZE - 1;
    pub fn build(bytes: &[u8]) -> Result<Self, InvalidMacAddress> {
        if bytes.len() == ETHERNET_MAC_ADDR_SIZE {
            Ok(Self(bytes.try_into().unwrap()))
        } else {
            Err(InvalidMacAddress)
        }
    }

    pub fn new(bytes: [u8; ETHERNET_MAC_ADDR_SIZE]) -> Self {
        Self(bytes)
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

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
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

//impl From<MacAddress> for &[u8; ETHERNET_MAC_ADDR_SIZE] {
//    fn from(address: MacAddress) -> Self {
//        address.0
//    }
//}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EthernetFrame {
    pub source: MacAddress,
    pub destin: MacAddress,
    pub protocol: FrameProtocol,
    pub data: Box<[u8]>,
}

impl EthernetFrame {
    fn to_bytes(&self) -> Box<[u8]> {
        let mut buffer: Vec<u8> = Vec::new();
        super::write_bytes(&mut buffer, self.source.as_bytes());
        super::write_bytes(&mut buffer, self.destin.as_bytes());
        super::write_u16(&mut buffer, self.protocol as u16);
        super::write_bytes(&mut buffer, self.data.as_ref());
        super::write_bytes(&mut buffer, &[0; ETHERNET_CRC_SIZE]);
        buffer.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum FrameProtocol {
    Ipv4 = 0x0800,
    Apr = 0x0806,
}

impl TryFrom<u16> for FrameProtocol {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0800 => Self::Ipv4,
            0x0806 => Self::Apr,
            _ => return Err(()),
        })
    }
}

// ALTERNATIVE c:
impl TryFrom<&[u8]> for EthernetFrame {
    type Error = ParseError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_ethernet_frame(value)
    }
}

impl From<EthernetFrame> for Box<[u8]> {
    fn from(frame: EthernetFrame) -> Self {
        frame.to_bytes()
    }
}

// I purposedfully ignored the first 8 bytes (the preamble). 7 of which are patterns in the
// form 010101... to make the NIC aware that something is going to be sent over the wire
// (and not some random noise) and another one  which signals the start of the transmission
// (https://en.wikipedia.org/wiki/Ethernet_frame#Preamble_and_start_frame_delimiter).
pub fn parse_ethernet_frame(data: &[u8]) -> Result<EthernetFrame, ParseError> {
    let mut parser = Parser::build(data);

    let source = MacAddress::new(parser.parse_chunk()?);
    let destin = MacAddress::new(parser.parse_chunk()?);

    let protocol = parser.parse_u16()?;

    let mut data_and_crc = parser.collect();
    if data_and_crc.len() < ETHERNET_CRC_SIZE {
        Err(ParseError::MissingBytes)
    } else {
        // TODO: check the the CRC bytes are all set to 0
        data_and_crc.resize(data_and_crc.len() - ETHERNET_CRC_SIZE, 0);
        Ok(EthernetFrame {
            source,
            destin,
            protocol: protocol
                .try_into()
                .map_err(|_| ParseError::InvalidFieldValue {
                    field: "ethernet_protocol",
                    value: protocol as usize,
                })?,
            data: data_and_crc.into(),
        })
    }
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;

    use super::{EthernetFrame, FrameProtocol, MacAddress, ETHERNET_MAC_ADDR_SIZE};
    use crate::protocols::{self, ParseError};

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

    // tests for ethernet
    #[test]
    fn marshall_and_unmarshall() {
        let original = EthernetFrame {
            source: MacAddress::new([10; ETHERNET_MAC_ADDR_SIZE]),
            destin: MacAddress::new([255; ETHERNET_MAC_ADDR_SIZE]),
            protocol: FrameProtocol::Ipv4,
            data: Box::new([200; 512]),
        };

        let bytes = original.to_bytes();
        let frame = super::parse_ethernet_frame(bytes.as_ref());

        assert!(frame.is_ok());
        assert_eq!(original, frame.unwrap());
    }

    #[test]
    fn missing_crc() {
        let mut buffer = Vec::new();
        protocols::write_bytes(&mut buffer, &[10; ETHERNET_MAC_ADDR_SIZE]);
        protocols::write_bytes(&mut buffer, &[11; ETHERNET_MAC_ADDR_SIZE]);
        protocols::write_u16(&mut buffer, FrameProtocol::Apr as u16);
        protocols::write_bytes(&mut buffer, &[0; 2]);

        assert_eq!(
            Err(ParseError::MissingBytes),
            super::parse_ethernet_frame(buffer.as_ref())
        );
    }

    #[test]
    fn empty_frame() {
        let mut buffer = Vec::new();
        assert_eq!(
            Err(ParseError::MissingBytes),
            super::parse_ethernet_frame(buffer.as_ref())
        );
    }

    #[test]
    fn invalid_protocol() {
        let mut buffer = Vec::new();
        protocols::write_bytes(&mut buffer, &[10; ETHERNET_MAC_ADDR_SIZE]);
        protocols::write_bytes(&mut buffer, &[11; ETHERNET_MAC_ADDR_SIZE]);
        protocols::write_u16(&mut buffer, 0);
        protocols::write_bytes(&mut buffer, &[0; 10]);

        assert_eq!(
            Err(ParseError::InvalidFieldValue {
                field: "ethernet_protocol",
                value: 0
            }),
            super::parse_ethernet_frame(buffer.as_ref())
        );
    }
}
