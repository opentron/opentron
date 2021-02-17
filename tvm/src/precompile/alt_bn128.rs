pub fn ecadd(input: &[u8]) -> Option<Vec<u8>> {
    use bn::{AffineG1, Fq, Group, G1};

    let words: Vec<_> = input.chunks(32).collect();

    let x1 = Fq::from_slice(words[0]).ok()?;
    let y1 = Fq::from_slice(words[1]).ok()?;

    let x2 = Fq::from_slice(words[2]).ok()?;
    let y2 = Fq::from_slice(words[3]).ok()?;

    let p1 = if x1 == Fq::zero() && y1 == Fq::zero() {
        G1::zero()
    } else {
        AffineG1::new(x1, y1).ok()?.into()
    };
    let p2 = if x2 == Fq::zero() && y2 == Fq::zero() {
        G1::zero()
    } else {
        AffineG1::new(x2, y2).ok()?.into()
    };

    let mut output = vec![0u8; 64];
    if let Some(ret) = AffineG1::from_jacobian(p1 + p2) {
        ret.x().to_big_endian(&mut output[0..32]).unwrap();
        ret.y().to_big_endian(&mut output[32..64]).unwrap();
        Some(output)
    } else {
        None
    }
}

pub fn ecmul(input: &[u8]) -> Option<Vec<u8>> {
    use bn::{AffineG1, Fq, Fr, Group, G1};

    let words: Vec<_> = input.chunks(32).collect();

    let x1 = Fq::from_slice(words[0]).ok()?;
    let y1 = Fq::from_slice(words[1]).ok()?;

    let fr = Fr::from_slice(words[2]).ok()?;

    let p = if x1 == Fq::zero() && y1 == Fq::zero() {
        G1::zero()
    } else {
        AffineG1::new(x1, y1).ok()?.into()
    };

    let mut output = vec![0u8; 64];
    if let Some(ret) = AffineG1::from_jacobian(p * fr) {
        ret.x().to_big_endian(&mut output[0..32]).unwrap();
        ret.y().to_big_endian(&mut output[32..64]).unwrap();
        Some(output)
    } else {
        None
    }
}

pub fn ecpairing(input: &[u8]) -> Option<Vec<u8>> {
    use bn::{pairing, AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};
    use primitive_types::U256;

    const PAIR_SIZE: usize = 192;

    fn read_one_pair(input: &[u8]) -> Option<(G1, G2)> {
        let words: Vec<_> = input.chunks(32).collect();

        let ax = Fq::from_slice(words[0]).ok()?;
        let ay = Fq::from_slice(words[1]).ok()?;
        let bay = Fq::from_slice(words[2]).ok()?;
        let bax = Fq::from_slice(words[3]).ok()?;
        let bby = Fq::from_slice(words[4]).ok()?;
        let bbx = Fq::from_slice(words[5]).ok()?;

        let ba = Fq2::new(bax, bay);
        let bb = Fq2::new(bbx, bby);

        let b = if ba.is_zero() && bb.is_zero() {
            G2::zero()
        } else {
            AffineG2::new(ba, bb).ok()?.into()
        };
        let a = if ax.is_zero() && ay.is_zero() {
            G1::zero()
        } else {
            AffineG1::new(ax, ay).ok()?.into()
        };

        Some((a, b))
    }

    // input len is not a multiple of PAIR_SIZE
    if input.len() == 0 || input.len() % PAIR_SIZE != 0 {
        return None;
    }

    let mut acc = Gt::one();
    for pair in input.chunks(PAIR_SIZE) {
        let (a, b) = read_one_pair(pair)?;
        acc = acc * pairing(a, b);
    }

    let result = if acc == Gt::one() { U256::one() } else { U256::zero() };

    let mut ret = vec![0u8; 32];
    result.to_big_endian(&mut ret[..]);
    Some(ret)
}
