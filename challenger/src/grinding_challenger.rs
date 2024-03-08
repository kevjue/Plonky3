use p3_baby_bear::BabyBear;
use p3_bn254::BN254;
use p3_field::{AbstractField, Field, PrimeField64};
use p3_maybe_rayon::prelude::*;
use p3_symmetric::CryptographicPermutation;
use tracing::instrument;

use crate::{BabyBearBN254Challenger, CanObserve, CanSampleBits, DuplexChallenger};

pub trait GrindingChallenger:
    CanObserve<Self::Witness> + CanSampleBits<usize> + Sync + Clone
{
    type Witness: Field;

    fn grind(&mut self, bits: usize) -> Self::Witness;

    #[must_use]
    fn check_witness(&mut self, bits: usize, witness: Self::Witness) -> bool {
        self.observe(witness);
        self.sample_bits(bits) == 0
    }
}

impl<F, P, const WIDTH: usize> GrindingChallenger for DuplexChallenger<F, P, WIDTH>
where
    F: PrimeField64,
    P: CryptographicPermutation<[F; WIDTH]>,
{
    type Witness = F;

    #[instrument(name = "grind for proof-of-work witness", skip_all)]
    fn grind(&mut self, bits: usize) -> Self::Witness {
        let witness = (0..F::ORDER_U64)
            .into_par_iter()
            .map(|i| F::from_canonical_u64(i))
            .find_any(|witness| self.clone().check_witness(bits, *witness))
            .expect("failed to find witness");
        assert!(self.check_witness(bits, witness));
        witness
    }
}

impl<P> GrindingChallenger for BabyBearBN254Challenger<P>
where
    P: CryptographicPermutation<[BN254; 3]>,
{
    type Witness = BabyBear;

    #[instrument(name = "grind for proof-of-work witness", skip_all)]
    fn grind(&mut self, bits: usize) -> Self::Witness {
        let witness = (0..BabyBear::ORDER_U64)
            .into_par_iter()
            .map(BabyBear::from_canonical_u64)
            .find_any(|witness| self.clone().check_witness(bits, *witness))
            .expect("failed to find witness");
        assert!(self.check_witness(bits, witness));
        witness
    }
}
