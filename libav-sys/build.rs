use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    #[cfg(not(target_family = "windows"))]
    let libs = metadeps::probe().unwrap();

    #[cfg(target_family = "windows")]
    let win_lib = std::env::var("AV_LIB");
    #[cfg(target_family = "windows")]
    let win_include = std::env::var("AV_INCLUDE");
    #[cfg(target_family = "windows")]
    {
        if win_lib.is_err() || win_include.is_err() {
            panic!(
                "Set AV_LIB and AV_INCLUDE variable to directories with libav libs and headers. \
                 DO NOT include drive in path: don't do `C:\\path`, use `\\path`"
            );
        }
    }

    #[cfg(target_family = "windows")]
    println!(
        "cargo:rustc-link-search=native={}",
        &win_lib.as_ref().unwrap()
    );

    for e in ["avcodec", "avformat", "avutil", "swresample"].iter() {
        #[cfg(target_family = "windows")]
        println!("cargo:rustc-link-lib={}\\{}", &win_lib.as_ref().unwrap(), e);

        let headers: Vec<PathBuf>;
        cfg_if::cfg_if! {
            if #[cfg(target_family = "windows")] {
                headers = vec![PathBuf::from(&format!("{}",&win_include.as_ref().unwrap()))];
            } else {
                headers = libs.get(&format!("lib{}", e))
                    .unwrap()
                    .include_paths
                    .clone();
            }
        };

        let ignored_macros = IgnoreMacros(
            vec![
                "FP_INFINITE".into(),
                "FP_NAN".into(),
                "FP_NORMAL".into(),
                "FP_SUBNORMAL".into(),
                "FP_ZERO".into(),
                "IPPORT_RESERVED".into(),
            ]
            .into_iter()
            .collect(),
        );

        let mut builder = common_builder()
            .header(format!("data/lib{}.h", e))
            .parse_callbacks(Box::new(ignored_macros))
            .rustfmt_bindings(true);

        for header in headers.iter() {
            builder = builder.clang_arg("-I").clang_arg(header.to_str().unwrap());
        }

        // Manually fix the comment so rustdoc won't try to pick them
        format_write(builder, &format!("src/{}.rs", e));
    }
}

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn format_write(builder: bindgen::Builder, output: &str) {
    let s = builder
        .generate()
        .unwrap()
        .to_string()
        .replace("/**", "/*")
        .replace("/*!", "/*");

    let mut file = File::create(output).unwrap();

    let _ = file.write(s.as_bytes());
}

fn common_builder() -> bindgen::Builder {
    bindgen::builder()
        .raw_line("#![allow(deprecated)]")
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
}
