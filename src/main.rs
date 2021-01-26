//use filecoin::accumulator::Accumulator as facc;
use std::fs::OpenOptions;
use std::io::{self, BufReader, Read};
use zcash::{Accumulator as zacc, CheckForCorrectness, UseCompression};

fn read_zcash_powers(fname: &str) {
    let mut reader = OpenOptions::new()
        .read(true)
        .open(fname)
        .expect(&format!("unable open {} in this directory", fname));
    {
        // We don't need to do anything with it, but it's important for
        // the hash chain.
        let mut tmp = [0; 64];
        reader
            .read(&mut tmp)
            .expect("unable to read BLAKE2b hash of previous contribution");
    }

    let accumulator = zacc::deserialize(&mut reader, UseCompression::Yes, CheckForCorrectness::Yes)
        .expect("unable to read uncompressed accumulator");
}
fn main() {
    println!("Hello, world!");
    let zcash_file = "zcash_poweroftau.md";
    read_zcash_powers(zcash_file);
}
