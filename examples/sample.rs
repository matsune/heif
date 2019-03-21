extern crate heif;

use std::env;

fn main() {
    for arg in env::args().skip(1) {
        let path = format!("./examples/{}.heic", arg);
        println!("[{}]", path);
        heif::reader::HeifReader::from(path.as_str()).unwrap();
    }
}
