use std::{env, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or(std::env::VarError::NotPresent)?;
    let dest_path = Path::new(&out_dir).join("consts.rs");

    let mut out = String::new();
    write_nums(&mut out)?;
    std::fs::write(&dest_path, out)?;

    Ok(())
}

const MAX_SMALL: u64 = 512;

fn write_nums(mut out: impl std::fmt::Write) -> std::fmt::Result {
    for i in 2..=MAX_SMALL {
        writeln!(out, "bisect!(N{i}, {i}, N{h}, N{p});", h = i / 2, p = i % 2)?;
    }
    Ok(())
}
