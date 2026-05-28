fn main() {
    use_feature("io_lifetimes_use_std");

    // Don't rerun this on changes other than build.rs, as we only depend on
    // the rustc version.
    println!("cargo:rerun-if-changed=build.rs");
}

fn use_feature(feature: &str) {
    println!("cargo:rustc-cfg={}", feature);
}
