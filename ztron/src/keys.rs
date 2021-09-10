use bech32::{FromBase32, ToBase32};
use ff::Field;
use ff::PrimeField;
use group::GroupEncoding;
use std::hash::{Hash, Hasher};
use std::io;
use std::mem;
use std::str::FromStr;
use zcash_primitives::sapling::keys::{ExpandedSpendingKey, FullViewingKey, OutgoingViewingKey};
use zcash_primitives::sapling::{Diversifier, PaymentAddress};

pub fn generate_rcm() -> Vec<u8> {
    let mut rng = rand::rngs::OsRng;
    let rcm = jubjub::Fr::random(&mut rng);
    rcm.to_repr().as_ref().to_vec()
}

#[derive(Clone, PartialEq)]
pub struct ZAddress(PaymentAddress);

impl ::std::fmt::Debug for ZAddress {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("zAddress")
            .field("pk_d", &hex::encode(self.pk_d()))
            .field("d", &hex::encode(self.d()))
            .finish()
    }
}

impl ::std::fmt::Display for ZAddress {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        bech32::encode_to_fmt(
            f,
            "ztron",
            (&self.0.to_bytes()[..]).to_base32(),
            bech32::Variant::Bech32,
        )
        .unwrap()
    }
}

impl ::std::ops::Deref for ZAddress {
    type Target = PaymentAddress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for ZAddress {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        bech32::decode(s)
            .ok()
            .and_then(|(hrp, data, _)| if hrp == "ztron" { Some(data) } else { None })
            .and_then(|data| Vec::<u8>::from_base32(&data).ok())
            .and_then(|raw| if raw.len() == 43 { Some(raw) } else { None })
            .and_then(|raw| PaymentAddress::from_bytes(unsafe { mem::transmute(&raw[0]) }))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid key format"))
            .map(ZAddress)
    }
}

impl Hash for ZAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bytes().hash(state);
    }
}

impl Eq for ZAddress {}

impl ZAddress {
    pub fn pk_d(&self) -> Vec<u8> {
        self.0.pk_d().to_bytes().to_vec()
    }

    pub fn d(&self) -> &[u8] {
        &self.0.diversifier().0[..]
    }
}

pub struct ZKey {
    sk: [u8; 32],
    d: Diversifier,
    esk: ExpandedSpendingKey,
    fvk: FullViewingKey,
    address: ZAddress,
}

impl ::std::fmt::Debug for ZKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("ZKey")
            .field("sk", &hex::encode(self.sk()))
            .field("ask", &hex::encode(self.ask()))
            .field("nsk", &hex::encode(self.nsk()))
            .field("ovk", &hex::encode(&self.ovk().0[..]))
            .field("ak", &hex::encode(self.ak()))
            .field("nk", &hex::encode(self.nk()))
            .field("ivk", &hex::encode(self.ivk()))
            .field("d", &hex::encode(self.d()))
            .field("pk_d", &hex::encode(self.pk_d()))
            .field("payment_address", &self.payment_address().to_string())
            .finish()
    }
}

impl ZKey {
    pub fn new(sk: [u8; 32], d: [u8; 11]) -> Option<Self> {
        let esk = ExpandedSpendingKey::from_spending_key(&sk);
        let fvk = FullViewingKey::from_expanded_spending_key(&esk);
        let d = Diversifier(d);

        let address = fvk.vk.to_payment_address(d)?;

        Some(ZKey {
            sk,
            d,
            esk,
            fvk,
            address: ZAddress(address),
        })
    }

    pub fn from_slice(sk: &[u8], d: &[u8]) -> Option<Self> {
        let mut raw_sk = [0u8; 32];
        raw_sk.copy_from_slice(sk);
        let mut raw_d = [0u8; 11];
        raw_d.copy_from_slice(d);

        Self::new(raw_sk, raw_d)
    }

    pub fn generate() -> Self {
        let sk = rand::random::<[u8; 32]>();
        loop {
            let d = rand::random::<[u8; 11]>();
            // or check: d.g_d::<Bls12>(&JUBJUB).is_some()
            if let Some(zkey) = ZKey::new(sk, d) {
                return zkey;
            }
        }
    }

    pub fn sk(&self) -> &[u8; 32] {
        &self.sk
    }

    pub fn ask(&self) -> Vec<u8> {
        self.esk.ask.to_repr().as_ref().to_vec()
    }

    pub fn nsk(&self) -> Vec<u8> {
        self.esk.nsk.to_repr().as_ref().to_vec()
    }

    pub fn ovk(&self) -> &OutgoingViewingKey {
        &self.esk.ovk
    }

