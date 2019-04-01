extern crate heif;

use std::env;
use heif::reader::HeifReader;

fn main() {
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        println!("{:?}", HeifReader::default().load(arg.as_str()).unwrap());
    }
}
