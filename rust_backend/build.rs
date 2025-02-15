use std::env;
use std::path::PathBuf;

fn main() {
    pyo3_build_config::add_extension_module_link_args();
    // Skip this step when building the final shared library on github. We
    // have to skip this step because `cross` won't be able to find jieba_vim.
    // Skipping this step is okay because in this case, exposing the shared
    // library to python is not necessary.
    if let Ok(value) = env::var("SKIP_EXPOSE_SHARED_LIBRARY_TO_PYTHON") {
        if value == "1" {
            return;
        }
    }
    if env::var("PROFILE").unwrap() == "release" {
        let pypackage: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "../pythonx/jieba_vim"]
                .iter()
                .collect();
        manual_build_pyo3::expose_shared_library(pypackage).unwrap();
    }
}
