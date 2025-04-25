use crate::magic::Magic;
use std::fmt::{Debug, Formatter};
// use derive::NumToEnum;
use fast_stream::bytes::{Bytes, ValueRead, ValueWrite};
use fast_stream::deflate::Deflate;
use fast_stream::derive::NumToEnum;
use fast_stream::endian::Endian;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;
use fast_stream::{enum_to_bytes, vec};
use std::io::{Error, ErrorKind, Seek, SeekFrom, Write};
pub trait Size {
    fn size(&self) -> usize;
}
#[repr(u16)]
#[derive(Debug, Clone, Default, PartialEq, NumToEnum)]
pub enum CompressionType {
    #[default]
    Store = 0x0000,
    Shrink = 0x0001,
    Implode = 0x0006,
    Deflate = 0x0008,
    Deflate64 = 0x0009,
    BZIP2 = 0x000C,
    LZMA = 0x000E,
    XZ = 0x005F,
    JPEG = 0x0060,
    WavPack = 0x0061,
    PPMd = 0x0062,
    AES = 0x0063,
}
impl CompressionType {
    pub const fn byte_size() -> usize {
        2
    }
}
enum_to_bytes!(CompressionType, u16);
const ZIP_FILE_HEADER_SIZE: usize = Magic::byte_size()
    + size_of::<u16>() * 2
    + CompressionType::byte_size()
    + size_of::<u16>() * 2
    + size_of::<u32>() * 3
    + size_of::<u16>() * 2;
//https://libzip.org/specifications/extrafld.txt
#[derive(Debug)]
pub enum Extra {
    NTFS {
        mtime: u64,
        atime: u64,
        ctime: u64,
    },
    UnixExtendedTimestamp {
        mtime: Option<i32>,
        atime: Option<i32>,
        ctime: Option<i32>,
    },
    UnixAttrs {
        uid: u32,
        gid: u32,
    },
}
impl Extra {
    pub fn optional_field_size<T: Sized>(field: &Option<T>) -> u16 {
        match field {
            None => 0,
            Some(_) => size_of::<T>() as u16,
        }
    }
    pub fn field_size(&self, center: bool) -> u16 {
        match self {
            Extra::NTFS { .. } => 32,
            Extra::UnixExtendedTimestamp {
                mtime,
                atime,
                ctime,
            } => {
                1 + Self::optional_field_size(mtime) + {
                    if !center {
                        Self::optional_field_size(atime) + Self::optional_field_size(ctime)
                    } else {
                        0
                    }
                }
            }
            Extra::UnixAttrs { .. } => 11,
        }
    }
    pub fn header_id(&self) -> u16 {
        match self {
            Extra::NTFS { .. } => 0x000a,
            Extra::UnixExtendedTimestamp { .. } => 0x5455,
            Extra::UnixAttrs { .. } => 0x7875,
        }
    }
    pub fn if_present(val: Option<i32>, if_present: u8) -> u8 {
        match val {
            Some(_) => if_present,
            None => 0,
        }
    }
}
impl ValueWrite for Extra {
    fn write_args<T: Sized>(&self, endian: &Endian, args: Option<T>) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(&self.header_id())?;
        stream.write_value(&self.field_size(args.is_some()))?;
        match self {
            Extra::NTFS {
                mtime,
                atime,
                ctime,
            } => {
                stream.write_value(&0_u32)?;
                stream.write_value(&1_u16)?;
                stream.write_value(&24_u16)?;
                stream.write_value(mtime)?;
                stream.write_value(atime)?;
                stream.write_value(ctime)?;
            }
            Extra::UnixExtendedTimestamp {
                mtime,
                atime,
                ctime,
            } => {
                let flags = Self::if_present(*mtime, 1)
                    | Self::if_present(*atime, 1 << 1)
                    | Self::if_present(*ctime, 1 << 2);
                stream.write_value(&flags)?;
                if let Some(mtime) = mtime {
                    stream.write_value(mtime)?;
                }
                if !args.is_some() {
                    if let Some(atime) = atime {
                        stream.write_value(atime)?;
                    }
                    if let Some(ctime) = ctime {
                        stream.write_value(ctime)?;
                    }
                }
            }
            Extra::UnixAttrs { uid, gid, .. } => {
                stream.write_value(&1_u8)?;
                stream.write_value(&4_u8)?;
                stream.write_value(uid)?;
                stream.write_value(&4_u8)?;
                stream.write_value(gid)?;
            }
        }
        Ok(stream)
    }
}
impl ValueRead for Extra {
    fn read_args<T: Sized>(stream: &mut Stream, args: Option<T>) -> std::io::Result<Self> {
        let id: u16 = stream.read_value()?;
        Ok(match id {
            0x5455 => {
                let _length: u16 = stream.read_value()?;
                let flags: u8 = stream.read_value()?;
                let mut bytes: u16 = 1;
                let mtime = if flags & 0x01 != 0 {
                    bytes += 4;
                    Some(stream.read_value()?)
                } else {
                    None
                };
                let atime = if flags & 0x02 != 0 {
                    if !args.is_some() {
                        Some(stream.read_value()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let ctime = if flags & 0x04 != 0 {
                    if !args.is_some() {
                        Some(stream.read_value()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                if flags & 0xF8 != 0 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Flags is invalid in ExtendedTimestamp",
                    ));
                }
                Self::UnixExtendedTimestamp {
                    mtime,
                    atime,
                    ctime,
                }
            }
            0x7875 => {
                let _length: u16 = stream.read_value()?;
                let _version: u8 = stream.read_value()?;
                let _uid_size: u8 = stream.read_value()?;
                let uid: u32 = stream.read_value()?;
                let _gid_size: u8 = stream.read_value()?;
                Self::UnixAttrs {
                    uid,
                    gid: stream.read_value()?,
                }
            }
            0x000A => {
                let tag: u16 = stream.read_value()?;
                if tag != 0x0001 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Tag is invalid in NtfsTimestamp",
                    ));
                }
                let size: u16 = stream.read_value()?;
                if size != 24 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid NTFS Timestamps size",
                    ));
                }
                Self::NTFS {
                    mtime: stream.read_value()?,
                    atime: stream.read_value()?,
                    ctime: stream.read_value()?,
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Extra id {} not match", id),
                ));
            }
        })
    }
}

