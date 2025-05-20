use crate::magic::Magic;
use fast_stream::bytes::{Bytes, StreamSized, ValueRead, ValueWrite};
use fast_stream::crc32::CRC32;
use fast_stream::deflate::{CompressionLevel, Deflate};
use fast_stream::derive::NumToEnum;
use fast_stream::endian::Endian;
use fast_stream::enum_to_bytes;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};

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
    pub fn size(&self, center: bool) -> u16 {
        2 + 2 + self.field_size(center)
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
    fn write_args<T: Sized>(self, endian: &Endian, args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(self.header_id())?;
        stream.write_value(self.field_size(args.is_some()))?;
        match self {
            Extra::NTFS {
                mtime,
                atime,
                ctime,
            } => {
                stream.write_value(0_u32)?;
                stream.write_value(1_u16)?;
                stream.write_value(24_u16)?;
                stream.write_value(mtime)?;
                stream.write_value(atime)?;
                stream.write_value(ctime)?;
            }
            Extra::UnixExtendedTimestamp {
                mtime,
                atime,
                ctime,
            } => {
                let flags = Self::if_present(mtime, 1)
                    | Self::if_present(Some(1), 1 << 1)
                    | Self::if_present(ctime, 1 << 2);
                stream.write_value(flags)?;
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
                stream.write_value(1_u8)?;
                stream.write_value(4_u8)?;
                stream.write_value(uid)?;
                stream.write_value(4_u8)?;
                stream.write_value(gid)?;
            }
        }
        Ok(stream)
    }
}
impl ValueRead for Extra {
    fn read_args<T: Sized>(stream: &mut Stream, _args: &Option<T>) -> Result<Self> {
        let id: u16 = stream.read_value()?;
        Ok(match id {
            0x5455 => {
                let mut length: u16 = stream.read_value()?;
                length -= 1;
                let flags: u8 = stream.read_value()?;
                let mtime = if flags & 0x01 != 0 {
                    length -= 4;
                    Some(stream.read_value()?)
                } else {
                    None
                };
                let atime = if flags & 0x02 != 0 {
                    if length == 0 {
                        None
                    } else {
                        length -= 4;
                        Some(stream.read_value()?)
                    }
                } else {
                    None
                };
                let ctime = if flags & 0x04 != 0 {
                    if length == 0 {
                        None
                    } else {
                        length -= 4;
                        Some(stream.read_value()?)
                    }
                } else {
                    None
                };
                if length > 0 {
                    stream.read_value::<u32>()?;
                }
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
pub struct DataDescriptor {
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}
impl DataDescriptor {
    pub fn size() -> usize {
        4 * 4
    }
}
impl ValueWrite for DataDescriptor {
    fn write_args<T: StreamSized>(self, endian: &Endian, args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        let magic = &[0x50, 0x4B, 0x07, 0x08];
        stream.write(magic)?;
        stream
            .write_value(self.crc32)?
            .write_value_args(self.compressed_size, args)?
            .write_value_args(self.uncompressed_size, args)?;
        Ok(stream)
    }
}
#[derive(Debug)]
pub struct ZipFile {
    pub min_version: u16,
    pub flags: u16,
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
    pub data_descriptor: Option<DataDescriptor>,
    pub data_position: u64,
}
impl ValueWrite for ZipFile {
    fn write_args<T: StreamSized>(self, endian: &Endian, args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value_args(Magic::File, args)?;
        stream.write_value_args(self.min_version, args)?;
        stream.write_value_args(self.flags, args)?;
        stream.write_value_args(self.compression_method, args)?;
        stream.write_value_args(self.last_modification_time, args)?;
        stream.write_value_args(self.last_modification_date, args)?;
        stream.write_value_args(self.crc_32_uncompressed_data, args)?;
        stream.write_value_args(self.compressed_size, args)?;
        stream.write_value_args(self.uncompressed_size, args)?;
        stream.write_value_args(self.file_name_length, args)?;
        stream.write_value_args(self.extra_field_length, args)?;
        stream.write_value_args(self.file_name, args)?;
        for extra_field in self.extra_fields {
            stream.write_value_args(extra_field, args)?;
        }
        if let Some(data_descriptor) = self.data_descriptor {
            stream.write_value_args(data_descriptor, args)?;
        }
        Ok(stream)
    }
}
impl ZipFile {
    pub fn size(&self, center: bool) -> usize {
        let mut bytes = ZIP_FILE_HEADER_SIZE + self.file_name.as_bytes().len();
        for extra_field in &self.extra_fields {
            bytes += extra_field.size(center) as usize
        }
        let data_descriptor_size = if self.data_descriptor.is_some() {
            DataDescriptor::size()
        } else {
            0
        };
        bytes + data_descriptor_size
    }
}
impl Directory {
    pub fn exec(&mut self, compression_level: &CompressionLevel) -> Result<()> {
        if let Some(data) = self.data.take() {
            if !self.compressed && self.compression_method == CompressionType::Deflate {
                self.crc_32_uncompressed_data = data.crc32_value()?;
                if let Some(file) = &mut self.file {
                    file.crc_32_uncompressed_data = self.crc_32_uncompressed_data;
                }
                let compress_size = data.compress(compression_level)?;
                self.compressed_size = compress_size as u32;
                self.compressed = true;
                if let Some(file) = &mut self.file {
                    file.compressed_size = self.compressed_size;
                }
            }
            data.seek_start()?;
            self.data = Some(data);
        }
        Ok(())
    }
    pub fn put_data(&mut self, stream: Stream) {
        self.compressed_size = stream.length() as u32;
        self.uncompressed_size = stream.length() as u32;
        if let Some(file) = &mut self.file {
            file.compressed_size = self.compressed_size;
            file.uncompressed_size = self.uncompressed_size;
        }
        self.compressed = false;
        self.data = Some(stream)
    }
    pub fn put_data_and_compress(
        &mut self,
        stream: Stream,
        compression_level: &CompressionLevel,
    ) -> Result<u64> {
        self.uncompressed_size = stream.length() as u32;
        let compress_size = stream.compress(compression_level)?;
        self.compressed_size = compress_size as u32;
        self.compressed = true;
        self.data = Some(stream);
        Ok(compress_size)
    }
    pub fn decompressed_callback(
        &mut self,
        stream: &mut Stream,
        callback_fun: &mut impl FnMut(usize),
    ) -> Result<Vec<u8>> {
        let position = if let Some(file) = &self.file {
            file.data_position
        } else {
            0
        };
        if let Some(data) = &mut self.data {
            data.seek_start()?;
            if self.compressed {
                data.decompress_callback(callback_fun)?;
            }
            return Ok(data.copy_data()?);
        }
        let compressed_data = self.origin_data(position, stream)?;
        let data = if self.compression_method == CompressionType::Deflate {
            let mut data = Stream::new(compressed_data.into());
            data.decompress_callback(callback_fun)?;
            data.take_data()?
        } else {
            compressed_data
        };
        Ok(data)
    }
    pub fn decompressed(&mut self, stream: &mut Stream) -> Result<Vec<u8>> {
        let position = if let Some(file) = &self.file {
            file.data_position
        } else {
            0
        };
        if let Some(data) = &mut self.data {
            data.seek_start()?;
            if self.compressed {
                data.decompress()?;
            }
            return Ok(data.copy_data()?);
        }
        let compressed_data = self.origin_data(position, stream)?;
        let data = if self.compression_method == CompressionType::Deflate {
            let mut data = Stream::new(compressed_data.into());
            data.decompress()?;
            data.take_data()?
        } else {
            compressed_data
        };
        Ok(data)
    }
    pub fn origin_data(&mut self, position: u64, stream: &mut Stream) -> Result<Vec<u8>> {
        if let Some(data) = &mut self.data {
            data.seek_start()?;
            data.copy_data()
        } else {
            stream.pin()?;
            stream.seek(SeekFrom::Start(position))?;
            let data = stream.read_exact_size(self.compressed_size as u64)?;
            stream.un_pin()?;
            Ok(data)
        }
    }
    pub fn take_data(&mut self, position: u64, stream: &mut Stream) -> Result<Vec<u8>> {
        if let Some(data) = &mut self.data.take() {
            data.seek_start()?;
            Ok(data.take_data()?)
        } else {
            stream.pin()?;
            stream.seek(SeekFrom::Start(position))?;
            let data = stream.read_exact_size(self.compressed_size as u64)?;
            stream.un_pin()?;
            Ok(data)
        }
    }
}

impl ValueRead for ZipFile {
    fn read_args<T: Sized>(stream: &mut Stream, _args: &Option<T>) -> Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::File {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory file magic number",
            ));
        }
        let mut file = ZipFile {
            min_version: stream.read_value()?,
            flags: stream.read_value()?,
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
            data_descriptor: None,
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
        // if file.bit_flag & 0x0008 != 0 && file.uncompressed_size == 0 {
        //     loop {
        //         let mut buffer = vec![0_u8; 4096];
        //         let bytes_read = stream.read(&mut buffer)?;
        //         if bytes_read == 0 {
        //             return Err(Error::new(
        //                 ErrorKind::InvalidData,
        //                 "Data Descriptor Invalid signature",
        //             ));
        //         }
        //         let magic = &[0x50, 0x4B, 0x07, 0x08];
        //         if let Some(position) = buffer.windows(4).position(|window| window == magic) {
        //             let remaing_bytes = bytes_read - position;
        //             if remaing_bytes < 16 {
        //                 //补充剩下字节
        //                 let mut remaing = vec![0_u8; 16 - remaing_bytes];
        //                 let bytes_read = stream.read(&mut remaing)?;
        //                 if bytes_read != 16 - remaing_bytes {
        //                     return Err(Error::new(
        //                         ErrorKind::InvalidData,
        //                         "Data Descriptor Invalid signature",
        //                     ));
        //                 }
        //                 buffer.extend_from_slice(&remaing);
        //             }
        //             let cursor = std::io::Cursor::new(buffer[4 + position..].to_vec());
        //             let mut data = Stream::new(Data::Mem(cursor));
        //             file.crc_32_uncompressed_data = data.read_value()?;
        //             file.compressed_size = data.read_value()?;
        //             file.uncompressed_size = data.read_value()?;
        //             break;
        //         }
        //     }
        // }
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
    pub flags: u16,
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
    pub file: Option<ZipFile>,
}
impl Directory {
    pub fn size(&self, center: bool) -> usize {
        let mut bytes =
            DIRECTORY_HEADER_SIZE + self.file_name.as_bytes().len() + self.file_comment.len();
        for extra_field in &self.extra_fields {
            bytes += extra_field.size(center) as usize
        }
        bytes
    }
}
impl ValueWrite for Directory {
    fn write_args<T: Sized>(self, endian: &Endian, _args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(Magic::Directory)?;
        stream.write_value(self.version)?;
        stream.write_value(self.min_version)?;
        stream.write_value(self.flags)?;
        stream.write_value(self.compression_method)?;
        stream.write_value(self.last_modification_time)?;
        stream.write_value(self.last_modification_date)?;
        stream.write_value(self.crc_32_uncompressed_data)?;
        stream.write_value(self.compressed_size)?;
        stream.write_value(self.uncompressed_size)?;
        stream.write_value(self.file_name_length)?;
        stream.write_value(self.extra_field_length)?;
        stream.write_value(self.file_comment_length)?;
        stream.write_value(self.number_of_starts)?;
        stream.write_value(self.internal_file_attributes)?;
        stream.write_value(self.external_file_attributes)?;
        stream.write_value(self.offset_of_local_file_header)?;
        stream.write_value(self.file_name)?;
        for extra_field in self.extra_fields {
            stream.write_value(extra_field)?;
        }
        stream.write_value(self.file_comment)?;
        Ok(stream)
    }
}
impl ValueRead for Directory {
    fn read_args<T: Sized>(stream: &mut Stream, _args: &Option<T>) -> Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::Directory {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory magic number",
            ));
        }
        let mut info = Self {
            compressed: true,
            data: None,
            version: stream.read_value()?,
            min_version: stream.read_value()?,
            flags: stream.read_value()?,
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
            file: None,
        };
        let file_name = stream.read_exact_size(info.file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        info.file_name = file_name;
        let mut total_bytes = 0;
        if info.extra_field_length > 0 {
            loop {
                let position = stream.position()?;
                let extra_field: Extra = stream.read_value_args(&Some(true))?; //.read_exact_size(file.extra_field_length as u64)?;
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
        let mut file: ZipFile = stream.read_value()?;
        if file.flags & 0x0008 != 0 {
            file.data_descriptor = Some(DataDescriptor {
                crc32: info.crc_32_uncompressed_data,
                compressed_size: info.compressed_size,
                uncompressed_size: info.uncompressed_size,
            })
        }
        // file.uncompressed_size = info.uncompressed_size;
        // file.compressed_size = info.compressed_size;
        // file.crc_32_uncompressed_data = info.crc_32_uncompressed_data;
        stream.un_pin()?;
        info.file = Some(file);
        Ok(info)
    }
}
