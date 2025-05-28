use fast_stream::stream::Stream;
use fast_zip::zip::Zip;
use std::fs;

fn main() {
    let data = fs::read("./data/fsign.ipa").unwrap();
    let stream = Stream::new(data.into());
    let mut zip = Zip::new(stream).unwrap();
    // zip.parse().unwrap();

    // let data = Stream::new("abc".as_bytes().to_vec().into());
    // zip.add_file_and_compress(data, "hi/hello.txt").unwrap();
    let wechat_data = fs::read("./data/.troll-fools").unwrap();
    let wechat_stream = Stream::new(wechat_data.into());
    // zip.add_file(wechat_stream, "Payload/WeChat").unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output, &mut |total, size, format| {
        println!("write {}", format)
    })
    .unwrap();

    fs::write("./data/copy.zip", output.take_data().unwrap()).unwrap();
}
