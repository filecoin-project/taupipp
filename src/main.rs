//use filecoin::accumulator::Accumulator as facc;
use std::fs::OpenOptions;
use std::io::{self, BufReader, BufWriter, Write};
use zcash::{Accumulator as zacc, CheckForCorrectness, UseCompression};

fn read_zcash_powers(fname: &str) {
    let mut reader = OpenOptions::new()
        .read(true)
        .open(fname)
        .expect(&format!("unable open {} in this directory", fname));
    let accumulator = zacc::deserialize(&mut reader, UseCompression::No, CheckForCorrectness::Yes)
        .expect("unable to read uncompressed accumulator");
}
fn main() {
    println!("Hello, world!");
    let zcash_file = "zcash_poweroftau.md";
    read_zcash_powers(zcash_file);
}
