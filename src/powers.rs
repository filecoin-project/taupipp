use bellman::groth16::aggregate::GenericSRS;
use digest::Digest;
use paired::Engine;
use sha2::Sha256;

/// TauParams contains the size of the vector of the tau^i vector in g1 and g2
pub struct TauParams {
    pub g1_length: usize,
    pub g2_length: usize,
    /// the number of powers that you take from the g1 and g2 vectors
    pub take: usize,
    pub compressed: bool,
}

impl TauParams {
    /// new expects the number of multiplicative gates, or equivalently, the
    /// length of the G2 tau vector.
    pub fn new(tau_length: usize, take: usize, compressed: bool) -> TauParams {
        assert!(take <= tau_length);
        TauParams {
            compressed,
            g2_length: tau_length,
            // More tau powers are needed in G1 because the Groth16 H query
            // includes terms of the form tau^i * (tau^m - 1) = tau^(i+m) - tau^i
            // where the largest i = m - 2, requiring the computation of tau^(2m - 2)
            // and thus giving us a vector length of 2^22 - 1.
            g1_length: (tau_length << 1) - 1,
            take,
        }
    }
}

/// TauPowers contains the actual values of the tau^i in G1 and G2 in affine
/// form.
pub struct TauPowers<E: Engine> {
    pub tau_g1: Vec<E::G1Affine>,
    pub tau_g2: Vec<E::G2Affine>,
}

impl<E: Engine> TauPowers<E> {
    pub fn hash(&self) -> Vec<u8> {
        let mut hash_input = Vec::new();
        self.tau_g1
            .iter()
            .zip(self.tau_g2.iter())
            .for_each(|(g1, g2)| {
                bincode::serialize_into(&mut hash_input, g1).expect("hash point error");
                bincode::serialize_into(&mut hash_input, g2).expect("hash point error");
            });
        Sha256::digest(&hash_input).to_vec()
    }
}

/// this function returns the generic srs required to aggregate Groth16 proofs
/// together. The generic srs will be able to aggregate up to $n$ proofs. $n$
/// must be smaller than half of the size of both CRS otherwise it panics. Both
/// CRS must use the same generators in G1 and G2 otherwise it panics.
pub fn create_ipp_srs<E: Engine>(p1: &TauPowers<E>, p2: &TauPowers<E>) -> GenericSRS<E> {
    // check correct sizes of both crs
    assert!(p1.tau_g1.len() == p1.tau_g2.len());
    assert!(p1.tau_g1.len() == p2.tau_g1.len());
    assert!(p1.tau_g2.len() == p2.tau_g2.len());
    // we make sure the two transcript use the same generators
    let b1 = p1.tau_g1[0] == p2.tau_g1[0];
    let b2 = p1.tau_g2[0] == p2.tau_g2[0];
    if !b1 || !b2 {
        panic!("the two transcript don't use the same bases");
    }
    // size of the CRS we need
    let tn = 2 * p1.tau_g1.len() + 1;

    let g_alpha_powers = p1.tau_g1.iter().take(tn).cloned().collect::<Vec<_>>();
    let g_beta_powers = p2.tau_g1.iter().take(tn).cloned().collect::<Vec<_>>();
    let h_alpha_powers = p1.tau_g2.iter().take(tn).cloned().collect::<Vec<_>>();
    let h_beta_powers = p2.tau_g2.iter().take(tn).cloned().collect::<Vec<_>>();
    GenericSRS::<E> {
        g_alpha_powers,
        g_beta_powers,
        h_alpha_powers,
        h_beta_powers,
    }
}
