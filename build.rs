use fs_err::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::write::{SimpleFileOptions, ZipWriter};

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let output = Command::new("cargo")
        .args(["package", "--list", "--allow-dirty"])
        .output()
        .unwrap();
    if !output.status.success() {
        panic!("cargo package failed");
    }

    let mut writer = ZipWriter::new(File::create(out_dir.join("source.zip")).unwrap());
    for path in String::from_utf8(output.stdout).unwrap().lines() {
        if matches!(path, "Cargo.toml.orig" | ".cargo_vcs_info.json") {
            continue;
        }

        writer
            .start_file(path, SimpleFileOptions::default())
            .unwrap();
        io::copy(
            &mut File::open(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap(),
            &mut writer,
        )
        .unwrap();
    }

    writer.finish().unwrap();
}
