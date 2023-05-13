extern crate bindgen;
use bindgen::callbacks::MacroParsingBehavior;
use bindgen::callbacks::ParseCallbacks;

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

// need to avoid macro parsing for a specific thing :(
// https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-1312298570
const IGNORE_MACROS: [&str; 20] = [
    "FE_DIVBYZERO",
    "FE_DOWNWARD",
    "FE_INEXACT",
    "FE_INVALID",
    "FE_OVERFLOW",
    "FE_TONEAREST",
    "FE_TOWARDZERO",
    "FE_UNDERFLOW",
    "FE_UPWARD",
    "FP_INFINITE",
    "FP_INT_DOWNWARD",
    "FP_INT_TONEAREST",
    "FP_INT_TONEARESTFROMZERO",
    "FP_INT_TOWARDZERO",
    "FP_INT_UPWARD",
    "FP_NAN",
    "FP_NORMAL",
    "FP_SUBNORMAL",
    "FP_ZERO",
    "IPPORT_RESERVED",
];

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        if self.0.contains(name) {
            MacroParsingBehavior::Ignore
        } else {
            MacroParsingBehavior::Default
        }
    }
}

impl IgnoreMacros {
    fn new() -> Self {
        Self(IGNORE_MACROS.into_iter().map(|s| s.to_owned()).collect())
    }
}

// largely taken from https://rust-lang.github.io/rust-bindgen/tutorial-3.html

fn main() {
    // link the lib
    println!("cargo:rustc-link-lib=ssh");

    // invalidate build whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .rustified_enum("ssh_.*")
        .parse_callbacks(Box::new(IgnoreMacros::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