    pub fn ak(&self) -> Vec<u8> {
        self.fvk.vk.ak.to_bytes().to_vec()
    }

    pub fn nk(&self) -> Vec<u8> {
        self.fvk.vk.nk.to_bytes().to_vec()
    }

    pub fn ivk(&self) -> Vec<u8> {
        self.fvk.vk.ivk().to_repr().as_ref().to_vec()
    }

    pub fn d(&self) -> &[u8] {
        &self.d.0[..]
    }

    pub fn pk_d(&self) -> Vec<u8> {
        self.address.0.pk_d().to_bytes().to_vec()
    }

    pub fn payment_address(&self) -> &ZAddress {
        &self.address
    }

    pub fn expsk(&self) -> &ExpandedSpendingKey {
        &self.esk
    }

    pub fn diversifier(&self) -> &Diversifier {
        &self.d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zkey_generate() {
        let zkey = ZKey::generate();
        println!("{:#?}", zkey);
    }

    #[test]
    fn ztron_address_parse() {
        let zaddr: ZAddress = "ztron1wc0f4nnqcl5k7xjrznv662gv88tweqze2myeqf3g4hr9xv882y02f7mmteylu5gdlpx6qrj7s5t"
            .parse()
            .unwrap();

        assert_eq!(hex::encode(zaddr.d()), "761e9ace60c7e96f1a4314");
        assert_eq!(
            hex::encode(zaddr.pk_d()),
            "d9ad290c39d6ec805956c9902628adc65330e7511ea4fb7b5e49fe510df84da0"
        );

        println!("{:?}", zaddr);
    }

    #[test]
    fn zkey_from_slice() {
        let zkey = ZKey::from_slice(
            &hex::decode("0be89fcec248ad504b798145f6785890cf2f7ffa49800fbb9a68240771157de2").unwrap(),
            &hex::decode("761e9ace60c7e96f1a4314").unwrap(),
        )
        .unwrap();

        assert_eq!(
            zkey.payment_address().to_string(),
            "ztron1wc0f4nnqcl5k7xjrznv662gv88tweqze2myeqf3g4hr9xv882y02f7mmteylu5gdlpx6qrj7s5t"
        );
    }

    #[test]
    fn ztron_zkey() {
        // A `curl https://api.nileex.io/wallet/getnewshieldedaddress`
        let mut sk = [0u8; 32];
        hex::decode_to_slice(
            "0be89fcec248ad504b798145f6785890cf2f7ffa49800fbb9a68240771157de2",
            &mut sk[..],
        )
        .unwrap();
        let mut d = [0u8; 11];
        hex::decode_to_slice("761e9ace60c7e96f1a4314", &mut d[..]).unwrap();

        let zkey = ZKey::new(sk, d).unwrap();

        println!("{:#?}", zkey);
        assert_eq!(
            zkey.payment_address().to_string(),
            "ztron1wc0f4nnqcl5k7xjrznv662gv88tweqze2myeqf3g4hr9xv882y02f7mmteylu5gdlpx6qrj7s5t"
        );
        assert_eq!(
            hex::encode(zkey.sk()),
            "0be89fcec248ad504b798145f6785890cf2f7ffa49800fbb9a68240771157de2"
        );
        assert_eq!(
            hex::encode(zkey.ask()),
            "2dd40ff3a771d2b1d9591d7f547c529a0e5b6e7388f1f3a7cd26b6e7c66c9e01"
        );
        assert_eq!(
            hex::encode(zkey.nsk()),
            "bf2504458518201d0ca7290c63c668eaab3ce24627d8e407a84ea06cb5d2b50d"
        );
        assert_eq!(
            hex::encode(&zkey.ovk().0[..]),
            "1c6dc699064b093f2e753b1b23b924e598823c482f163a28446d6b640cb884c6"
        );
        assert_eq!(
            hex::encode(zkey.ak()),
            "6bd298642ec2dbd867f5da20346e3388671b6d9df48b463d57d89c4c2cceb7c6"
        );
        assert_eq!(
            hex::encode(zkey.nk()),
            "8b8b2eee2589da3fbe27b60f6eb9b274039adf85596f2049a327d97c6bdcf5e5"
        );
        assert_eq!(
            hex::encode(zkey.ivk()),
            "cca0d2ebf5b044f910fad48df399f4a9ff4f1ed6bcf929048dc78546b2bcf402"
        );
        assert_eq!(hex::encode(zkey.d()), "761e9ace60c7e96f1a4314");
        assert_eq!(
            hex::encode(zkey.pk_d()),
            "d9ad290c39d6ec805956c9902628adc65330e7511ea4fb7b5e49fe510df84da0"
        );
    }
}
