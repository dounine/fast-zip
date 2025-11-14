use fast_stream::bytes::{Bytes, ValueRead, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
use std::io::{Error, ErrorKind, Result};

//https://libzip.org/specifications/extrafld.txt
#[derive(Debug, Clone)]
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
    pub fn size(&self) -> u16 {
        2 + 2 + self.field_size()
    }
    pub fn field_size(&self) -> u16 {
        match self {
            Extra::NTFS { .. } => 32,
            Extra::UnixExtendedTimestamp {
                atime,
                ctime,
                mtime,
                ..
            } => {
                1 + Self::optional_field_size(mtime)
                    + Self::optional_field_size(atime)
                    + Self::optional_field_size(ctime)
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
        let size = self.field_size();
        stream.write_value(size)?;
        match self {
            Extra::NTFS {
                mtime,
                atime,
                ctime,
                ..
            } => {
                stream.write_value(0_u32)?; //reserved
                stream.write_value(1_u16)?; //Tag1
                stream.write_value(24_u16)?; //Size1
                stream.write_value(mtime)?;
                stream.write_value(atime)?;
                stream.write_value(ctime)?;
            }
            Extra::UnixExtendedTimestamp {
                mtime,
                atime,
                ctime,
                ..
            } => {
                let flags: u8 = 3;
                // Self::if_present(mtime, 1) | Self::if_present(Some(1), 1 << 1) | Self::if_present(ctime, 1 << 2);
                stream.write_value(flags)?;
                if let Some(mtime) = mtime {
                    stream.write_value(mtime)?;
                }
                // if !r#type.value() {
                if let Some(atime) = atime {
                    stream.write_value(atime)?;
                }
                if let Some(ctime) = ctime {
                    stream.write_value(ctime)?;
                }
                // }
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
            0x5855 => {
                let mut _length: u16 = stream.read_value()?;
                let mtime = if _length > 0 {
                    _length -= 4;
                    Some(stream.read_value()?)
                } else {
                    None
                };
                let atime = if _length > 0 {
                    _length -= 4;
                    Some(stream.read_value()?)
                } else {
                    None
                };
                let ctime = if _length > 0 {
                    _length -= 4;
                    Some(stream.read_value()?)
                } else {
                    None
                };
                Self::UnixExtendedTimestamp {
                    mtime,
                    atime,
                    ctime,
                }
            }
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
                let mut _length: u16 = stream.read_value()?;
                let _reserved: u32 = stream.read_value()?;
                _length -= 4;
                let tag: u16 = stream.read_value()?;
                _length -= 2;
                if tag != 0x0001 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Tag is invalid in NtfsTimestamp",
                    ));
                }
                let size: u16 = stream.read_value()?;
                _length -= 2;
                if size != 24 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid NTFS Timestamps size",
                    ));
                }
                let mtime: u64 = if _length > 0 {
                    _length -= 8;
                    stream.read_value::<u64>()?
                } else {
                    0
                };
                let atime: u64 = if _length > 0 {
                    _length -= 8;
                    stream.read_value::<u64>()?
                } else {
                    0
                };
                let ctime: u64 = if _length > 0 {
                    _length -= 8;
                    stream.read_value::<u64>()?
                } else {
                    0
                };
                Self::NTFS {
                    mtime,
                    atime,
                    ctime,
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
