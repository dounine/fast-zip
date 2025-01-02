use crate::stream::endian::Endian;
use crate::stream::stream::Stream;
use std::io;
use std::io::Read;

#[allow(dead_code)]
pub trait ValueRead<T: Read>: Sized {
    fn read(read: &mut T, endian: &Endian) -> io::Result<Self>;
}
#[allow(dead_code)]
impl<T: Read> Stream<T> {
    pub fn read_value<Value: ValueRead<T>>(&mut self) -> io::Result<Value> {
        Value::read(&mut self.inner, &self.endian)
    }
}
#[macro_export]
macro_rules! value_read {
    ($($typ:ty, $size:expr),*) => {
        $(
            impl<T: std::io::Read> ValueRead<T> for $typ {
                fn read(read: &mut T, endian: &crate::stream::endian::Endian) -> std::io::Result<Self> {
                    use crate::stream::endian::Endian;
                    let mut buf = [0u8; $size];
                    read.read_exact(&mut buf)?;
                    let value = match endian {
                        Endian::Big => <$typ>::from_be_bytes(buf),
                        Endian::Little => <$typ>::from_le_bytes(buf),
                    };
                    Ok(value)
                }
            }
        )*
    }
}
value_read!(u8, 1, u16, 2, u32, 4, u64, 8);
