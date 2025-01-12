use derive::NumToEnum;
use fast_stream::bytes::{ValueRead, ValueWrite};
use fast_stream::endian::Endian;
// use fast_stream::from_bytes;
use fast_stream::enum_to_bytes;
use fast_stream::stream::Stream;
use std::io::{Read, Seek, Write};

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, NumToEnum)]
pub enum Magic {
    EoCd = 0x06054b50,
    Directory = 0x02014b50,
    File = 0x04034b50,
}
enum_to_bytes!(Magic, u32);