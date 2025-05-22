use fast_stream::stream::Stream;
use fast_zip::zip::Zip;
use std::fs;

fn main() {
    let data = fs::read("./data/iphone.ipa").unwrap();
    let stream = Stream::new(data.into());
    let mut zip = Zip::new(stream).unwrap();
    // zip.parse().unwrap();

    for (_, dir) in &mut zip.directories {
        if dir.file_name == "Payload/" {
            //"Payload/FKCamera Full.app/embedded.mobileprovision" {
            let mp_content_bytes: &[u8] = include_bytes!("../../data/iphone2.mobileprovision");
            // let data = Stream::new(mp_content_bytes.to_vec().into());
            // let data = Stream::new("abc".as_bytes().to_vec().into());
            // dir.put_data(data);
            // dir.put_data_and_compress("你好吗".as_bytes().to_vec().into()).unwrap();
        }
    }

    // let data = Stream::new("abc".as_bytes().to_vec().into());
    // zip.add_file_and_compress(data, "hi/hello.txt").unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output, &mut |total, size, format| {
        println!("write {}", format)
    })
    .unwrap();

    fs::write("./data/copy.zip", output.take_data().unwrap()).unwrap();
}
