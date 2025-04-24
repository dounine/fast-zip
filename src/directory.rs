use crate::magic::Magic;
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
// #[repr(u16)]
// #[derive(Debug, Clone, Default, PartialEq, NumToEnum)]
// pub enum ExtraType {
//     #[default]
//     ZIP64 = 0x0001,
//     ExtendedTimestamp = 0x5455,
//     UnixExtraType = 0x7875,
//     AESEncryptionInfo = 0x9901,
//     NTFSTimestamp = 0x000A,
//     Other(u16),
// }
// enum_to_bytes!(ExtraType, u16);
// #[derive(Debug, Default)]
// pub struct Extra {
//     pub id: ExtraType,
//     pub size: u16,
//     pub data: Vec<u8>,
// }
#[derive(Debug)]
pub struct ExtendedTimestamp {
    pub modified_time: Option<u32>,
    pub access_time: Option<u32>,
    pub create_time: Option<u32>,
}
impl Size for ExtendedTimestamp {
    fn size(&self) -> usize {
        2 + 1
            + if self.modified_time.is_some() { 4 } else { 0 }
            + if self.access_time.is_some() { 4 } else { 0 }
            + if self.create_time.is_some() { 4 } else { 0 }
    }
}
#[derive(Debug, Default)]
pub struct UnixExtraType {
    length: u16,
    version: u8,
    uid_size: u8,
    uid: u32,
    gid_size: u8,
    gid: u32,
}

