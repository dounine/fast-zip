use crate::stream::pin::Pin;
use crate::stream::stream::Stream;
use std::io;
use std::io::{Seek, SeekFrom};

#[allow(dead_code)]
pub trait Len {
    fn len(&mut self) -> io::Result<u64>;
}
impl<T: Seek> Len for Stream<T> {
    fn len(&mut self) -> io::Result<u64> {
        self.pin()?;
        let len = self.inner.seek(SeekFrom::End(0))?;
        self.un_pin()?;
        Ok(len)
    }
}
