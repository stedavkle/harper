#![warn(clippy::pedantic)]

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const DICT_PATH: &str = "dictionary_de.dict";

fn main() {
    println!("cargo::rerun-if-changed={DICT_PATH}");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compressed-dictionary-de.zst");

    let in_file = File::open(DICT_PATH).expect("German dictionary file exists");
    let out_file = File::create(dest_path).expect("Can create output file");
    let reader = BufReader::new(in_file);
    let writer = BufWriter::new(out_file);

    // Use a lesser compression level to speed up debug builds.
    let compression_level = match env::var("OPT_LEVEL").unwrap().as_str() {
        "3" | "2" | "s" | "z" => zstd::zstd_safe::max_c_level(),
        _ => 4,
    };

    zstd::stream::copy_encode(reader, writer, compression_level)
        .expect("Able to write compressed German dictionary");
}
