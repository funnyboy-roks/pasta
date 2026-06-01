use rand::{Rng, RngExt, distr::Distribution};

pub struct LowerAlphanumeric;

// Adapted from rand::distr::Alphabetic
impl Distribution<char> for LowerAlphanumeric {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        const GEN_ASCII_STR_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        const RANGE: usize = GEN_ASCII_STR_CHARSET.len();
        let offset = rng.random_range(0..RANGE);
        GEN_ASCII_STR_CHARSET[offset] as char
    }
}
