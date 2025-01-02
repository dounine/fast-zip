use std::fmt::Error;
use std::io;
use std::io::{ErrorKind, Seek, SeekFrom};
use crate::stream::stream::Stream;

#[allow(dead_code)]
pub trait Pin {
    fn pin(&mut self) -> io::Result<u64>;
    fn un_pin(&mut self) -> io::Result<u64>;
}
impl<T: Seek> Pin for Stream<T> {
    fn pin(&mut self) -> io::Result<u64> {
        let current = self.stream_position()?;
        self.pins.push(current);
        Ok(current)
    }

    fn un_pin(&mut self) -> io::Result<u64> {
        if let Some(pos) = self.pins.pop() {
            self.seek(SeekFrom::Start(pos))?;
            return Ok(pos);
        }
        Err(io::Error::new(ErrorKind::NotFound, Error::default()))
    }
}
