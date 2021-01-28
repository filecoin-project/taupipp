use paired::bls12_381::Bls12;
use std::path::Path;
use taupipp::fetch;
use taupipp::powers;

// 1 << 19 because we need powers up to 2n **included**, but powers of tau
//  of length 2^21 for example only goes to powers g^a^{2^21 -1} so we can't
//  use the maximum power of two. In this case, 2^21 is the lowest one
//  (zcash) so we need to use the one below. The function takes care of
//  going to powers 2^20 included.
const MAX_PROOFS: usize = 1 << 19;
// required tau length
const TAU_LENGTH: usize = (MAX_PROOFS << 1) + 1;
/// Config holds information about the powers of tau and where to find the
/// default file and where to fetch it via http if not present
struct Config {
    pub powers: powers::TauParams,
    pub file: String,
    http: String,
}

impl Config {
    /// returns the config we wish to download, in this case zcash and filecoin one.
    fn get_defaults() -> (Config, Config) {
        (Config{
            powers: powers::TauParams::new(1 << 21,TAU_LENGTH),
            file: "zcash_powers".to_string(),
            // taken from https://github.com/ZcashFoundation/powersoftau-attestations/tree/master/0088
            http: "https://powersoftau-transcript.s3-us-west-2.amazonaws.com/88dc1dc6914e44568e8511eace177e6ecd9da9a9bd8f67e4c0c9f215b517db4d1d54a755d051978dbb85ef947918193c93cd4cf4c99c0dc5a767d4eeb10047a4".to_string(), 
        }, Config {
            powers: powers::TauParams::new(1 << 27,TAU_LENGTH),
            file: "filecoin_powers".to_string(),
            // IPFS gateway issue ?
            //http: "https://trusted-setup.filecoin.io/phase1/challenge_19".to_string(),
            http: "https://trusted-setup.s3.eu-central-1.amazonaws.com/challenge_18".to_string(),
        })
    }

    /// looks if the file is present, otherwise returns the download URL
    fn get_uri(&self) -> fetch::URI {
        if Path::new(&self.file).exists() {
            println!("Path {} found - using file to combine", &self.file);
            fetch::URI::File(self.file.clone())
        } else {
            println!(
                "Path {} not found - using http endpoint to download: {}",
                &self.file, &self.http
            );
            fetch::URI::HTTP(self.http.clone())
        }
    }
}

fn main() {
    let (zcash, filecoin) = Config::get_defaults();
    /////////////////// ZCASH ///////////////////////////////
    println!(
        "Reading zcash taus - look for default file '{}'.",
        &zcash.file
    );
    let zcash_acc = fetch::read_powers_from::<Bls12>(&zcash.powers, zcash.get_uri())
        .expect("failed to read zcash params");

    /////////////////// Filecoin ///////////////////////////////
    println!(
        "Reading filecoin taus - look for default file '{}'",
        &filecoin.file
    );
    let filecoin_acc = fetch::read_powers_from::<Bls12>(&filecoin.powers, filecoin.get_uri())
        .expect("failed to read filecoin powers");

    /////////////////// IPP  ///////////////////////////////
    println!("\nCombining both powers into one IPP SRS");
    let ipp_srs = powers::create_ipp_srs(&zcash_acc, &filecoin_acc);
    let srs_fname = "ipp_srs";
    println!("Writing the srs to {}", srs_fname);
    let mut file = std::fs::File::create(srs_fname).expect("create ipp_srs file failed");
    ipp_srs.write(&mut file).expect("failed to write the srs");

    println!("\n\nYou can find below the hashes of the powers used from both sides\nand the hash of the resulting SRS:\n");
    println!("\t- ZCASH HASH   : {:x?}", &zcash_acc.hash());
    println!("\t- FILECOIN HASH: {:x?}", &filecoin_acc.hash());
    println!("\t- IPP SRS HASH : {:x?}\n", &ipp_srs.hash());

    println!("Done!");
}