#[derive(Debug)]
pub struct ZipFile {
    pub min_version: u16,
    pub bit_flag: u16,
    pub compression_method: CompressionType,
    pub last_modification_time: u16,
    pub last_modification_date: u16,
    pub crc_32_uncompressed_data: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_name: String,
    pub extra_fields: Vec<Extra>,
    pub data_position: u64,
}
impl ValueWrite for ZipFile {
    fn write_args<T: Sized>(&self, endian: &Endian, args: Option<T>) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(&Magic::File)?;
        stream.write_value(&self.min_version)?;
        stream.write_value(&self.bit_flag)?;
        stream.write_value(&self.compression_method)?;
        stream.write_value(&self.last_modification_time)?;
        stream.write_value(&self.last_modification_date)?;
        stream.write_value(&self.crc_32_uncompressed_data)?;
        stream.write_value(&self.compressed_size)?;
        stream.write_value(&self.uncompressed_size)?;
        stream.write_value(&self.file_name_length)?;
        stream.write_value(&self.extra_field_length)?;
        stream.write_value(&self.file_name)?;
        for extra_field in &self.extra_fields {
            stream.write_value(extra_field)?;
        }
        Ok(stream)
    }
}
impl ZipFile {
    pub fn size(&self, center: bool) -> usize {
        let mut bytes = ZIP_FILE_HEADER_SIZE + self.file_name.as_bytes().len();
        for extra_field in &self.extra_fields {
            //Extra ID length + length + data
            bytes += 2 + 2 + extra_field.field_size(center) as usize
        }
        bytes
    }
}
impl Directory {
    pub fn set_data(&mut self, stream: Stream) {
        self.compression_method = CompressionType::Store;
        self.compressed_size = 0;
        self.uncompressed_size = stream.length() as u32;
        self.compressed = false;
        self.data = Some(stream)
    }
    pub fn decompressed(&mut self, stream: &mut Stream) -> std::io::Result<Vec<u8>> {
        let compressed_data = self.origin_data(stream)?;
        let data = if self.compression_method == CompressionType::Deflate {
            let mut data = Stream::new(compressed_data.into());
            data.decompress()?;
            data.take_data()?
        } else {
            compressed_data
        };
        Ok(data)
    }
    pub fn origin_data(&mut self, stream: &mut Stream) -> std::io::Result<Vec<u8>> {
        if let Some(data) = &mut self.data {
            data.copy_data()
        } else {
            stream.pin()?;
            stream.seek(SeekFrom::Start(self.file.data_position))?;
            let data = stream.read_exact_size(self.compressed_size as u64)?;
            stream.un_pin()?;
            Ok(data)
        }
    }
}

impl ValueRead for ZipFile {
    fn read_args<T: Sized>(stream: &mut Stream, args: Option<T>) -> std::io::Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::File {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory file magic number",
            ));
        }
        let mut file = ZipFile {
            min_version: stream.read_value()?,
            bit_flag: stream.read_value()?,
            compression_method: stream.read_value()?,
            last_modification_time: stream.read_value()?,
            last_modification_date: stream.read_value()?,
            crc_32_uncompressed_data: stream.read_value()?,
            compressed_size: stream.read_value()?,
            uncompressed_size: stream.read_value()?,
            file_name_length: stream.read_value()?,
            extra_field_length: stream.read_value()?,
            file_name: "".to_string(),
            extra_fields: vec![],
            data_position: 0,
        };
        let file_name = stream.read_exact_size(file.file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        file.file_name = file_name.clone();
        let mut total_bytes = 0;
        if file.extra_field_length > 0 {
            loop {
                let position = stream.position()?;
                let extra_field: Extra = stream.read_value()?; //.read_exact_size(file.extra_field_length as u64)?;
                file.extra_fields.push(extra_field);
                let size = stream.position()? - position;
                total_bytes += size;
                if total_bytes >= file.extra_field_length as u64 {
                    break;
                }
            }
        }
        file.data_position = stream.stream_position()?;
        // let data = file.un_compressed_data(stream)?;
        Ok(file)
    }
}
const DIRECTORY_HEADER_SIZE: usize = Magic::byte_size()
    + size_of::<u16>() * 6
    + size_of::<u32>() * 3
    + size_of::<u16>() * 5
    + size_of::<u32>() * 2;
