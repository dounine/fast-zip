use fast_stream::deflate::CompressionLevel;
use fast_stream::stream::Stream;
use fast_zip::magic::Magic::File;
use fast_zip::zip::Zip;
use plist::Value;
use plist::Value::String;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};

fn main() {
    let mut zip = Zip::create(Stream::empty());
    let mut output = Stream::new(vec![].into());
    zip.write_clear = false;
    zip.with_crc32(true);
    let data = fs::read("./data/Info.plist").unwrap();
    zip.add_file(data.into(),"Payload/Grace.app/Info.plist").unwrap();
    zip.package(&mut output, &mut |total, size, format| {
        // println!("write {}", format)
    })
        .unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./data/hi4.ipa".to_string())
        .unwrap();
    file.write_all(&output.take_data().unwrap()).unwrap();
    
    // let data = fs::read("./data/Info.plist").unwrap();
    // let data: Value = plist::from_bytes(&data).unwrap();
    // data.to_file_binary("./data/Info2.plist").unwrap();
    // let data = fs::read("./data/Info2.plist").unwrap();
    // let data: Value = plist::from_bytes(&data).unwrap();
    // println!("{:?}", data);
    if true {
        return;
    }
    // new_data
    //     .add_file(Stream::new(b"hello world".to_vec().into()), "hi.txt")
    //     .unwrap();
    // new_data
    //     .add_file(Stream::new(b"hello world".to_vec().into()), "hi2.txt")
    //     .unwrap();
    // new_data
    //     .add_file(Stream::new(b"hello world".to_vec().into()), "hello/hi.txt")
    //     .unwrap();
    // let mut f =
    //     std::fs::File::open("/Users/lake/dounine/github/ipa/fast-zip/data/hi2.zip").unwrap();
    // let mut data = vec![];
    // f.read_to_end(&mut data).unwrap();
    // let mut new_data = Zip::create(Stream::new(data.into()));
    // new_data.parse().unwrap();
    // // new_data.with_compression_level(CompressionLevel::NoCompression);
    // // new_data.add_file(Stream::new(data.into()), "hi.txt").unwrap();
    // let mut output = Stream::empty();
    // new_data.package(&mut output, &mut |t, v, f| {}).unwrap();
    // fs::write(
    //     "/Users/lake/dounine/github/ipa/fast-zip/data/hi.zip",
    //     output.take_data().unwrap(),
    // )
    // .unwrap();

    let data = fs::read("./data/signed.ipa").unwrap();
    let stream = Stream::new(data.into());
    let mut zip = Zip::new(stream).unwrap();
    // for (_, dir) in &mut zip.directories {
    //     dir.decompressed().unwrap();
    // }
    // zip.parse().unwrap();

    // let data = Stream::new(b"abc".to_vec().into());
    // zip.add_file_and_compress(data, "hi/hello.txt").unwrap();
    // let wechat_data = fs::read("./data/ios.dylib").unwrap();
    // let wechat_stream = Stream::new(wechat_data.into());
    // zip.add_file(data, "Payload/fsign.app/hi.txt").unwrap();

    // let mut zip = zip.into_cache();
    // let zip_bytes = zip.into_bytes().unwrap();
    // let mut zip = Zip::from_cache(zip_bytes).unwrap().into_parser();

    let mut output = Stream::new(vec![].into());
    zip.write_clear = false;
    zip.directories
        .retain(|k, _| *k == "Payload/Grace.app/Info.plist");
    zip.package(&mut output, &mut |total, size, format| {
        println!("write {}", format)
    })
    .unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./data/hi4.ipa".to_string())
        .unwrap();
    file.write_all(&output.take_data().unwrap()).unwrap();
    // fs::write("./data/hi4.ipa", output.take_data().unwrap()).unwrap();
}
