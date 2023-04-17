use std::mem::MaybeUninit;

mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// in practice, this number would be much higher (1024, 2048, etc)
pub const SECURITY_PARAM: u64 = 512;

pub struct Puzzle {
    pp: MaybeUninit<bindings::LHP_param_t>,
    z: MaybeUninit<bindings::LHP_puzzle_t>,
}

pub struct Solution {
    pub val: Vec<u8>,
    s: MaybeUninit<bindings::LHP_puzzle_sol_t>,
}

type _Param = MaybeUninit<bindings::LHP_param_t>;
type _Puzzle = MaybeUninit<bindings::LHP_puzzle_t>;

impl Puzzle {
    pub fn single(time: u64, secret: &[u8]) -> Self {
        let mut params: _Param = MaybeUninit::uninit();
        let mut puzzle: _Puzzle = MaybeUninit::uninit();

        unsafe {
            bindings::LHP_init_param(params.as_mut_ptr());
            bindings::LHP_PSetup(params.as_mut_ptr(), SECURITY_PARAM, time);

            bindings::LHP_init_puzzle(puzzle.as_mut_ptr());

            bindings::LHP_PGen(
                puzzle.as_mut_ptr(),
                params.as_mut_ptr(),
                secret.as_ptr() as *mut _,
                secret.len(),
            );

            Self {
                pp: params,
                z: puzzle,
            }
        }
    }

    pub fn batch(time: u64, secrets: &[&[u8]]) -> Self {
        let mut params: _Param = MaybeUninit::uninit();
        let mut final_puzzle: _Puzzle = MaybeUninit::uninit();

        let puzzles_count = secrets.len();
        let mut puzzles: Vec<_Puzzle> = (0..puzzles_count).map(|_| MaybeUninit::uninit()).collect();

        unsafe {
            bindings::LHP_init_param(params.as_mut_ptr());
            bindings::LHP_PSetup(params.as_mut_ptr(), SECURITY_PARAM, time);

            bindings::LHP_init_puzzle(final_puzzle.as_mut_ptr());

            for (secret, puzzle) in std::iter::zip(secrets, &mut puzzles) {
                bindings::LHP_init_puzzle(puzzle.as_mut_ptr());
                bindings::LHP_PGen(
                    puzzle.as_mut_ptr(),
                    params.as_mut_ptr(),
                    secret.as_ptr() as *mut _,
                    secret.len(),
                );
            }

            bindings::LHP_PEval(
                params.as_mut_ptr(),
                puzzles.as_mut_ptr() as *mut _,
                puzzles_count,
                final_puzzle.as_mut_ptr(),
            );

            for mut puzzle in puzzles {
                bindings::LHP_clear_puzzle(puzzle.as_mut_ptr());
                puzzle.assume_init_drop();
            }

            Self {
                pp: params,
                z: final_puzzle,
            }
        }
    }

    pub fn solve(&mut self) -> Solution {
        let mut s: MaybeUninit<bindings::LHP_puzzle_sol_t> = MaybeUninit::uninit();

        unsafe {
            bindings::LHP_init_solution(s.as_mut_ptr());
            bindings::LHP_PSolve(self.pp.as_mut_ptr(), self.z.as_mut_ptr(), s.as_mut_ptr());

            let mpz_struct = s.assume_init().s[0];

            let size = 1;
            let nail = 0;

            let numb = 8 * size - nail;

            let len_in_bytes = (bindings::__gmpz_sizeinbase(&mpz_struct, 2) + numb - 1) / numb;

            let mut val: Vec<u8> = Vec::with_capacity(len_in_bytes);
            let mut countp: usize = 0;

            bindings::__gmpz_export(
                val.as_mut_ptr() as *mut _,
                &mut countp as *mut _,
                1,
                size,
                0,
                nail,
                &mpz_struct,
            );

            val.set_len(countp);

            Solution { val, s }
        }
    }
}

impl Drop for Puzzle {
    fn drop(&mut self) {
        unsafe {
            bindings::LHP_clear_param(self.pp.as_mut_ptr());
            self.pp.assume_init_drop();

            bindings::LHP_clear_puzzle(self.z.as_mut_ptr());
            self.z.assume_init_drop();
        }
    }
}

impl Drop for Solution {
    fn drop(&mut self) {
        unsafe {
            bindings::LHP_clear_solution(self.s.as_mut_ptr());
            self.s.assume_init_drop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::*;

    #[test]
    fn generate_and_solve_puzzle() {
        let secret = b"paymo!";
        let mut puzzle = Puzzle::single(1000000, secret);
        let solution = puzzle.solve();

        assert_eq!(secret, &solution.val[..]);
    }

    #[test]
    fn generate_and_solve_batch_puzzle() {
        let secrets: Vec<&[u8]> = vec![b"paymo!", b"is", b"cool", b"!!!!", b"SENIOR", b"PROJECT!"];
        let mut puzzle = Puzzle::batch(1000000, &secrets);

        let solution = puzzle.solve();
        let expected: [u8; 8] = [80, 83, 18, 241, 145, 139, 12, 148];

        assert_eq!(expected, &solution.val[..]);

        let secrets_sum: BigInt = secrets
            .iter()
            .map(|s| BigInt::from_bytes_be(Sign::NoSign, s))
            .sum();
        let expected_sum = BigInt::from_bytes_be(Sign::NoSign, &expected);

        assert_eq!(expected_sum, secrets_sum);
    }
}