#[derive(Debug)]
pub struct Directory {
    pub compressed: bool,
    pub data: Option<Stream>,
    pub version: u16,
    pub min_version: u16,
    pub bit_flag: u16,
    pub compression_method: CompressionType,
    pub last_modification_time: u16,
    pub last_modification_date: u16,
    pub crc_32_uncompressed_data: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_comment_length: u16,
    pub number_of_starts: u16,
    pub internal_file_attributes: u16,
    pub external_file_attributes: u32,
    pub offset_of_local_file_header: u32,
    pub file_name: String,
    pub extra_fields: Vec<Extra>,
    pub file_comment: Vec<u8>,
    pub file: ZipFile,
}
impl Directory {
    pub fn size(&self, center: bool) -> usize {
        let mut bytes =
            DIRECTORY_HEADER_SIZE + self.file_name.as_bytes().len() + self.file_comment.len();
        for extra_field in &self.extra_fields {
            bytes += 2 + 2 + extra_field.field_size(center) as usize
        }
        bytes
    }
}
impl ValueWrite for Directory {
    fn write_args<T: Sized>(&self, endian: &Endian, args: Option<T>) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(&Magic::Directory)?;
        stream.write_value(&self.version)?;
        stream.write_value(&self.min_version)?;
        stream.write_value(&self.bit_flag)?;
        stream.write_value(&self.compression_method)?;
        stream.write_value(&self.last_modification_time)?;
        stream.write_value(&self.last_modification_date)?;
        stream.write_value(&self.crc_32_uncompressed_data)?;
        stream.write_value(&self.compressed_size)?;
        stream.write_value(&self.uncompressed_size)?;
        stream.write_value(&self.file_name_length)?;
        stream.write_value(&self.extra_field_length)?;
        stream.write_value(&self.file_comment_length)?;
        stream.write_value(&self.number_of_starts)?;
        stream.write_value(&self.internal_file_attributes)?;
        stream.write_value(&self.external_file_attributes)?;
        stream.write_value(&self.offset_of_local_file_header)?;
        stream.write_value(&self.file_name)?;
        for extra_field in &self.extra_fields {
            stream.write_value(extra_field)?;
        }
        stream.write_value(&self.file_comment)?;
        Ok(stream)
    }
}
impl ValueRead for Directory {
    fn read_args<T: Sized>(stream: &mut Stream, args: Option<T>) -> std::io::Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::Directory {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory magic number",
            ));
        }
        let file = ZipFile {
            min_version: 0,
            bit_flag: 0,
            compression_method: CompressionType::Deflate,
            last_modification_time: 0,
            last_modification_date: 0,
            crc_32_uncompressed_data: 0,
            compressed_size: 0,
            uncompressed_size: 0,
            file_name_length: 0,
            extra_field_length: 0,
            file_name: "".to_string(),
            extra_fields: vec![],
            data_position: 0,
        };
        let mut info = Self {
            compressed: true,
            data: None,
            version: stream.read_value()?,
            min_version: stream.read_value()?,
            bit_flag: stream.read_value()?,
            compression_method: stream.read_value()?,
            last_modification_time: stream.read_value()?,
            last_modification_date: stream.read_value()?,
            crc_32_uncompressed_data: stream.read_value()?,
            compressed_size: stream.read_value()?,
            uncompressed_size: stream.read_value()?,
            file_name_length: stream.read_value()?,
            extra_field_length: stream.read_value()?,
            file_comment_length: stream.read_value()?,
            number_of_starts: stream.read_value()?,
            internal_file_attributes: stream.read_value()?,
            external_file_attributes: stream.read_value()?,
            offset_of_local_file_header: stream.read_value()?,
            file_name: "".to_string(),
            extra_fields: vec![],
            file_comment: vec![],
            file,
        };
        let file_name = stream.read_exact_size(info.file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        info.file_name = file_name;
        let mut total_bytes = 0;
        if info.extra_field_length > 0 {
            loop {
                let position = stream.position()?;
                let extra_field: Extra = stream.read_value_args(Some(true))?; //.read_exact_size(file.extra_field_length as u64)?;
                info.extra_fields.push(extra_field);
                let size = stream.position()? - position;
                total_bytes += size;
                if total_bytes >= info.extra_field_length as u64 {
                    break;
                }
            }
        }
        info.file_comment = stream.read_exact_size(info.file_comment_length as u64)?;
        stream.pin()?;
        stream.seek(SeekFrom::Start(info.offset_of_local_file_header as u64))?;
        let file: ZipFile = stream.read_value()?;
        stream.un_pin()?;
        info.file = file;
        Ok(info)
    }
}
