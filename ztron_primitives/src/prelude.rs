//! Remove generics from basic type.

use pairing::bls12_381::Bls12;

use crate::keys;
use crate::primitives;

pub use keys::{prf_expand, OutgoingViewingKey};
pub use primitives::Diversifier;

/// ask, nsk, ovk
pub type ExpandedSpendingKey = keys::ExpandedSpendingKey<Bls12>;

/// ak, nk
pub type ViewingKey = primitives::ViewingKey<Bls12>;

/// vk(ak, nk), nvk
pub type FullViewingKey = keys::FullViewingKey<Bls12>;

/// ak, nsk
pub type ProofGenerationKey = primitives::ProofGenerationKey<Bls12>;

/// pk_d, d
pub type PaymentAddress = primitives::PaymentAddress<Bls12>;
