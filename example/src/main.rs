use fast_stream::stream::Stream;
use fast_zip::zip::Zip;
use std::fs;

fn main() {
    let zip_file = fs::File::open("./data/app.ipa").unwrap();
    let stream = Stream::new(zip_file.into());
    let mut zip = Zip::new(stream);
    zip.parse().unwrap();
    let mut data = Stream::new(vec![1, 2, 3].into());

    // zip.add_directory(&mut data, "Payload/hello.txt").unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output).unwrap();

    // println!("{:?}", data);
    fs::write("./data/output/copy.zip", output.take_data().unwrap()).unwrap();
}
