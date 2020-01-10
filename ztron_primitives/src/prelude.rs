//! Remove generics from basic type.

use pairing::bls12_381::Bls12;

use crate::keys;
use crate::primitives;

pub use crate::JUBJUB;
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::JUBJUB;

    #[test]
    fn payment_address() {
        let raw = [
            22, 106, 69, 173, 225, 17, 87, 175, 245, 51, 184, // Diversifier
            87, 51, 47, 45, 138, 162, 37, 14, 235, 116, 153, 118, 131, 215, 139, 45, 16, 176, 185, 127, 97, 104, 202,
            47, 132, 227, 34, 199, 55, 52, 105, 207, // pk_d
        ];
        let expected_addr = "ztron1ze4ytt0pz9t6lafnhptnxted323z2rhtwjvhdq7h3vk3pv9e0ask3j30sn3j93ehx35u7ku7q0d";

        let addr = PaymentAddress::from_bytes(&raw, &JUBJUB).unwrap();
        assert_eq!(addr.to_string(), expected_addr);

        let parsed_addr: PaymentAddress = expected_addr.parse().unwrap();
        assert_eq!(parsed_addr, addr);
    }
}
