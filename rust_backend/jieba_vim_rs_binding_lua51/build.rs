#[cfg(target_os = "macos")]
fn add_extension_module_link_args() {
    println!("cargo::rustc-link-arg=-undefined");
    println!("cargo::rustc-link-arg=dynamic_lookup");
}

#[cfg(not(target_os = "macos"))]
fn add_extension_module_link_args() {}

fn main() {
    add_extension_module_link_args();
}
