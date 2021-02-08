use crate::powers::{TauParams, TauPowers};
use groupy::{CurveAffine, EncodedPoint};
use paired::Engine;
use rayon::prelude::*;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::path::Path;

/// An URI can designate multiple ways to fetch a power of tau ceremony
#[derive(Clone, Debug)]
pub enum URI {
    /// A local filesystem path
    File(String),
    /// A HTTP endpoint which returns the transcript in the body's answer.
    HTTP(String),
}

impl URI {
    /// Returns an URI that represents the file if it exists or the http
    /// endpoint otherwise.
    pub fn try_from_file(fname: &str, http: &str) -> Self {
        if Path::new(&fname).exists() {
            URI::File(fname.to_string())
        } else {
            URI::HTTP(http.to_string())
        }
    }

    pub fn get_reader(&self) -> Box<dyn Read + Send> {
        match self {
            Self::File(fname) => Box::new(
                OpenOptions::new()
                    .read(true)
                    .open(fname)
                    .expect(&format!("unable open {} in this directory", fname)),
            ) as Box<dyn Read + Send>,
            Self::HTTP(endpoint) => {
                let resp =
                    isahc::get(endpoint).expect(&format!("unable to open endpoint {}", endpoint));
                Box::new(resp.into_body()) as Box<dyn Read + Send>
            }
        }
    }
}

impl fmt::Display for URI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            URI::File(fname) => write!(f, "file uri: {}", fname),
            URI::HTTP(endpoint) => write!(f, "HTTP endpoint uri: {}", endpoint),
        }
    }
}

/// This functions fetches the power of tau transcript using the corresponding
/// method of the URI and returns the tau powers.
/// Note it fetches and reads only the first tau^i in G1 and G2 groups of the
/// full transcript as only this part is needed for IPP.
pub fn read_powers_from<E: Engine>(
    params: &TauParams,
    uri: URI,
) -> Result<TauPowers<E>, DeserializationError> {
    let mut reader = uri.get_reader();
    read_powers(&params, &mut reader)
}

/// read_vec reads up to size * point_length bytes from the reader and only
/// verifies and returns "take" points - useful to advance the reader to a
/// certain point without verifying everything if one does not need the full CRS.
fn read_vec<R: Read + Send, C: EncodedPoint>(
    reader: &mut R,
    size: usize, // read this number of points from the reader
    take: usize, // only verify and take this number of points
) -> Result<Vec<C::Affine>, DeserializationError> {
    assert!(take <= size);
    // Read the encoded elements
    let mut res = vec![C::empty(); take];
    for encoded in &mut res {
        reader.read_exact(encoded.as_mut())?;
    }
    println!(
        "\t- {} points have been read from source - processing them now.",
        take
    );
    let (_, res_affine) = rayon::join(
        || {
            // read the rest that we wish to skip
            let mut empty = C::empty();
            for _ in 0..(size - take) {
                reader
                    .read_exact(empty.as_mut())
                    .expect("failed to read from input");
            }
        },
        || {
            let r = res
                .into_par_iter()
                .enumerate()
                .map(|(i, source)| {
                    source
                        .into_affine()
                        .map_err(|e| {
                            println!("Error at index {}", i);
                            e.into()
                        })
                        .and_then(|source| Ok(source))
                })
                .collect::<Result<Vec<_>, DeserializationError>>();
            println!("\t- Finished processing all points into affine coordinates");
            r
        },
    );
    res_affine
}

/// read some bytes representing the hash - we are not verifying the hash chain
/// here so we don't need those bytes.
fn skip_hash<R: Read>(r: &mut R) {
    let mut tmp = [0; 64];
    r.read(&mut tmp)
        .expect("unable to read BLAKE2b hash of previous contribution");
}

/// read_powers reads only the first tau^i in G1 and G2 groups of the full
/// transcript as only this part is needed for IPP.
fn read_powers<E: Engine, R: Read + Send>(
    params: &TauParams,
    reader: &mut R,
) -> Result<TauPowers<E>, DeserializationError> {
    skip_hash(reader);
    println!("\t- Processing g1 tau elements");
    let g1p = if params.compressed {
        read_vec::<_, <E::G1Affine as CurveAffine>::Compressed>(
            reader,
            params.g1_length,
            params.take,
        )?
    } else {
        read_vec::<_, <E::G1Affine as CurveAffine>::Uncompressed>(
            reader,
            params.g1_length,
            params.take,
        )?
    };
    println!("\t- Processing g2 tau elements");
    let g2p = if params.compressed {
        read_vec::<_, <E::G2Affine as CurveAffine>::Compressed>(
            reader,
            params.g2_length,
            params.take,
        )?
    } else {
        read_vec::<_, <E::G2Affine as CurveAffine>::Uncompressed>(
            reader,
            params.g2_length,
            params.take,
        )?
    };
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
