use curve25519_dalek::{constants::ED25519_BASEPOINT_TABLE, edwards::EdwardsPoint, scalar::Scalar};
use monero_serai::{random_scalar, ringct::hash_to_point};
use rand_core::OsRng;
use sha3::{Digest, Keccak256};

pub fn hash(data: &[u8]) -> [u8; 32] {
    Keccak256::digest(data).into()
}

pub fn generate_user_key_pair() -> (Scalar, EdwardsPoint) {
    let private_key = random_scalar(&mut OsRng);
    let public_key = &private_key * &ED25519_BASEPOINT_TABLE;

    (private_key, public_key)
}

pub fn generate_user_tag(public: &EdwardsPoint, secret: &Scalar) -> EdwardsPoint {
    secret * hash_to_point(*public)
}
