use paired::bls12_381::Bls12;
use taupipp::fetch;
use taupipp::powers;

fn main() {
    /////////////////// ZCASH ///////////////////////////////
    println!("reading zcash taus!");
    let zcash_params = powers::TauParams::new(1 << 21);
    let zcash_uri = fetch::URI::File("zcash_poweroftau.md".to_string());
    let zcash_acc = fetch::read_powers_from::<Bls12>(zcash_params, zcash_uri)
        .expect("failed to read zcash params");

    /////////////////// Filecoin ///////////////////////////////
    // last contribution for Filecoin's power of tau:
    // https://github.com/arielgabizon/perpetualpowersoftau/tree/master/0018_GolemFactory_response
    println!("reading filecoin taus!");
    let fil_params = powers::TauParams::new(1 << 27);
    let filecoin_uri =
        fetch::URI::HTTP("https://trusted-setup.filecoin.io/phase1/challenge_19".to_string());
    let filecoin_acc = fetch::read_powers_from::<Bls12>(fil_params, filecoin_uri)
        .expect("failed to read filecoin powers");

    /////////////////// IPP  ///////////////////////////////
    println!("combining both powers into one IPP SRS");
    let ipp_srs = powers::create_ipp_srs(&zcash_acc, &filecoin_acc, 1 << 19);
    let srs_fname = "ipp_srs";
    println!("Writing the srs to {}", srs_fname);
    let mut file = std::fs::File::create(srs_fname).expect("create ipp_srs file failed");
    ipp_srs.write(&mut file).expect("failed to write the srs");
    println!("Done!");
}
