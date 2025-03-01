fn main() {
    pyo3_build_config::add_extension_module_link_args();
    config_expose_shared_library();
}

#[cfg(feature = "expose-shared-library")]
fn config_expose_shared_library() {
    use std::env;
    use std::path::PathBuf;

    if env::var("PROFILE").unwrap() == "release" {
        let pypackage: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "../pythonx/jieba_vim"]
                .iter()
                .collect();
        expose_shared_library::expose_shared_library(pypackage).unwrap();
    }
}

#[cfg(not(feature = "expose-shared-library"))]
fn config_expose_shared_library() {}
