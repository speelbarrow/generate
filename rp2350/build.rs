use std::{
    env::var_os,
    fs::{read_to_string, write},
    path::PathBuf,
};

use glob::glob;

fn main() {
    let out = &PathBuf::from(var_os("OUT_DIR").unwrap());
    for entry in glob("*.x").unwrap() {
        if let Ok(path) = entry {
            let s = read_to_string(&path).unwrap();
            write(out.join(path.file_name().unwrap()), s).unwrap();
            println!("cargo::rerun-if-changed={}", path.display());
        }
    }
    println!("cargo::rerun-if-changed=build.rs");
}
