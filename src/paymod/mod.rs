extern crate libc;

type mpz_t = [libc::c_ulong; 1]; // or [libc::c_uchar; 16], depending on architecture

#[repr(C)]
struct LHP_puzzle_t {
    u:mpz_t,
    v:mpz_t, 
}

#[repr(C)]
struct LHP_param_t {
	T:mpz_t,
	N:mpz_t,
	g:mpz_t,
	h:mpz_t,
}

pub mod paymo {
    pub fn op_channel() {
        
    }

    pub fn cl_channel() {

    }
    /*
    input: private key, and value to send
    functionality: pay the other wallet on the channel
    */
    pub fn pay() {

    }
    /*

    */

    fn j_spend() {

    }

    fn kt_gen() {

    }
    mod vtlrs {
        /*
        input: security parameter = 128
        output: common reference string
        */
        fn setup() {

        }

        /*
        input: a commitment
        ouput: a signature
        */
        fn fop() {

        }
        /*
        input: signature, transaction, hiding time T, Randomness r
        ouput: a commitment and a proof
        */
        fn com() {

        }
        
        /*
        input: commitment
        output: signature and randomness used in generating the commitment
        */
        fn op() {

        }

        /*
        input: proof, commitment, and tx 
        output: 0/1 if and only iff sig on tx is valid 
        */
        fn vfy() {

        }
    }
}