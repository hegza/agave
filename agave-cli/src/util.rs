use fs_err as fs;
use std::{
    io::{Read, Write},
    path,
};

pub(crate) fn read_file(ifile: impl AsRef<path::Path>) -> String {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(ifile.as_ref())
        .unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    buf
}

pub(crate) fn write_file(ofile: impl AsRef<path::Path>, content: &[u8]) {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(ofile.as_ref())
        .unwrap();
    file.write_all(content).unwrap();
}
