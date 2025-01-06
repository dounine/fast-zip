use crate::stream::bytes::ValueRead;
use crate::stream::stream::Stream;
use derive::NumToEnum;
use std::io::{Read, Seek, Write};

#[repr(u32)]
#[derive(Debug, PartialEq, NumToEnum)]
pub enum Magic {
    EoCd = 0x06054b50,
    Directory = 0x02014b50,
    File = 0x04034b50,
}
impl<T: Read + Write + Seek> ValueRead<T> for Magic {
    fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
        let value: u32 = stream.read_value()?;
        Ok(value.into())
    }
}
