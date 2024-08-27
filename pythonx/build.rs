use const_format::concatcp;
use curl::easy::Easy;
use std::io::Write;
use std::{env, fs, io};

fn ensure_not_exists(path: &str) -> Result<(), ()> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => Ok(()),
            _ => Err(()),
        },
    }
}

fn mkdir(path: &str) {
    match fs::create_dir(path) {
        Ok(()) => (),
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => panic!("Failed to mkdir \"{}\"", path),
        },
    }
}

fn download_word_list_from_cutword(path: &str) {
    let outfile = fs::File::create(path).unwrap();
    let mut outfile = io::BufWriter::new(outfile);
    let mut handle = Easy::new();
    let url = "https://raw.githubusercontent.com/liwenju0/cutword/main/cutword/unionwords.txt";
    handle.url(url).unwrap();
    let mut transfer = handle.transfer();
    transfer
        .write_function(|data| {
            outfile.write_all(data).unwrap();
            Ok(data.len())
        })
        .unwrap();
    transfer.perform().unwrap();
}

// See https://pyo3.rs/v0.22.2/building-and-distribution.html#manual-builds
// for detail about `SOURCE` and `DEST` in module `target_cfg`.

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(target_os = "windows")]
mod target_cfg {
    use const_format::concatcp;

    pub const SOURCE: &str = concatcp!("lib", super::PKG_NAME, ".dll");
    pub const DEST: &str = concatcp!(super::PKG_NAME, ".pyd");
}

#[cfg(target_os = "macos")]
mod target_cfg {
    use const_format::concatcp;

    pub const SOURCE: &str = concatcp!("lib", super::PKG_NAME, ".dylib");
    pub const DEST: &str = concatcp!(super::PKG_NAME, ".so");
}

#[cfg(target_os = "linux")]
mod target_cfg {
    use const_format::concatcp;

    pub const SOURCE: &str = concatcp!("lib", super::PKG_NAME, ".so");
    pub const DEST: &str = concatcp!(super::PKG_NAME, ".so");
}

fn main() {
    mkdir("src/data");
    download_word_list_from_cutword("src/data/unionwords.txt");
    pyo3_build_config::add_extension_module_link_args();
    if env::var("PROFILE").unwrap() == "release" {
        env::set_current_dir("jieba_vim").unwrap();
        let so = target_cfg::DEST;
        let dylib_src = concatcp!("../target/release/", target_cfg::SOURCE);
        match ensure_not_exists(so) {
            Ok(()) => symlink::symlink_file(dylib_src, so).unwrap(),
            Err(()) => panic!(
                "Failed to create symlink from `{}` to `{}`",
                dylib_src, so
            ),
        }
    }
}
