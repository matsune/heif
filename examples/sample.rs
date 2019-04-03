extern crate heif;

use heif::reader::HeifReader;
use std::env;

fn main() {
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        let mut reader = HeifReader::default();
        reader.load(arg.as_str()).unwrap();
        let grid_item_ids = reader
            .get_item_list_by_type("grid".parse().unwrap())
            .unwrap();
        if grid_item_ids.is_empty() {
            panic!("grid empty")
        }
        let grid = reader.grid_item_by_id(grid_item_ids[0]).unwrap();
        println!("grid {:?}", grid);
        let master_image_ids = reader.get_master_image_ids().unwrap();
        if master_image_ids.is_empty() {
            panic!("master images empty")
        }
        println!("master_image_ids {:?}", master_image_ids);
        let first_tile_id = master_image_ids[0];
        let data = reader.get_item_data_with_decoder_parameters(first_tile_id);
        println!("data {:?}", data)
    }
}
