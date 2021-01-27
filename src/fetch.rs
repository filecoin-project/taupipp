use crate::powers::{TauParams, TauPowers};
use groupy::EncodedPoint;
use isahc::prelude::*;
use paired::{bls12_381::Bls12, Engine, PairingCurveAffine};
use rayon::prelude::*;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufReader, Error, Read};

/// An URI can designate multiple ways to fetch a power of tau ceremony
#[derive(Clone, Debug)]
pub enum URI {
    /// A local filesystem path
    File(String),
    /// A HTTP endpoint which returns the transcript in the body's answer.
    HTTP(String),
}

/// This functions fetches the power of tau transcript using the corresponding
/// method of the URI and returns the tau powers.
/// Note it fetches and reads only the first tau^i in G1 and G2 groups of the
/// full transcript as only this part is needed for IPP.
pub fn read_powers_from<E: Engine>(
    params: TauParams,
    uri: URI,
) -> Result<TauPowers<E>, DeserializationError> {
    match uri {
        URI::File(a) => read_powers_from_file(params, &a),
        URI::HTTP(a) => read_powers_from_url(params, &a),
        _ => panic!("don't know how to fetch these params"),
    }
}

/// read_vec reads up to size * point_length bytes from the reader and only
/// verifies and returns "take" points - useful to advance the reader to a
/// certain point without verifying everything if one does not need the full CRS.
fn read_vec<R: Read, C: PairingCurveAffine>(
    reader: &mut R,
    size: usize, // read this number of points from the reader
    take: usize, // only verify and take this number of points
) -> Result<Vec<C>, DeserializationError> {
    assert!(take <= size);
    // Read the encoded elements
    let mut res = vec![C::Compressed::empty(); take];
    for encoded in &mut res {
        reader.read_exact(encoded.as_mut())?;
    }
    // read the rest that we wish to skip
    let mut empty = C::Compressed::empty();
    for i in 0..(size - take) {
        reader.read_exact(empty.as_mut());
    }
    let res_affine = res
        .into_par_iter()
        .map(|source| {
            source
                .into_affine()
                .map_err(|e| e.into())
                .and_then(|source| {
                    if source.is_zero() {
                        Err(DeserializationError::PointAtInfinity)
                    } else {
                        Ok(source)
                    }
                })
        })
        .collect::<Result<Vec<_>, DeserializationError>>()?;
    Ok(res_affine)
}

/// read some bytes representing the hash - we are not verifying the hash chain
/// here so we don't need those bytes.
fn skip_hash<R: Read>(r: &mut R) {
    let mut tmp = [0; 64];
    r.read(&mut tmp)
        .expect("unable to read BLAKE2b hash of previous contribution");
}

/// read the powers from the given file using the given parameters.
fn read_powers_from_file<E: Engine>(
    params: TauParams,
    fname: &str,
) -> Result<TauPowers<E>, DeserializationError> {
    let mut reader = OpenOptions::new()
        .read(true)
        .open(fname)
        .expect(&format!("unable open {} in this directory", fname));
    read_powers(params, &mut reader)
}

/// read the powers from the given URL using the given parameters. Currently, it
/// fetches first all then process the points. TODO fetch and process in
/// parallel.
fn read_powers_from_url<E: Engine>(
    params: TauParams,
    url: &str,
) -> Result<TauPowers<E>, DeserializationError> {
    let resp = isahc::get(url)?;
    let mut body = resp.into_body();
    read_powers(params, &mut body)
}

/// read_powers reads only the first tau^i in G1 and G2 groups of the full
/// transcript as only this part is needed for IPP.
fn read_powers<E: Engine, R: Read>(
    params: TauParams,
    reader: &mut R,
) -> Result<TauPowers<E>, DeserializationError> {
    skip_hash(reader);
    let g1p = read_vec::<_, E::G1Affine>(reader, params.g1_length, 20)?;
    let g2p = read_vec::<_, E::G2Affine>(reader, params.g2_length, 20)?;
    Ok(TauPowers {
        tau_g1: g1p,
        tau_g2: g2p,
    })
}

/// Errors that might occur during deserialization.
#[derive(Debug)]
pub enum DeserializationError {
    IoError(io::Error),
    DecodingError(groupy::GroupDecodingError),
    PointAtInfinity,
    Fetch(isahc::Error),
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializationError::IoError(ref e) => write!(f, "Disk IO error: {}", e),
            DeserializationError::DecodingError(ref e) => write!(f, "Decoding error: {}", e),
            DeserializationError::PointAtInfinity => write!(f, "Point at infinity found"),
            DeserializationError::Fetch(ref e) => write!(f, "Fetching url: {}", e),
        }
    }
}

impl From<io::Error> for DeserializationError {
    fn from(err: io::Error) -> DeserializationError {
        DeserializationError::IoError(err)
    }
}

impl From<groupy::GroupDecodingError> for DeserializationError {
    fn from(err: groupy::GroupDecodingError) -> DeserializationError {
        DeserializationError::DecodingError(err)
    }
}

impl From<isahc::Error> for DeserializationError {
    fn from(err: isahc::Error) -> DeserializationError {
        DeserializationError::Fetch(err)
    }
}
