//! Replacement of zcash_primitives::transaction::builder::Builder.

use crypto_api_chachapoly::ChachaPolyIetf;
use ff::{Field, PrimeField};
use keys::Address;
use lazy_static::lazy_static;
use primitive_types::U256;
use rand::{rngs::OsRng, CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use zcash_primitives::keys::{ExpandedSpendingKey, FullViewingKey, OutgoingViewingKey};
use zcash_primitives::merkle_tree::MerklePath;
use zcash_primitives::note_encryption::{Memo, SaplingNoteEncryption};
use zcash_primitives::primitives::{Diversifier, Note, PaymentAddress, Rseed};
use zcash_primitives::prover::TxProver;
use zcash_primitives::redjubjub::{PrivateKey, PublicKey, Signature};
use zcash_primitives::sapling;
use zcash_primitives::sapling::Node;
use zcash_primitives::transaction::components::{Amount, GROTH_PROOF_SIZE};
use zcash_proofs::prover::LocalTxProver;
use group::GroupEncoding;

use crate::keys::ZAddress;

lazy_static! {
    pub static ref TX_PROVER: LocalTxProver = {
        use std::path::Path;

        let spend_path = "./ztron-params/sapling-spend.params";
        let output_path = "./ztron-params/sapling-output.params";

        eprintln!("loading local tx prover");

        LocalTxProver::new(Path::new(spend_path), Path::new(output_path))
    };
}

// pub use zcash_primitives::transaction::builder::Error;
#[derive(Debug, PartialEq)]
pub enum Error {
    AnchorMismatch,
    BindingSig,
    ChangeIsNegative(Amount),
    InvalidAddress,
    InvalidAmount,
    InvalidMerklePath,
    InvalidRcm,
    NoChangeAddress,
    SpendProof,
    InvalidTransaction(&'static str),
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub fn parse_merkle_path(path: &[u8], position: u64) -> Result<MerklePath<Node>, Error> {
    let mut formated_path = Vec::with_capacity(1065);
    formated_path.push(0x20); // depth = 32
    for seg in path.chunks(32) {
        formated_path.push(0x20); // length = 32
        formated_path.extend_from_slice(seg);
    }
    // position in LE
    formated_path.extend_from_slice(&position.to_le_bytes()[..]);
    assert_eq!(formated_path.len(), 1065);
    MerklePath::from_slice(&formated_path).map_err(|_| Error::InvalidMerklePath)
}

pub fn parse_rcm(value: &[u8]) -> Result<jubjub::Fr, Error> {
    let mut r = [0u8; 32];
    r.copy_from_slice(value);
    // TODO: handle error
    Ok(jubjub::Fr::from_bytes(&r).unwrap())
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransactionType {
    Mint,
    Transfer,
    Burn,
}

struct TransparentInput {
    amount: U256,
}

struct TransparentOutput {
    address: Address,
    amount: U256,
}

struct SaplingSpend {
    expsk: ExpandedSpendingKey,
    diversifier: Diversifier,
    note: Note,
    alpha: jubjub::Fr,
    merkle_path: MerklePath<Node>,
}

pub struct SpendDescription {
    pub cv: jubjub::ExtendedPoint,
    pub anchor: bls12_381::Scalar,
    pub nullifier: [u8; 32],
    pub rk: PublicKey,
    pub zkproof: [u8; GROTH_PROOF_SIZE],
    pub spend_auth_sig: Option<Signature>,
}

impl SpendDescription {
    fn generate_spend_sig(&mut self, spend: &SaplingSpend, sighash: &[u8; 32]) {
        let mut rng = rand::rngs::OsRng;

        let spend_sig = sapling::spend_sig(PrivateKey(spend.expsk.ask), spend.alpha, sighash, &mut rng);
        self.spend_auth_sig = Some(spend_sig);
    }
}

impl SaplingSpend {
    fn generate_spend_proof<P: TxProver>(&self, ctx: &mut P::SaplingProvingContext, prover: &P) -> SpendDescription {
        let fvk = FullViewingKey::from_expanded_spending_key(&self.expsk);
        let nf = {
            let mut raw = [0u8; 32];
            raw.copy_from_slice(&self.note.nf(&fvk.vk, self.merkle_path.position));
            raw
        };
        let proof_generation_key = self.expsk.proof_generation_key();
        let anchor = self.merkle_path.root(Node::new(self.note.cmu().into())).into();

        let (zkproof, cv, rk) = prover
            .spend_proof(
                ctx,
                proof_generation_key,
                self.diversifier,
                self.note.rseed,
                self.alpha,
                self.note.value,
                anchor,
                self.merkle_path.clone(),
            )
            .expect("proving should not fail");

        SpendDescription {
            cv,
            anchor,
            nullifier: nf,
            rk,
            zkproof,
            spend_auth_sig: None,
        }
    }
}

pub struct SaplingOutput {
    ovk: OutgoingViewingKey,
    to: PaymentAddress,
    note: Note,
    memo: Memo,
}

pub struct OutputDescription {
    pub cv: jubjub::ExtendedPoint,
    pub cmu: bls12_381::Scalar,
    pub ephemeral_key: jubjub::ExtendedPoint,
    pub enc_ciphertext: [u8; 580],
    pub out_ciphertext: [u8; 80],
    pub zkproof: [u8; GROTH_PROOF_SIZE],
}

impl SaplingOutput {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        ovk: OutgoingViewingKey,
        to: PaymentAddress,
        value: Amount,
        memo: Option<Memo>,
    ) -> Result<Self, Error> {
        let g_d = match to.g_d() {
            Some(g_d) => g_d,
            None => return Err(Error::InvalidAddress),
        };
        if value.is_negative() {
            return Err(Error::InvalidAmount);
        }

        let rcm = jubjub::Fr::random(rng);

        let note = Note {
            g_d,
            pk_d: to.pk_d().clone(),
            value: value.into(),
            rseed: Rseed::BeforeZip212(rcm),
        };

        Ok(SaplingOutput {
            ovk,
            to,
            note,
            memo: memo.unwrap_or_default(),
        })
    }

    fn generate_output_proof<P: TxProver>(&self, ctx: &mut P::SaplingProvingContext, prover: &P) -> OutputDescription {
        let mut rng = rand::rngs::OsRng;

        let cmu = self.note.cmu(); // note commitment

        let mut enc = SaplingNoteEncryption::new(
            Some(self.ovk),
            self.note.clone(),
            self.to.clone(),
            self.memo.clone(),
            &mut rng,
        );

        let c_enc = enc.encrypt_note_plaintext();

        let epk = enc.epk().clone();

        // zkproof, value_commitment
        let rcm = match self.note.rseed {
            Rseed::BeforeZip212(rcm) => rcm,
            _ => unreachable!(),
        };
        let (zkproof, cv) = prover.output_proof(ctx, *enc.esk(), self.to.clone(), rcm, self.note.value);

        let c_out = enc.encrypt_outgoing_plaintext(&cv, &self.note.cmu());

        OutputDescription {
            cv,
            cmu,
            ephemeral_key: epk.into(),
            enc_ciphertext: c_enc,
            out_ciphertext: c_out,
            zkproof,
        }
    }
}

fn abi_encode_transfer(spends: &[SpendDescription], outputs: &[OutputDescription], binding_sig: &Signature) -> Vec<u8> {
    use ethabi::Token;

    //input: nf, anchor, cv, rk, proof
    //output: cm, cv, epk, proof
    // transfer(
    //    bytes32[10][] input,
    //    bytes32[2][] spendAuthoritySignature,
    //    bytes32[9][] output,
    //    bytes32[2] bindingSignature,
    //    bytes32[21][] c
    // )

    let input = Token::Array(
        spends
            .iter()
            .map(|spend_desc| {
                let mut raw = Vec::with_capacity(10 * 32);
                raw.extend_from_slice(&spend_desc.nullifier[..]);
                raw.extend_from_slice(spend_desc.anchor.to_repr().as_ref());
                raw.extend_from_slice(&spend_desc.cv.to_bytes()[..]);
                spend_desc.rk.write(&mut raw).unwrap();
                raw.extend_from_slice(&spend_desc.zkproof[..]);
                Token::FixedBytes(raw)
            })
            .collect(),
    );
    let spend_auth_sig = Token::Array(
        spends
            .iter()
            .map(|spend| {
                let mut raw = Vec::with_capacity(64);
                spend.spend_auth_sig.as_ref().unwrap().write(&mut raw).unwrap();
                Token::FixedBytes(raw)
            })
            .collect(),
    );
    let output = Token::Array(
        outputs
            .iter()
            .map(|output_desc| {
                let mut raw = Vec::with_capacity(9 * 32);
                raw.extend_from_slice(output_desc.cmu.to_repr().as_ref());
                raw.extend_from_slice(&output_desc.cv.to_bytes());
                raw.extend_from_slice(&output_desc.ephemeral_key.to_bytes());
                raw.extend_from_slice(&output_desc.zkproof[..]);
                Token::FixedBytes(raw)
            })
            .collect(),
    );
    let binding_signature = {
        let mut raw = Vec::with_capacity(64);
        binding_sig.write(&mut raw).unwrap();
        Token::FixedBytes(raw)
    };

    let c = Token::Array(
        outputs
            .iter()
            .map(|output_desc| {
                let mut raw = Vec::with_capacity(21 * 32);
                raw.extend_from_slice(&output_desc.enc_ciphertext[..]);
                raw.extend_from_slice(&output_desc.out_ciphertext[..]);
                Token::FixedBytes(raw)
            })
            .collect(),
    );
    let parameters = [input, spend_auth_sig, output, binding_signature, c];

    ethabi::encode(&parameters)
}

fn abi_encode_burn(
    spend_desc: &SpendDescription,
    maybe_output: Option<&OutputDescription>,
    transparent_output: &TransparentOutput,
    binding_sig: &Signature,
    burn_cipher: &[u8; 80],
) -> Vec<u8> {
    use ethabi::Token;

    //input: nf, anchor, cv, rk, proof
    //output: cm, cv, epk, proof
    //function burn(
    //    bytes32[10] calldata input,
    //    bytes32[2] calldata spendAuthoritySignature,
    //    uint256 rawValue,
    //    bytes32[2] calldata bindingSignature,
    //    address payTo,
    //    bytes32[3] calldata burnCipher, // encryptBurnMessageByOvk(ovk, toAmount, transparentToAddress);
    //    bytes32[9][] calldata output,
    //    bytes32[21][] calldata c)

    let input = {
        let mut raw = Vec::with_capacity(10 * 32);
        raw.extend_from_slice(&spend_desc.nullifier[..]);
        raw.extend_from_slice(spend_desc.anchor.to_repr().as_ref());
        raw.extend_from_slice(spend_desc.cv.to_bytes().as_ref());
        spend_desc.rk.write(&mut raw).unwrap();
        raw.extend_from_slice(&spend_desc.zkproof[..]);
        Token::FixedBytes(raw)
    };
    let spend_auth_sig = {
        let mut raw = Vec::with_capacity(64);
        spend_desc.spend_auth_sig.as_ref().unwrap().write(&mut raw).unwrap();
        Token::FixedBytes(raw)
    };
    let raw_value = {
        let mut raw = [0u8; 32];
        transparent_output.amount.to_big_endian(&mut raw[..]);
        Token::FixedBytes(raw.to_vec())
    };
    let binding_signature = {
        let mut raw = Vec::with_capacity(64);
        binding_sig.write(&mut raw).unwrap();
        Token::FixedBytes(raw)
    };
    let pay_to = {
        let mut raw = [0u8; 32];
        raw[12..].copy_from_slice(transparent_output.address.as_tvm_bytes());
        // FIXME: should use Token::Address
        Token::FixedBytes(raw.to_vec())
    };
    let burn_cipher = Token::FixedBytes(burn_cipher.to_vec());
    let output = Token::Array(
        maybe_output
            .iter()
            .map(|output_desc| {
                let mut raw = Vec::with_capacity(9 * 32);
                raw.extend_from_slice(output_desc.cmu.to_repr().as_ref());
                raw.extend_from_slice(output_desc.cv.to_bytes().as_ref());
                raw.extend_from_slice(output_desc.ephemeral_key.to_bytes().as_ref());
                raw.extend_from_slice(&output_desc.zkproof[..]);
                Token::FixedBytes(raw)
            })
            .collect(),
    );
    let c = Token::Array(
        maybe_output
            .iter()
            .map(|output_desc| {
                let mut raw = Vec::with_capacity(21 * 32);
                raw.extend_from_slice(&output_desc.enc_ciphertext[..]);
                raw.extend_from_slice(&output_desc.out_ciphertext[..]);
                Token::FixedBytes(raw)
            })
            .collect(),
    );

    let parameters = [
        input,
        spend_auth_sig,
        raw_value,
        binding_signature,
        pay_to,
        burn_cipher,
        output,
        c,
    ];
    ethabi::encode(&parameters)
}

/// Generates a Transaction from its inputs and outputs.
pub struct Builder<R: RngCore + CryptoRng> {
    rng: R,
    contract_address: Address,
    scaling_factor: U256,
    pub value_balance: Amount,
    anchor: Option<bls12_381::Scalar>,
    spends: Vec<SaplingSpend>,
    outputs: Vec<SaplingOutput>,
    transparent_input: Option<TransparentInput>,
    transparent_output: Option<TransparentOutput>,
    // change_address: Option<(OutgoingViewingKey, PaymentAddress<Bls12>)>,
}

impl Builder<OsRng> {
    pub fn new(contract_address: Address, scaling_exponent: u8) -> Self {
        Builder::new_with_rng(contract_address, scaling_exponent, OsRng)
    }
}

impl<R: RngCore + CryptoRng> Builder<R> {
    pub fn new_with_rng(contract_address: Address, scaling_exponent: u8, rng: R) -> Self {
        Builder {
            rng,
            contract_address,
            scaling_factor: U256::exp10(scaling_exponent as usize),
            value_balance: Amount::zero(),
            anchor: None,
            spends: vec![],
            outputs: vec![],
            transparent_input: None,
            transparent_output: None,
        }
    }

    /// Adds a Sapling note to be spent in this transaction.
    pub fn add_sapling_spend(
        &mut self,
        expsk: ExpandedSpendingKey,
        diversifier: Diversifier,
        note: Note,
        merkle_path: MerklePath<Node>,
    ) -> Result<(), Error> {
        if self.spends.len() >= 2 {
            return Err(Error::InvalidTransaction("too many sapling spends"));
        }

        // Consistency check: all anchors must equal the first one
        let cm = Node::new(note.cmu().into());
        if let Some(anchor) = self.anchor {
            let path_root: bls12_381::Scalar = merkle_path.root(cm).into();
            if path_root != anchor {
                return Err(Error::AnchorMismatch);
            }
        } else {
            self.anchor = Some(merkle_path.root(cm).into())
        }

        let alpha = jubjub::Fr::random(&mut self.rng);

        self.value_balance += Amount::from_u64(note.value).map_err(|_| Error::InvalidAmount)?;

        self.spends.push(SaplingSpend {
            expsk,
            diversifier,
            note,
            alpha,
            merkle_path,
        });

        Ok(())
    }

    /// Adds a Sapling address to send funds to.
    pub fn add_sapling_output(
        &mut self,
        ovk: OutgoingViewingKey,
        to: ZAddress,
        value: Amount,
        memo: Option<Memo>,
    ) -> Result<(), Error> {
        if self.outputs.len() >= 2 {
            return Err(Error::InvalidTransaction("too many sapling output"));
        }

        let output = SaplingOutput::new(&mut self.rng, ovk, (*to).clone(), value, memo)?;
        self.value_balance -= value;
        self.outputs.push(output);

        Ok(())
    }

    /// Adds a transparent coin to be spent in this transaction.
    pub fn add_transparent_input(&mut self, value: U256) -> Result<(), Error> {
        if self.transparent_input.is_some() {
            return Err(Error::InvalidTransaction("mint can only have one transparent input"));
        }
        let input = TransparentInput { amount: value };
        self.transparent_input = Some(input);
        Ok(())
    }

    /// Adds a transparent address to send funds to.
    pub fn add_transparent_output(&mut self, to: &Address, value: U256) -> Result<(), Error> {
        if self.transparent_output.is_some() {
            return Err(Error::InvalidTransaction("burn can only have one transparent output"));
        }
        let output = TransparentOutput {
            address: to.clone(),
            amount: value,
        };
        self.transparent_output = Some(output);

        Ok(())
    }

    fn transaction_type(&self) -> Result<TransactionType, Error> {
        if self.transparent_input.is_some() {
            if self.outputs.len() == 1 && self.spends.is_empty() && self.transparent_output.is_none() {
                return Ok(TransactionType::Mint);
            }
            return Err(Error::InvalidTransaction(
                "mint must be a transaction to 1 shielded output",
            ));
        } else if self.transparent_output.is_some() {
            if self.spends.len() == 1 && self.outputs.len() <= 1 && self.transparent_input.is_none() {
                return Ok(TransactionType::Burn);
            }
            return Err(Error::InvalidTransaction(
                "burn must be a transaction from 1 shielded output, to max 1 shielded output",
            ));
        } else {
            if self.spends.len() >= 1 && self.outputs.len() >= 1 {
                return Ok(TransactionType::Transfer);
            }
            return Err(Error::InvalidTransaction("invalid mint, burn, or transfer"));
        }
    }

    fn build_mint(self, prover: &impl TxProver) -> Result<Vec<u8>, Error> {
        if self.value_balance.is_positive() {
            return Err(Error::InvalidAmount);
        }
        let transparent_input_value = self.transparent_input.as_ref().unwrap().amount;
        let shielded_output_value = -i64::from(self.value_balance);
        if U256::from(shielded_output_value) * self.scaling_factor != transparent_input_value {
            return Err(Error::InvalidTransaction("input & output amount mismatch"));
        }

        let mut ctx = prover.new_sapling_proving_context();

        let output_desc = self.outputs[0].generate_output_proof(&mut ctx, prover);

        let mut transaction_data = Vec::with_capacity(1024);
        transaction_data.extend_from_slice(self.contract_address.as_tvm_bytes());
        // receive note value
        transaction_data.extend_from_slice(&shielded_output_value.to_be_bytes()[..]);
        // encodeReceiveDescriptionWithoutC
        transaction_data.extend_from_slice(output_desc.cmu.to_repr().as_ref());
        transaction_data.extend_from_slice(output_desc.cv.to_bytes().as_ref());
        transaction_data.extend_from_slice(output_desc.ephemeral_key.to_bytes().as_ref());
        transaction_data.extend_from_slice(&output_desc.zkproof[..]);
        // encodeCencCout
        transaction_data.extend_from_slice(&output_desc.enc_ciphertext[..]);
        transaction_data.extend_from_slice(&output_desc.out_ciphertext[..]);
        transaction_data.extend(&[0u8; 12]);

        let sighash = {
            let mut hasher = Sha256::new();
            hasher.update(&transaction_data);
            hasher.finalize()
        };
        let binding_sig = prover
            .binding_sig(&mut ctx, self.value_balance, sighash.as_ref())
            .map_err(|()| Error::BindingSig)?;

        let mut parameter = vec![0u8; 32];

        let raw_value = U256::from(shielded_output_value) * U256::exp10(18); // value * scaleFactor
        raw_value.to_big_endian(&mut parameter[..32]);

        parameter.extend_from_slice(output_desc.cmu.to_repr().as_ref());
        parameter.extend_from_slice(output_desc.cv.to_bytes().as_ref());
        parameter.extend_from_slice(output_desc.ephemeral_key.to_bytes().as_ref());
        parameter.extend_from_slice(&output_desc.zkproof[..]);

        binding_sig.write(&mut parameter).unwrap();

        parameter.extend_from_slice(&output_desc.enc_ciphertext[..]);
        parameter.extend_from_slice(&output_desc.out_ciphertext[..]);
        parameter.extend(&[0u8; 12]);

        Ok(parameter)
    }

    fn build_transfer(self, prover: &impl TxProver) -> Result<Vec<u8>, Error> {
        if self.value_balance != Amount::zero() {
            return Err(Error::InvalidAmount);
        }

        let mut ctx = prover.new_sapling_proving_context();

        let mut spend_descs: Vec<_> = self
            .spends
            .iter()
            .map(|output| output.generate_spend_proof(&mut ctx, prover))
            .collect();

        let output_descs: Vec<_> = self
            .outputs
            .iter()
            .map(|output| output.generate_output_proof(&mut ctx, prover))
            .collect();

        let mut transaction_data = Vec::with_capacity(1024);
        transaction_data.extend_from_slice(self.contract_address.as_tvm_bytes());
        for spend in &spend_descs {
            // encodeSpendDescriptionWithoutSpendAuthSig
            transaction_data.extend_from_slice(&spend.nullifier[..]);
            transaction_data.extend_from_slice(spend.anchor.to_repr().as_ref());
            transaction_data.extend_from_slice(spend.cv.to_bytes().as_ref());
            spend.rk.write(&mut transaction_data).unwrap();
            transaction_data.extend_from_slice(&spend.zkproof[..]);
        }

        for output in &output_descs {
            // encodeReceiveDescriptionWithoutC
            transaction_data.extend_from_slice(output.cmu.to_repr().as_ref());
            transaction_data.extend_from_slice(output.cv.to_bytes().as_ref());
            transaction_data.extend_from_slice(output.ephemeral_key.to_bytes().as_ref());
            transaction_data.extend_from_slice(&output.zkproof[..]);
        }

        for output in &output_descs {
            // encodeCencCout
            transaction_data.extend_from_slice(&output.enc_ciphertext[..]);
            transaction_data.extend_from_slice(&output.out_ciphertext[..]);
            transaction_data.extend(&[0u8; 12]);
        }

        let sighash = {
            let mut hasher = Sha256::new();
            hasher.update(&transaction_data);
            hasher.finalize()
        };

        for (desc, spend) in spend_descs.iter_mut().zip(self.spends.iter()) {
            desc.generate_spend_sig(spend, sighash.as_ref());
        }
        for desc in &spend_descs {
            println!("!!! => {:?}", desc.spend_auth_sig);
        }

        let binding_sig = prover
            .binding_sig(&mut ctx, self.value_balance, sighash.as_ref())
            .map_err(|_| Error::BindingSig)?;

        Ok(abi_encode_transfer(&spend_descs, &output_descs, &binding_sig))
    }

    fn encrypt_burn_message(&self) -> Result<[u8; 80], Error> {
        // encryptBurnMessageByOvk
        let ovk = self.spends[0].expsk.ovk;
        let mut plaintext = [0u8; 64];
        let t_output = self.transparent_output.as_ref().unwrap();
        t_output.amount.to_big_endian(&mut plaintext[..32]);
        plaintext[32..32 + 21].copy_from_slice(t_output.address.as_bytes());

        let mut output = [0u8; 80];
        assert_eq!(
            ChachaPolyIetf::aead_cipher()
                .seal_to(&mut output, &plaintext, &[], &ovk.0[..], &[0u8; 12])
                .unwrap(),
            80
        );
        Ok(output)
    }

    fn build_burn(self, prover: &impl TxProver) -> Result<Vec<u8>, Error> {
        if self.value_balance.is_negative() {
            return Err(Error::InvalidAmount);
        }
        let transparent_output_value = self.transparent_output.as_ref().unwrap().amount;
        let shielded_input_value = i64::from(self.value_balance);
        if U256::from(shielded_input_value) * self.scaling_factor != transparent_output_value {
            return Err(Error::InvalidTransaction("input & output amount mismatch"));
        }

        let mut ctx = prover.new_sapling_proving_context();

        let mut spend_desc = self.spends[0].generate_spend_proof(&mut ctx, prover);

        let maybe_output_desc = if self.outputs.len() == 1 {
            Some(self.outputs[0].generate_output_proof(&mut ctx, prover))
        } else {
            None
        };

        let mut transaction_data = Vec::with_capacity(1024);
        transaction_data.extend_from_slice(self.contract_address.as_tvm_bytes());
        // encodeSpendDescriptionWithoutSpendAuthSig
        transaction_data.extend_from_slice(&spend_desc.nullifier[..]);
        transaction_data.extend_from_slice(spend_desc.anchor.to_repr().as_ref());
        transaction_data.extend_from_slice(spend_desc.cv.to_bytes().as_ref());
        spend_desc.rk.write(&mut transaction_data).unwrap();
        transaction_data.extend_from_slice(&spend_desc.zkproof[..]);

        if let Some(ref output_desc) = maybe_output_desc {
            // encodeReceiveDescriptionWithoutC
            transaction_data.extend_from_slice(output_desc.cmu.to_repr().as_ref());
            transaction_data.extend_from_slice(output_desc.cv.to_bytes().as_ref());
            transaction_data.extend_from_slice(output_desc.ephemeral_key.to_bytes().as_ref());
            transaction_data.extend_from_slice(&output_desc.zkproof[..]);
            // encodeCencCout
            transaction_data.extend_from_slice(&output_desc.enc_ciphertext[..]);
            transaction_data.extend_from_slice(&output_desc.out_ciphertext[..]);
            transaction_data.extend(&[0u8; 12]);
        }

        transaction_data.extend_from_slice(self.transparent_output.as_ref().unwrap().address.as_tvm_bytes());
        transaction_data.extend_from_slice(&i64::from(self.value_balance).to_be_bytes()[..]);

        let sighash = {
            let mut hasher = Sha256::new();
            hasher.update(&transaction_data);
            hasher.finalize()
        };

        spend_desc.generate_spend_sig(&self.spends[0], sighash.as_ref());

        let binding_sig = prover
            .binding_sig(&mut ctx, self.value_balance, sighash.as_ref())
            .map_err(|_| Error::BindingSig)?;

        let burn_ciphertext = self.encrypt_burn_message()?;

        Ok(abi_encode_burn(
            &spend_desc,
            maybe_output_desc.as_ref(),
            self.transparent_output.as_ref().unwrap(),
            &binding_sig,
            &burn_ciphertext,
        ))
    }

    pub fn build(self, prover: &impl TxProver) -> Result<(TransactionType, Vec<u8>), Error> {
        // also check validity
        let txn_type = self.transaction_type()?;

        let ret = match txn_type {
            TransactionType::Mint => self.build_mint(prover)?,
            TransactionType::Transfer => self.build_transfer(prover)?,
            TransactionType::Burn => self.build_burn(prover)?,
        };
        Ok((txn_type, ret))
    }
}
