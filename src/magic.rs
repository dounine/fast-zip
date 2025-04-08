use fast_stream::derive::NumToEnum;
use fast_stream::enum_to_bytes;

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, NumToEnum)]
pub enum Magic {
    EoCd = 0x06054b50,
    Directory = 0x02014b50,
    File = 0x04034b50,
    // Unknown(u32),
}
impl Magic {
    pub const fn byte_size() -> usize {
        4
    }
}
enum_to_bytes!(Magic, u32);
