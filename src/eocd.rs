use crate::stream::endian::Endian;
use crate::stream::bytes::ValueRead;
use crate::stream::stream::Stream;
use std::io::{Read, Seek, SeekFrom, Write};
use crate::magic::Magic;

#[derive(Debug)]
pub struct EoCd {
    pub number_of_disk: u16,
    pub directory_starts: u16,
    pub number_of_directory_disk: u16,
    pub entries: u16,
    pub size: u32,
    pub offset: u32,
    pub comment_length: u16,
}

impl<T: Read + Write + Seek> ValueRead<T> for EoCd {
    fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
        stream.seek(SeekFrom::End(-22))?;

        let magic: Magic = stream.read_value()?;
        if magic != Magic::EoCd {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not a zip file",
            ));
        }

        Ok(EoCd {
            number_of_disk: stream.read_value()?,
            directory_starts: stream.read_value()?,
            number_of_directory_disk: stream.read_value()?,
            entries: stream.read_value()?,
            size: stream.read_value()?,
            offset: stream.read_value()?,
            comment_length: stream.read_value()?,
        })
    }
}
