use anyhow::{ensure, Result};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipWriter;

fn main() -> Result<()> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let output = Command::new("cargo")
        .args(["package", "--list", "--allow-dirty"])
        .output()?;
    ensure!(output.status.success(), "unsuccessful");

    let mut writer = ZipWriter::new(File::create(out_dir.join("source.zip"))?);
    for path in String::from_utf8(output.stdout)?.lines() {
        if path == "Cargo.toml.orig" {
            continue;
        }

        writer.start_file(path, Default::default())?;
        io::copy(
            &mut File::open(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))?,
            &mut writer,
        )?;
    }

    writer.finish()?;
    Ok(())
}
