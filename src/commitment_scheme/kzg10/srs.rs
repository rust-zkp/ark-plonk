// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! The Public Parameters can also be referred to as the Structured Reference
//! String (SRS).
use super::key::{CommitKey, OpeningKey};
use crate::{error::Error, util};
use ark_ec::{PairingEngine, ProjectiveCurve};
use ark_ff::{PrimeField, UniformRand};
use rand_core::{CryptoRng, RngCore};

/// The Public Parameters can also be referred to as the Structured Reference
/// String (SRS). It is available to both the prover and verifier and allows the
/// verifier to efficiently verify and make claims about polynomials up to and
/// including a configured degree.
#[derive(Debug, Clone)]
pub struct PublicParameters<E: PairingEngine> {
    /// Key used to generate proofs for composed circuits.
    pub(crate) commit_key: CommitKey<E>,
    /// Key used to verify proofs for composed circuits.
    pub(crate) opening_key: OpeningKey<E>,
}

impl<E: PairingEngine> PublicParameters<E> {
    /// Returns an untrimmed [`CommitKey`] reference contained in the
    /// `PublicParameters` instance.
    pub fn commit_key(&self) -> &CommitKey<E> {
        &self.commit_key
    }

    /// Returns an [`OpeningKey`] reference contained in the
    /// `PublicParameters` instance.
    pub fn opening_key(&self) -> &OpeningKey<E> {
        &self.opening_key
    }

    /// Setup generates the public parameters using a random number generator.
    /// This method will in most cases be used for testing and exploration.
    /// In reality, a `Trusted party` or a `Multiparty Computation` will be used
    /// to generate the SRS. Returns an error if the configured degree is less
    /// than one.
    pub fn setup<R: RngCore + CryptoRng + UniformRand>(
        max_degree: usize,
        mut rng: &mut R,
    ) -> Result<PublicParameters<E>, Error> {
        // Cannot commit to constants
        if max_degree < 1 {
            return Err(Error::DegreeIsZero);
        }

        // Generate the secret scalar beta
        let beta = E::Fr::rand(&mut rng);

        // Compute powers of beta up to and including beta^max_degree
        let powers_of_beta = util::powers_of(&beta, max_degree);

        // Powers of G1 that will be used to commit to a specified polynomial
        let g: E::G1Projective = E::G1Projective::rand(&mut rng);
        let powers_of_g: Vec<E::G1Projective> = powers_of_beta
            .iter()
            .copied()
            .map(|s| g.mul(s.into_repr()))
            .collect();
        assert_eq!(powers_of_g.len(), max_degree + 1);

        // Normalise all projective points
        let normalised_g =
            E::G1Projective::batch_normalization_into_affine(&powers_of_g);

        // Compute beta*G2 element and stored cached elements for verifying
        // multiple proofs.
        let h: E::G2Projective = E::G2Projective::rand(&mut rng);
        let beta_h: E::G2Projective = h.mul(beta.into_repr());

        Ok(PublicParameters {
            commit_key: CommitKey {
                powers_of_g: normalised_g,
            },
            opening_key: OpeningKey::new(g.into(), h.into(), beta_h.into()),
        })
    }

