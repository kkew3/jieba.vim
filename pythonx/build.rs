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

fn main() {
    mkdir("src/data");
    download_word_list_from_cutword("src/data/unionwords.txt");
    pyo3_build_config::add_extension_module_link_args();
    if env::var("PROFILE").unwrap() == "release" {
        env::set_current_dir("jieba_vim").unwrap();
        let so = "jieba_navi_rs.so";
        let dylib_src = "../target/release/libjieba_navi_rs.dylib";
        match ensure_not_exists(so) {
            Ok(()) => symlink::symlink_file(dylib_src, so).unwrap(),
            Err(()) => panic!(
                "Failed to create symlink from `{}` to `{}`",
                dylib_src, so
            ),
        }
    }
}
