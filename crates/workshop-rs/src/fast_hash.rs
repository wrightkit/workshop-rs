//! Fast fixed-seed hashing for internal maps. std's SipHash is measurable in
//! catalog load and parse; these maps hold no untrusted-key DoS surface worth
//! its cost, and no output may depend on their iteration order.

use std::hash::BuildHasher;

use ahash::{AHasher, RandomState};

/// A `BuildHasher` with fixed seeds. ahash's own `Default` needs the
/// `runtime-rng` feature, which is not enabled for this crate.
#[derive(Clone)]
pub(crate) struct FastState(RandomState);

impl Default for FastState {
    fn default() -> Self {
        Self(RandomState::with_seeds(
            0x243f_6a88_85a3_08d3,
            0x1319_8a2e_0370_7344,
            0xa409_3822_299f_31d0,
            0x082e_fa98_ec4e_6c89,
        ))
    }
}

impl BuildHasher for FastState {
    type Hasher = AHasher;

    fn build_hasher(&self) -> AHasher {
        self.0.build_hasher()
    }
}

pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V, FastState>;
