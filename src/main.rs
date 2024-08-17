//! A libfuzzer-like fuzzer using qemu for binary-only coverage

mod client;

mod fuzzer;

mod harness;

mod instance;

mod options;

use crate::fuzzer::Fuzzer;
pub fn main() {
    Fuzzer::new().fuzz().unwrap();
}