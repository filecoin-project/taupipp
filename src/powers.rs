use paired::{Engine, PairingCurveAffine};
/// TauParams contains the size of the vector of the tau^i vector in g1 and g2
/// More tau powers are needed in G1 because the Groth16 H query
/// includes terms of the form tau^i * (tau^m - 1) = tau^(i+m) - tau^i
/// where the largest i = m - 2, requiring the computation of tau^(2m - 2)
/// and thus giving us a vector length of 2^22 - 1.
pub struct TauParams {
    pub g1_length: usize,
    pub g2_length: usize,
}

impl TauParams {
    /// new expects the number of multiplicative gates, or equivalently, the
    /// length of the G2 tau vector.
    pub fn new(tau_length: usize) -> TauParams {
        TauParams {
            g2_length: tau_length,
            g1_length: (tau_length << 1) - 1,
        }
    }
}

/// TauPowers contains the actual values of the tau^i in G1 and G2 in affine
/// form.
pub struct TauPowers<E: Engine> {
    pub tau_g1: Vec<E::G1Affine>,
    pub tau_g2: Vec<E::G2Affine>,
}
