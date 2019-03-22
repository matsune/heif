extern crate heif;

use std::env;

fn main() {
    let mut reader = heif::reader::HeifReader::default();
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        reader.parse(arg.as_str()).unwrap();
    }
}
