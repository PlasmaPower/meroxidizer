use amcl::{
    bls381::{big::Big, bls381::utils},
    errors::AmclError,
};
use log::trace;

pub const SIG_SIZE: usize = 48;

const DST: &[u8] = b"MEROS-V00-CS01-with-BLS12381G1_XMD:SHA-256_SSWU_RO_";

#[derive(Clone)]
pub struct SecretKey(Big);

impl SecretKey {
    pub fn new(bytes: &[u8]) -> Result<SecretKey, AmclError> {
        utils::secret_key_from_bytes(bytes).map(SecretKey)
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; SIG_SIZE] {
        let hash = utils::hash_to_curve_g1(msg, DST);
        let signature = amcl::bls381::pair::g1mul(&hash, &self.0);
        let sig = utils::serialize_g1(&signature);
        trace!(
            "signing {} -> {}",
            hex::encode_upper(msg),
            // TODO: remove cast once we update our minimum rust version enough
            hex::encode_upper(&sig as &[u8])
        );
        sig
    }

    pub fn get_public_key(&self) -> [u8; 96] {
        let point = amcl::bls381::pair::g2mul(&amcl::bls381::ecp2::ECP2::generator(), &self.0);
        utils::serialize_g2(&point)
    }
}
