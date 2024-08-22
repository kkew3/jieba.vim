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

fn main() {
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
