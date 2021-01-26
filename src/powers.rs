use bellman::groth16::aggregate::srs::{VerifierSRS, SRS};
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

pub fn create_ipp_srs<E: Engine>(
    n: usize,
    p1: &TauPowers<E>,
    p2: &TauPowers<E>,
) -> (SRS<E>, VerifierSRS<E>) {
    // we make sure the two transcript use the same generators
    let b1 = p1.tau_g1[0] == p2.tau_g1[0];
    let b2 = p1.tau_g2[0] == p2.tau_g2[0];
    if !b1 || !b2 {
        panic!("the two transcript don't use the same bases");
    }
    let g_alpha_powers = p1.tau_g1.iter().skip(n).take(n+1).collect::<Vec<_>>();
    let g_beta_powers = p2.tau_g1.iter().skip(n).take(n+1).collect::<Vec<_>>();
    let h_alpha_powers = p1.tau_g2.iter().take(n+1).collect::<Vec<_>>();
    let h_beta_powers = p2.tau_g2.iter().take(n+1).collect::<Vec<_>>();
    let srs = SRS<E> {
        n:n,
        g_alpha_powers,
        h_alpha_powers,
        g_beta_powers,
        h_beta_powers,
    }
    let vk = VerifierSRS<E> {
        n:n,
        g:p1.tau_g1[0].clone(),
        h:p1.tau_g2[0].clone(),
        g_alpha:p1.tau_g1[1].clone(),
        g_beta:p2.tau_g1[1].clone(),
        h_alpha:p1.tau_g2[1].clone(),
        h_beta:p2.tau_g2[1].clone(),
        g_alpha_n1: srs.g_alpha_powers[1].clone(),
        g_beta_n1: srs.g_beta_powers[1].clone(),
    }
    (srs,vk)
}