impl ValueRead for UnixExtraType {
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
        Ok(Self {
            length: stream.read_value()?,
            version: stream.read_value()?,
            uid_size: stream.read_value()?,
            uid: stream.read_value()?,
            gid_size: stream.read_value()?,
            gid: stream.read_value()?,
        })
    }
}
impl ValueWrite for UnixExtraType {
    fn write(&self, endian: &Endian) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(&0x00b_u16)?; //11位长度
        stream.write_value(&self.version)?;
        stream.write_value(&self.uid_size)?;
        stream.write_value(&self.uid)?;
        stream.write_value(&self.gid_size)?;
        stream.write_value(&self.gid)?;
        Ok(stream)
    }
}
impl Size for UnixExtraType {
    fn size(&self) -> usize {
        2 + 1 + 1 + 4 + 1 + 4
    }
}
#[derive(Debug, Default)]
pub struct NtfsTimestamp {
    tag: u16,
    size: u16,
    modified_time: u64,
    access_time: u64,
    create_time: u64,
}
impl Size for NtfsTimestamp {
    fn size(&self) -> usize {
        2 + 2 + 8 + 8 + 8
    }
}
impl ValueRead for NtfsTimestamp {
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
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
        Ok(Self {
            tag,
            size,
            modified_time: stream.read_value()?,
            access_time: stream.read_value()?,
            create_time: stream.read_value()?,
        })
    }
}
impl ValueWrite for NtfsTimestamp {
    fn write(&self, endian: &Endian) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(&0x0001_u16)?;
        stream.write_value(&24_u16)?;
        stream.write_value(&self.modified_time)?;
        stream.write_value(&self.access_time)?;
        stream.write_value(&self.create_time)?;
        Ok(stream)
    }
}
#[derive(Debug, Default)]
pub struct ZIP64 {
    size: u16,
    uncompressed_size: Option<u64>,
    compressed_size: Option<u64>,
    local_header_offset: Option<u64>,
    disk_number: Option<u32>,
}
impl Size for ZIP64 {
    fn size(&self) -> usize {
        self.uncompressed_size
            .and_then(|_| Some(8))
            .unwrap_or_default()
            + self
                .compressed_size
                .and_then(|_| Some(8))
                .unwrap_or_default()
            + self
                .local_header_offset
                .and_then(|_| Some(8))
                .unwrap_or_default()
            + self.disk_number.and_then(|_| Some(8)).unwrap_or_default()
    }
}
impl ValueWrite for ZIP64 {
    fn write(&self, endian: &Endian) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        let mut size: u16 = 0;
        if self.uncompressed_size.is_some() {
            size += 8;
        }
        if self.compressed_size.is_some() {
            size += 8;
        }
        if self.local_header_offset.is_some() {
            size += 8;
        }
        if self.disk_number.is_some() {
            size += 4;
        }
        stream.write_value(&size)?;
        if let Some(value) = &self.uncompressed_size {
            stream.write_value(value)?;
        }
        if let Some(value) = &self.compressed_size {
            stream.write_value(value)?;
        }
        if let Some(value) = &self.local_header_offset {
            stream.write_value(value)?;
        }
        if let Some(value) = &self.disk_number {
            stream.write_value(value)?;
        }
        Ok(stream)
    }
}
impl ValueRead for ZIP64 {
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
        let size: u16 = stream.read_value()?;
        let mut bytes: u16 = size;
        let uncompressed_size = if size >= 8 {
            if bytes <= 0 {
                None
            } else {
                bytes -= 8;
                Some(stream.read_value()?)
            }
        } else {
            None
        };
        let compressed_size = if size >= 8 {
            if bytes <= 0 {
                None
            } else {
                bytes -= 8;
                Some(stream.read_value()?)
            }
        } else {
            None
        };
        let local_header_offset = if size >= 8 {
            if bytes <= 0 {
                None
            } else {
                bytes -= 8;
                Some(stream.read_value()?)
            }
        } else {
            None
        };
        let disk_number = if size >= 4 {
            if bytes <= 0 {
                None
            } else {
                bytes -= 4;
                Some(stream.read_value()?)
            }
        } else {
            None
        };
        Ok(Self {
            size,
            uncompressed_size,
            compressed_size,
            local_header_offset,
            disk_number,
        })
    }
}
//https://libzip.org/specifications/extrafld.txt
#[derive(Debug)]
pub enum Extra {
    ZIP64(ZIP64),
    NTFSTimestamp(NtfsTimestamp),
    ExtendedTimestamp(ExtendedTimestamp),
    UnixExtraType(UnixExtraType),
}
impl ValueWrite for Extra {
    fn write(&self, endian: &Endian) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        match self {
            Extra::ZIP64(v) => {
                stream.write_value(&0x0001_u16)?;
                stream.write_value(v)?;
            }
            Extra::NTFSTimestamp(v) => {
                stream.write_value(&0x000A_u16)?;
                stream.write_value(v)?;
            }
            Extra::ExtendedTimestamp(v) => {
                stream.write_value(&0x5455_u16)?;
                stream.write_value(v)?;
            }
            Extra::UnixExtraType(v) => {
                stream.write_value(&0x7875_u16)?;
                stream.write_value(v)?;
            }
        }
        Ok(stream)
    }
}
impl ValueRead for Extra {
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
        let id: u16 = stream.read_value()?;
        Ok(match id {
            0x0001 => Self::ZIP64(stream.read_value()?),
            0x5455 => Self::ExtendedTimestamp(stream.read_value()?),
            0x7875 => Self::UnixExtraType(stream.read_value()?),
            0x000A => Self::NTFSTimestamp(stream.read_value()?),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Extra id {} not match", id),
                ));
            }
        })
    }
}
impl ValueRead for ExtendedTimestamp {
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
        let length: u16 = stream.read_value()?;
        let flags: u8 = stream.read_value()?;
        let mut bytes: u16 = 1;
        let modified_time = if flags & 0x01 != 0 {
            bytes += 4;
            Some(stream.read_value()?)
        } else {
            None
        };
        let access_time = if flags & 0x02 != 0 {
            if bytes >= length {
                None
            } else {
                Some(stream.read_value()?)
            }
        } else {
            None
        };
        let create_time = if flags & 0x04 != 0 {
            if bytes >= length {
                None
            } else {
                Some(stream.read_value()?)
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

        Ok(Self {
            modified_time,
            access_time,
            create_time,
        })
    }
}
impl ValueWrite for ExtendedTimestamp {
    fn write(&self, endian: &Endian) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        let mut length: u16 = 1;
        let mut flags: u8 = 0x00;
        if self.modified_time.is_some() {
            flags |= 0x01;
            length += 4;
        }
        if self.access_time.is_some() {
            flags |= 0x02;
            length += 4;
        }
        if self.create_time.is_some() {
            flags |= 0x04;
            length += 4;
        }
        stream.write_value(&length)?;
        stream.write_value(&flags)?;
        if let Some(mtime) = &self.modified_time {
            stream.write_value(mtime)?;
        }
        if let Some(atime) = &self.access_time {
            stream.write_value(atime)?;
        }
        if let Some(ctime) = &self.create_time {
            stream.write_value(ctime)?;
        }
        Ok(stream)
    }
}
impl Size for Extra {
    fn size(&self) -> usize {
        2 + match self {
            Extra::ZIP64(z) => z.size(),
            Extra::ExtendedTimestamp(v) => v.size(),
            Extra::UnixExtraType(v) => v.size(),
            Extra::NTFSTimestamp(v) => v.size(),
        }
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
    fn write(&self, endian: &Endian) -> std::io::Result<Stream> {
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
    pub fn size(&self) -> usize {
        let mut bytes = ZIP_FILE_HEADER_SIZE + self.file_name.as_bytes().len();
        for extra_field in &self.extra_fields {
            bytes += extra_field.size()
        }
        bytes
    }
}
impl Directory {
    pub fn set_data(&mut self, stream: Stream) {
        self.compression_method = CompressionType::Store;
        self.compressed_size = 0;
        self.uncompressed_size = stream.length() as u32;
        self.uncompressed = true;
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
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
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
    pub uncompressed: bool,
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
    pub fn size(&self) -> usize {
        let mut bytes =
            DIRECTORY_HEADER_SIZE + self.file_name.as_bytes().len() + self.file_comment.len();
        for extra_field in &self.extra_fields {
            bytes += extra_field.size()
        }
        bytes
    }
}
impl ValueWrite for Directory {
    fn write(&self, endian: &Endian) -> std::io::Result<Stream> {
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
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
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
            uncompressed: false,
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
                let extra_field: Extra = stream.read_value()?; //.read_exact_size(file.extra_field_length as u64)?;
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
