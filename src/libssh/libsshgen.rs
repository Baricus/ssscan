// ensures we don't have tons of warning from C
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// add libssh!
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
