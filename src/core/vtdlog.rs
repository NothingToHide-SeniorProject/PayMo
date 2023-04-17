#![allow(non_snake_case)]
use curve25519_dalek::{
    constants::ED25519_BASEPOINT_TABLE, edwards::EdwardsPoint, scalar::Scalar, traits::Identity,
};
use monero_serai::random_scalar;
use rand::Rng;
use rand_core::OsRng;

use super::lhtlp::Puzzle;

// in practice, those numbers would be much higher
pub const N: usize = 20;
pub const THRESHOLD: usize = 11;

// the Lagrange polynomial we use only has values calculated at 0
pub struct LagrangePolynomial {
    coefficients: Vec<Scalar>,
    basis: Vec<LagrangeBasis>,
}

impl LagrangePolynomial {
    pub fn from_random_coefficients() -> Self {
        let mut coefficients = Vec::with_capacity(N);
        for _ in 0..N {
            coefficients.push(random_scalar(&mut OsRng));
        }

        let basis = coefficients
            .iter()
            .enumerate()
            .map(|(i, _)| LagrangeBasis::from_coefficients(i, &coefficients))
            .collect();

        Self {
            coefficients,
            basis,
        }
    }

    pub fn coefficients(&self) -> &[Scalar] {
        &self.coefficients
    }

    pub fn basis(&self) -> &[LagrangeBasis] {
        &self.basis
    }
}

pub struct LagrangeBasis {
    pub value_at_zero: Scalar,
}

impl LagrangeBasis {
    pub fn from_coefficients(index: usize, coefficients: &[Scalar]) -> Self {
        let mut value_at_zero = Scalar::one();

        for (i, coefficient) in coefficients.iter().enumerate() {
            if i == index {
                continue;
            }

            let numerator = coefficient;
            let denominator = coefficient - coefficients[index];

            value_at_zero *= numerator * denominator.invert();
        }

        Self { value_at_zero }
    }

    pub fn inverse_at_zero(&self) -> Scalar {
        self.value_at_zero.invert()
    }

    pub fn value_at_zero(&self) -> Scalar {
        self.value_at_zero
    }
}

// the puzzles should be batched, otherwise the solver can just paralellize the
// solving process; however, I am not sure how to extract each x_i from the batched puzzle;
// therefore, for simplicity and since this is a PoC, we will solve the puzzles one by one.

pub struct Vtdlog {
    c: Commit,
}

// in the real protocol described in the PayMo paper, the solver would give
// I to the prover, where |I| = t-1, which would then reveal the x_i for i \in I,
// and then we solver would run verification/proof algorithm, and then
// would solve n − t + 1 puzzles.
//
// however, we found the explanation in the paper really confusing, and giving the
// time constraints and the fact we are skipping the proofs, we decided to deviate
// from the paper.
//
// we deviate as follows: the first t-1 shares will always be the randomized ones and correct ones;
// for the n-t-1 left, one will be correct, the others will all be randomized.
// therefore, the solver will need to keep solving the puzzles until it finds the one that
// allows to reconstruct the secret.
//
// for this to work, the solver will need to solve, in the worst case, *all* the puzzles.
//
// also, note that the puzzles need to be batched, since otherwise the solver can just solve
// the t puzzles in parallel. for simplicity, and given that it is a bit unclear how to
// extract individual puzzles results from the batched puzzle, we decided to make
// the solver solve the puzzles one by one.
//
//
//
//
// of course, in a real implementation, everything described in the paper should be followed!
// there is no point in not following the paper but to make the implementation simpler.
//
// in fact, we could just wrap x in a puzzle and that's it, if we really want to make things
// simpler (by simpler = Alice and Bob are honest).
//
// we did not do that because we actually started implementing the protocol described in the paper,
// but found its description confusing, and so we decided to play around with some ideas in the
// original protocol, even though in the context of our implementation, they don't improve
// security.
impl Vtdlog {
    pub fn commit(time: u64, x: Scalar) -> Commit {
        let lagrange = LagrangePolynomial::from_random_coefficients();
        let basis = lagrange.basis();

        let H = &x * &ED25519_BASEPOINT_TABLE;

        let mut x_s = Vec::with_capacity(N);
        let mut H_s = Vec::with_capacity(N);

        let mut sum_t_xs = Scalar::zero();
        let mut sum_t_Hs = EdwardsPoint::identity();

        let mut puzzles = Vec::with_capacity(N);

        for i in 1..=THRESHOLD - 1 {
            let x_i = random_scalar(&mut OsRng);
            let H_i = &x_i * &ED25519_BASEPOINT_TABLE;

            x_s.push(x_i);
            H_s.push(H_i);

            sum_t_xs += x_i * basis[i - 1].value_at_zero();
            sum_t_Hs += H_i * basis[i - 1].value_at_zero();

            puzzles.push(Puzzle::single(time, x_i.as_bytes()));
        }

        let mut rng = rand::thread_rng();
        let correct = rng.gen_range(THRESHOLD..=N);

        for i in THRESHOLD..=N {
            let x_i;
            let H_i;

            if i == correct {
                x_i = (x - sum_t_xs) * basis[i - 1].inverse_at_zero();
                H_i = (H - sum_t_Hs) * basis[i - 1].inverse_at_zero();
            } else {
                x_i = random_scalar(&mut OsRng);
                H_i = &x_i * &ED25519_BASEPOINT_TABLE;
            }

            x_s.push(x_i);
            H_s.push(H_i);

            puzzles.push(Puzzle::single(time, x_i.as_bytes()));
        }

        Commit {
            H,
            H_s,
            lagrange,
            puzzles,
        }
    }

    pub fn from_commit(commit: Commit) -> Self {
        Self { c: commit }
    }

    pub fn solve(&mut self) -> Scalar {
        let basis = self.c.lagrange.basis();
        let mut x = Scalar::zero();

        for i in 1..=N {
            let puzzle = &mut self.c.puzzles[i - 1];

            let x_i = puzzle.solve();
            let x_i = Scalar::from_bytes_mod_order(x_i.val.clone().try_into().unwrap());

            let H_i = &x_i * &ED25519_BASEPOINT_TABLE;

            if H_i != &x_i * &ED25519_BASEPOINT_TABLE {
                panic!("H_i != x_i * G");
            }

            if i < THRESHOLD {
                x += x_i * basis[i - 1].value_at_zero();
            } else {
                let candidate = x + x_i * basis[i - 1].value_at_zero();

                if self.c.H == &candidate * &ED25519_BASEPOINT_TABLE {
                    return candidate;
                }
            }
        }

        unreachable!()
    }
}

pub struct Commit {
    pub H: EdwardsPoint,
    pub H_s: Vec<EdwardsPoint>,

    pub lagrange: LagrangePolynomial,
    pub puzzles: Vec<Puzzle>,
}

pub struct Solution {
    pub x: Scalar,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lagrange_basis_inverse() {
        let lagrange_polynomial = LagrangePolynomial::from_random_coefficients();
        let basis = lagrange_polynomial.basis();

        assert_eq!(
            basis[0].inverse_at_zero(),
            basis[0].value_at_zero().invert()
        );
    }

    #[test]
    fn test_vtdlog() {
        let secret = random_scalar(&mut OsRng);

        let commitment = Vtdlog::commit(100, secret);

        let mut vtdlog = Vtdlog::from_commit(commitment);

        let solution = vtdlog.solve();
        assert_eq!(solution, secret);
    }
}
