use std::env;
use std::path::PathBuf;

fn main() {
    pyo3_build_config::add_extension_module_link_args();
    if env::var("PROFILE").unwrap() == "release" {
        let pypackage: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "../pythonx/jieba_vim"]
                .iter()
                .collect();
        manual_build_pyo3::expose_shared_library(pypackage).unwrap();
    }
}
