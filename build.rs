use std::env::var;
use std::io::Write;

fn main() {
    use_cfg_if_compiles(
        "can_vector",
        r#"
            use std::io::{self, Read, Write};

            struct Test;

            impl Read for Test {
                fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                    Ok(0)
                }

                fn is_read_vectored(&self) -> bool {
                    true
                }
            }

            impl Write for Test {
                fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                    Ok(0)
                }

                fn flush(&mut self) -> io::Result<()> {
                    Ok(())
                }

                fn is_write_vectored(&self) -> bool {
                    true
                }
            }
        "#,
    ); // https://github.com/rust-lang/rust/issues/69941
    use_cfg_if_compiles(
        "write_all_vectored",
        r#"
            use std::io::{self, IoSlice, Write};

            struct Test;

            impl Write for Test {
                fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                    Ok(0)
                }

                fn flush(&mut self) -> io::Result<()> {
                    Ok(())
                }

                fn write_all_vectored(&mut self, _: &mut [IoSlice<'_>]) -> io::Result<()> {
                    Ok(())
                }
            }
        "#,
    ); // https://github.com/rust-lang/rust/issues/70436

    use_feature("io_lifetimes_use_std");

    // Don't rerun this on changes other than build.rs, as we only depend on
    // the rustc version.
    println!("cargo:rerun-if-changed=build.rs");
}

fn use_cfg_if_compiles(cfg: &str, test: &str) {
    if can_compile(test) {
        use_feature(cfg);
    }
}

fn use_feature(feature: &str) {
    println!("cargo:rustc-cfg={}", feature);
}

/// Test whether the rustc at `var("RUSTC")` can compile the given code.
fn can_compile<T: AsRef<str>>(test: T) -> bool {
    use std::process::Stdio;

    let out_dir = var("OUT_DIR").unwrap();
    let rustc = var("RUSTC").unwrap();
    let target = var("TARGET").unwrap();

    // Use `RUSTC_WRAPPER` if it's set, unless it's set to an empty string, as
    // documented [here].
    // [here]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-reads
    let wrapper = var("RUSTC_WRAPPER")
        .ok()
        .and_then(|w| if w.is_empty() { None } else { Some(w) });

    let mut cmd = if let Some(wrapper) = wrapper {
        let mut cmd = std::process::Command::new(wrapper);
        // The wrapper's first argument is supposed to be the path to rustc.
        cmd.arg(rustc);
        cmd
    } else {
        std::process::Command::new(rustc)
    };

    cmd.arg("--crate-type=rlib") // Don't require `main`.
        .arg("--emit=metadata") // Do as little as possible but still parse.
        .arg("--target")
        .arg(target)
        .arg("--out-dir")
        .arg(out_dir); // Put the output somewhere inconsequential.

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    let mut child = cmd
        .arg("-") // Read from stdin.
        .stdin(Stdio::piped()) // Stdin is a pipe.
        .stderr(Stdio::null()) // Errors from feature detection aren't interesting and can be confusing.
        .spawn()
        .unwrap();

    writeln!(child.stdin.take().unwrap(), "{}", test.as_ref()).unwrap();

    child.wait().unwrap().success()
}
