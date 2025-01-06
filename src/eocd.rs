use crate::magic::Magic;
use crate::stream::bytes::ValueRead;
use crate::stream::len::Len;
use crate::stream::pin::Pin;
use crate::stream::stream::Stream;
use std::io::{Read, Seek, SeekFrom, Write};

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
impl EoCd {
    pub fn find_offset<T: Read + Write + Seek>(stream: &mut Stream<T>) -> std::io::Result<u64> {
        let max_eocd_size: u64 = u16::MAX as u64 + 22;
        let mut search_size: u64 = 22; //最快的搜索
        let file_size = stream.len()?;

        if file_size < search_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not a zip file",
            ));
        }
        let eocd_magic: u32 = Magic::EoCd.into();
        loop {
            // 确保搜索范围不超过 EOCD 的最大大小
            search_size = search_size.min(max_eocd_size);
            stream.seek(SeekFrom::End(-(search_size as i64)))?;
            for i in 0..search_size - 3 {
                stream.pin()?;
                let magic: u32 = stream.read_value()?;
                stream.un_pin()?;
                stream.seek(SeekFrom::Current(1))?;
                if magic == eocd_magic {
                    return Ok(search_size - i);
                }
                if search_size == 22 {
                    break;
                }
            }
            if search_size >= max_eocd_size {
                break;
            }
            search_size = (search_size * 2).min(file_size);
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "not a zip file",
        ))
    }
}

impl<T: Read + Write + Seek> ValueRead<T> for EoCd {
    fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
        let eocd_offset = Self::find_offset(stream)?;
        stream.seek(SeekFrom::End(-(eocd_offset as i64)))?;
        stream.seek(SeekFrom::Current(4))?;

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
