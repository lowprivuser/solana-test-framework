use std::fs::{metadata, File};
use std::io::Read;

pub fn load_file_to_bytes(filename: &str) -> (Vec<u8>, usize) {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    return (buffer, metadata.len() as usize);
}