    /*
        /// Serialize the [`PublicParameters`] into bytes.
        ///
        /// This operation is designed to store the raw representation of the
        /// contents of the PublicParameters. Therefore, the size of the bytes
        /// outputed by this function is expected to be the double than the one
        /// that [`PublicParameters::to_var_bytes`].
        ///
        /// # Note
        /// This function should be used when we want to serialize the
        /// PublicParameters allowing a really fast deserialization later.
        /// This functions output should not be used by the regular
        /// [`PublicParameters::from_slice`] fn.
        pub fn to_raw_var_bytes(&self) -> Vec<u8> {
            let mut bytes = self.opening_key.to_bytes().to_vec();
            bytes.extend(&self.commit_key.to_raw_var_bytes());

            bytes
        }

        /// Deserialize [`PublicParameters`] from a set of bytes created by
        /// [`PublicParameters::to_raw_var_bytes`].
        ///
        /// The bytes source is expected to be trusted and no checks will be
        /// performed reggarding the content of the points that the bytes
        /// contain serialized.
        ///
        /// # Safety
        /// This function will not produce any memory errors but can deal to the
        /// generation of invalid or unsafe points/keys. To make sure this does not
        /// happen, the inputed bytes must match the ones that were generated by
        /// the encoding functions of this lib.
        pub unsafe fn from_slice_unchecked(bytes: &[u8]) -> Self {
            let opening_key = &bytes[..OpeningKey::SIZE];
            let opening_key = OpeningKey::from_slice(opening_key)
                .expect("Error at OpeningKey deserialization");

            let commit_key = &bytes[OpeningKey::SIZE..];
            let commit_key = CommitKey::from_slice_unchecked(commit_key);

            Self {
                commit_key,
                opening_key,
            }
        }

        /// Serialises a [`PublicParameters`] struct into a slice of bytes.
        pub fn to_var_bytes(&self) -> Vec<u8> {
            let mut bytes = self.opening_key.to_bytes().to_vec();
            bytes.extend(self.commit_key.to_var_bytes().iter());
            bytes
        }

        /// Deserialise a slice of bytes into a Public Parameter struct performing
        /// security and consistency checks for each point that the bytes
        /// contain.
        ///
        /// # Note
        /// This function can be really slow if the [`PublicParameters`] have a
        /// certain degree. If the bytes come from a trusted source such as a
        /// local file, we recommend to use
        /// [`PublicParameters::from_slice_unchecked`] and
        /// [`PublicParameters::to_raw_var_bytes`].
        pub fn from_slice(bytes: &[u8]) -> Result<PublicParameters<E>, Error> {
            if bytes.len() <= OpeningKey::SIZE {
                return Err(Error::NotEnoughBytes);
            }
            let mut buf = bytes;
            let opening_key = OpeningKey::from_reader(&mut buf)?;
            let commit_key = CommitKey::from_slice(&buf)?;

            let pp = PublicParameters {
                commit_key,
                opening_key,
            };

            Ok(pp)
        }
    */
    /// Trim truncates the [`PublicParameters`] to allow the prover to commit to
    /// polynomials up to the and including the truncated degree.
    /// Returns the [`CommitKey`] and [`OpeningKey`] used to generate and verify
    /// proofs.
    ///
    /// Returns an error if the truncated degree is larger than the public
    /// parameters configured degree.
    pub fn trim(
        &self,
        truncated_degree: usize,
    ) -> Result<(CommitKey<E>, OpeningKey<E>), Error> {
        let truncated_prover_key =
            self.commit_key.truncate(truncated_degree)?;
        let opening_key = self.opening_key.clone();
        Ok((truncated_prover_key, opening_key))
    }

    /// Max degree specifies the largest Polynomial
    /// that this prover key can commit to.
    pub fn max_degree(&self) -> usize {
        self.commit_key.max_degree()
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod test {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use rand::SeedableRng;

    #[test]
    fn test_powers_of() {
        let x = Fr::from(10u64);
        let degree = 100u64;

        let powers_of_x = util::powers_of(&x, degree as usize);

        for (i, x_i) in powers_of_x.iter().enumerate() {
            assert_eq!(*x_i, x.pow(&[i as u64, 0, 0, 0]))
        }

        let last_element = powers_of_x.last().unwrap();
        assert_eq!(*last_element, x.pow(&[degree, 0, 0, 0]))
    }

    /*
    #[test]
    fn test_serialise_deserialise_public_parameter() {
        let pp = PublicParameters::setup(1 << 7, &mut OsRng).unwrap();

        let got_pp = PublicParameters::from_slice(&pp.to_var_bytes()).unwrap();

        assert_eq!(got_pp.commit_key.powers_of_g, pp.commit_key.powers_of_g);
        assert_eq!(got_pp.opening_key.g, pp.opening_key.g);
        assert_eq!(got_pp.opening_key.h, pp.opening_key.h);
        assert_eq!(got_pp.opening_key.beta_h, pp.opening_key.beta_h);
    }

    #[test]
    fn public_parameters_bytes_unchecked() {
        let pp = PublicParameters::setup(1 << 7, &mut OsRng).unwrap();

        let pp_p = unsafe {
            let bytes = pp.to_raw_var_bytes();
            PublicParameters::from_slice_unchecked(&bytes)
        };

        assert_eq!(pp.commit_key, pp_p.commit_key);
        assert_eq!(pp.opening_key.g, pp_p.opening_key.g);
        assert_eq!(pp.opening_key.h, pp_p.opening_key.h);
        assert_eq!(pp.opening_key.beta_h, pp_p.opening_key.beta_h);
    }
    */
}
