pub mod ethernet;
use std::io::Write;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    MissingBytes,
    InvalidFieldValue { field: &'static str, value: usize },
}

type Result<T> = std::result::Result<T, ParseError>;

struct Parser<'a> {
    iter: std::slice::Iter<'a, u8>,
}

impl<'a> Parser<'a> {
    fn build(data: &'a [u8]) -> Self {
        Self { iter: data.iter() }
    }

    fn parse_chunk<const N: usize>(&mut self) -> Result<[u8; N]> {
        let bytes = self
            .iter
            .next_chunk::<N>()
            .map_err(|_| ParseError::MissingBytes)?;

        Ok(bytes.map(|v| *v))
    }

    fn next_u8(&mut self) -> Option<u8> {
        self.iter.next().copied()
    }

    fn parse_u16(&mut self) -> Result<u16> {
        let bytes = self.parse_chunk()?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn parse_u32(&mut self) -> Result<u32> {
        let bytes = self.parse_chunk()?;
        Ok(u32::from_be_bytes(bytes))
    }

    fn parse_u64(&mut self) -> Result<u64> {
        let bytes = self.parse_chunk()?;
        Ok(u64::from_be_bytes(bytes))
    }

    fn collect(self) -> Vec<u8> {
        self.iter.clone().copied().collect()
    }
}

#[inline]
fn write_u32(writer: &mut impl Write, value: u32) {
    writer.write_all(&value.to_be_bytes()).unwrap()
}

#[inline]
fn write_u16(writer: &mut impl Write, value: u16) {
    writer.write_all(&value.to_be_bytes()).unwrap()
}

#[inline]
fn write_bytes(writer: &mut impl Write, data: &[u8]) {
    writer.write_all(data).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    fn collect() {
        let mut parser = Parser::build(&[0, 1, 2]);

        assert_eq!(parser.next_u8(), Some(0));
        assert_eq!(&[1, 2], parser.collect().as_slice());
    }
}
