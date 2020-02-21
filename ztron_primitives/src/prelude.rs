//! Remove generics from basic type.

use ff::{PrimeField, PrimeFieldRepr};
use pairing::bls12_381::Bls12;

use crate::jubjub::JubjubEngine;
use crate::keys;
use crate::note_encryption::generate_esk;
use crate::primitives;

pub use crate::keys::{prf_expand, OutgoingViewingKey};
pub use crate::note_encryption::{Memo, SaplingNoteEncryption};
pub use crate::primitives::Diversifier;
pub use crate::transaction::components::Amount;
pub use crate::JUBJUB;

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

/// Generate uniformly random scalar in Jubjub. The result is of length 32.
pub fn generate_r() -> <Bls12 as JubjubEngine>::Fs {
    let mut rng = rand::rngs::OsRng;
    // Fs::random(&mut rng)
    generate_esk(&mut rng)
    // let mut raw = [0u8; 32];
    // generate_esk(&mut rng)
    // generate_esk(&mut rng)
    // .into_repr()
    // .write_le(&mut raw[..])
    // .expect("write ok");
    // raw
}

/// (PaymentAddress, sk)
pub fn generate_zkey_pair() -> (PaymentAddress, [u8; 32], ExpandedSpendingKey, FullViewingKey) {
    let sk = rand::random::<[u8; 32]>();

    let mut d: [u8; 11];
    let mut diversifier: Diversifier;
    loop {
        d = rand::random();
        diversifier = Diversifier(d);
        if diversifier.g_d::<Bls12>(&JUBJUB).is_some() {
            break;
        }
    }

    let esk = ExpandedSpendingKey::from_spending_key(&sk);
    let fvk = FullViewingKey::from_expanded_spending_key(&esk, &JUBJUB);

    let zaddr = fvk.vk.to_payment_address(diversifier, &JUBJUB).unwrap();

    (zaddr, sk, esk, fvk)
}

pub fn rcm_to_bytes(rcm: <Bls12 as JubjubEngine>::Fs) -> Vec<u8> {
    let mut raw = vec![];
    rcm.into_repr().write_le(&mut raw).expect("write ok");
    raw
}

/// value, r
pub type ValueCommitment = primitives::ValueCommitment<Bls12>;

/// pk_d, d, value, r
pub type Note = primitives::Note<Bls12>;

pub fn compute_note_commitment(note: &Note) -> Vec<u8> {
    let mut result = vec![];
    note.cm(&JUBJUB)
        .into_repr()
        .write_le(&mut result)
        .expect("length is 32 bytes");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JUBJUB;
    use ff::PrimeField; // into_repr()
    use ff::PrimeFieldRepr;
    use hex::{FromHex, ToHex};

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

    #[test]
    fn generate_address() {
        let _ = generate_zkey_pair();
    }

    #[test]
    fn generate_from_sk() {
        // The following address is generated from nile testnet rpc
        let sk = Vec::from_hex("04ccdf23099b4be2f2ab3a41877e94ff4d2ffaff4a9eb5fc4d0d6db685142d49").unwrap();
        let d = Vec::from_hex("166a45ade11157aff533b8").unwrap();
        let mut raw_d = [0u8; 11];
        (&mut raw_d[..]).copy_from_slice(&d);
        let diversifier = Diversifier(raw_d);

        let esk = ExpandedSpendingKey::from_spending_key(&sk);

        let mut buf: Vec<u8> = vec![];
        esk.ask.into_repr().write_le(&mut buf).unwrap();
        assert_eq!(
            buf.encode_hex::<String>(),
            "8c893dfa38956290f2a1df9e6019b4a6c5f670613583948d8d975dcbccf03407"
        );

        buf.clear();
        esk.nsk.into_repr().write_le(&mut buf).unwrap();
        assert_eq!(
            buf.encode_hex::<String>(),
            "560832b298c76f021126b35bfdd3d4bb62ec0d632029674b3e9157f1bff6b208"
        );

        assert_eq!(
            esk.ovk.0.encode_hex::<String>(),
            "034484bed6abcd44ca9a8af1dd64c8b66d70a0a92471dc24b87b5bfdba8f0ef9"
        );

        let fvk = FullViewingKey::from_expanded_spending_key(&esk, &JUBJUB);

        buf.clear();
        fvk.vk.ak.write(&mut buf).unwrap();
        assert_eq!(
            buf.encode_hex::<String>(),
            "3255f7f2280657560a271f5b15e14ff9cfeae7b16e7f5910f904f8fe0ce45db6"
        );

        buf.clear();
        fvk.vk.nk.write(&mut buf).unwrap();
        assert_eq!(
            buf.encode_hex::<String>(),
            "c10e516acb4a2da828c0d31da54d9441f88f4d5713630c1809b9ebb3f7c4fbd4"
        );

        buf.clear();
        fvk.vk.ivk().into_repr().write_le(&mut buf).unwrap();
        assert_eq!(
            buf.encode_hex::<String>(),
            "b0456583f7a43c05ae2ec72905575ff5737fb2f652d4c0b4bc93849217481006"
        );

        assert_eq!(
            fvk.vk.to_payment_address(diversifier, &JUBJUB).unwrap().to_string(),
            "ztron1ze4ytt0pz9t6lafnhptnxted323z2rhtwjvhdq7h3vk3pv9e0ask3j30sn3j93ehx35u7ku7q0d"
        );
    }
}
