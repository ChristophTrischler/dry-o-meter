use argon2::{
    Argon2, PasswordHash,
    password_hash::{SaltString, rand_core::OsRng},
};
use rand::Rng;
use std::iter;

#[allow(dead_code)]
#[inline]
pub fn random_string(limit: usize) -> String {
    iter::repeat(())
        .map(|_| rand::rng().sample(rand::distr::Alphanumeric))
        .map(char::from)
        .take(limit)
        .collect()
}
