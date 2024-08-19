//! A libfuzzer-like fuzzer using qemu for binary-only coverage
#![feature(min_specialization)]
mod client;

mod fuzzer;

mod harness;

mod instance;

mod options;

mod input;
use input::PacketData;

mod mutator;
use mutator::LainMutator;

mod metadata;
use metadata::{PacketLenFeedback, PacketLenMinimizerScheduler};

use crate::fuzzer::Fuzzer;
pub fn main() {
    Fuzzer::new().fuzz().unwrap();
}