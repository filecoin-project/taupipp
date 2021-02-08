use bellman::groth16::aggregate::GenericSRS;
use paired::bls12_381::Bls12;
use taupipp::fetch;

fn main() {
    let fname = "ipp_srs";
    let http = "http://ec2-52-59-194-3.eu-central-1.compute.amazonaws.com:8000/ipp_srs";
    let uri = fetch::URI::try_from_file(&fname, &http);
    let mut reader = uri.get_reader();
    println!("fetching srs from {}", &uri);
    let srs = GenericSRS::<Bls12>::read(&mut reader).expect("unable to read correctly the srs");
    println!("{}", hex::encode(srs.hash()));
}
