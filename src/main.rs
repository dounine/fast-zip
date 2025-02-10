// use crate::stream::stream::Stream;
use derive::NumToEnum;
// use std::io::{Read, Seek, SeekFrom};
use crate::error::ZipError;
use crate::zip::Zip;
use fast_stream::stream::Stream;
use std::io::Cursor;
use std::path::Prefix::Verbatim;
use std::{fs, io};

mod directory;
mod eocd;
mod error;
mod magic;
mod zip;

#[repr(u32)]
#[derive(Debug, NumToEnum)]
pub enum Cpu {
    X84 = 1,
    Arm = 2,
    Hello = 3 | 4,
    Unknown(u32),
}

fn main() -> Result<(), ZipError> {
    let zip_file = fs::File::open("./data/hello.zip")?;
    let stream = Stream::new(zip_file);
    let mut zip = Zip::new(stream);
    zip.parse()?;

    let mut data: Vec<u8> = vec![];
    let mut output = Cursor::new(&mut data);
    zip.write(&mut output)?;

    // println!("{:?}", data);
    fs::write("./data/copy.zip", &mut data)?;

    // let value: u8 = stream.read_value()?;
    // let value = stream.seek(SeekFrom::End(-22))?;
    // println!("position {}", value);
    // let mut ecof = [0u8; 22];
    // stream.read_exact(&mut ecof)?;
    // let position = stream.stream_position()?;
    // let mut cursor = Cursor::new(zip_file);
    // cursor.seek(SeekFrom::End(-22)).unwrap();
    // // zip_file.seek(SeekFrom::End(-22)).unwrap();
    // let mut ecof = [0u8; 22];
    // cursor.read_exact(&mut ecof).unwrap();
    // let magic = &ecof[..4];
    // if magic != b"PK\x05\x06" {
    //     println!("不是zip文件")
    // }
    // println!("{}", position);
    Ok(())
    // // let mut cursor = Cursor::new(&mut ipa_file);
    // println!("Hello, world!");
}
