// basic code to determine rustc version and generate a timestamp at build time

use std::{env, path};
use std::fs;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::process;
use std::ffi;

use chrono::{Local, DateTime};

pub fn dt_now_str() -> String {
    let dt: DateTime<Local> = Local::now();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn get_rustc_from_cmd<P: AsRef<ffi::OsStr>>(executable: P) -> String {
    // IMPROVEME
    // maybe change this to use one of the rustc-version* crates, at the cost of an additional dependency
    let output = process::Command::new(executable).arg("-V").output().expect("failed to spawn rustc");
    return String::from_utf8(output.stdout).unwrap();
}

fn main() {
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("buildinfo_gen.rs");
    let mut outfile = fs::File::create(&dst).unwrap();

    let rustc_shellcmd = env::var("RUSTC").unwrap();
    let rustc_version = get_rustc_from_cmd(&rustc_shellcmd);
    let rustc_cut = match rustc_version.find(" (") {
        Some(index) => &rustc_version[0..index],
        None => &rustc_version[0..rustc_version.len()-1]    // remove one byte of trailing \n
    };

    let mut out = String::new();

    out.push_str(
r#"//
// auto-generated, do not modify
//
"#);

    write!(out, "pub const BUILD_RUSTC:    &str = \"{}\";\n", rustc_cut).unwrap();
    write!(out, "pub const BUILD_DATETIME: &str = \"{}\";\n", dt_now_str()).unwrap();

    outfile.write(out.as_ref()).unwrap();
}
