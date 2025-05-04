// clippy::deref_addrof has false positives for *&raw const expressions.
#![allow(unused, clippy::deref_addrof)]

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
