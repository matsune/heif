extern crate heif;

use std::env;

fn main() {
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        println!("{:?}", heif::reader::HeifReader::default().load(arg.as_str()).unwrap());
    }
}
