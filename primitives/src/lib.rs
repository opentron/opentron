use hex::FromHex;
use std::cmp;
use std::fmt;
use std::hash;
use std::io;
use std::ops;
use std::slice;
use std::str::FromStr;

macro_rules! impl_ops_for_hash {
    (
        $impl_for:ident,
        $ops_trait_name:ident,
        $ops_fn_name:ident,
        $ops_assign_trait_name:ident,
        $ops_assign_fn_name:ident,
        $ops_tok:tt,
        $ops_assign_tok:tt
    ) => {
        impl<'r> ops::$ops_assign_trait_name<&'r $impl_for> for $impl_for {
            fn $ops_assign_fn_name(&mut self, rhs: &'r $impl_for) {
                for (lhs, rhs) in self.as_bytes_mut().iter_mut().zip(rhs.as_bytes()) {
                    *lhs $ops_assign_tok rhs;
                }
            }
        }

        impl ops::$ops_assign_trait_name<$impl_for> for $impl_for {
            #[inline]
            fn $ops_assign_fn_name(&mut self, rhs: $impl_for) {
                *self $ops_assign_tok &rhs;
            }
        }

        impl<'l, 'r> ops::$ops_trait_name<&'r $impl_for> for &'l $impl_for {
            type Output = $impl_for;

            fn $ops_fn_name(self, rhs: &'r $impl_for) -> Self::Output {
                let mut ret = self.clone();
                ret $ops_assign_tok rhs;
                ret
            }
        }

        impl ops::$ops_trait_name<$impl_for> for $impl_for {
            type Output = $impl_for;

            #[inline]
            fn $ops_fn_name(self, rhs: Self) -> Self::Output {
                &self $ops_tok &rhs
            }
        }
    };
}

macro_rules! impl_cmp_for_fixed_hash {
    ( $name:ident ) => {
        impl cmp::PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.as_bytes() == other.as_bytes()
            }
        }

        impl cmp::Ord for $name {
            #[inline]
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                self.as_bytes().cmp(other.as_bytes())
            }
        }
    };
}

#[derive(Clone, Copy, Eq, Default)]
pub struct H256(pub [u8; 32]);

impl From<[u8; 32]> for H256 {
    /// Constructs a hash type from the given bytes array of fixed length.
    ///
    /// # Note
    ///
    /// The given bytes are interpreted in big endian order.
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        H256(bytes)
    }
}

impl<'a> From<&'a [u8; 32]> for H256 {
    /// Constructs a hash type from the given reference
    /// to the bytes array of fixed length.
    ///
    /// # Note
    ///
    /// The given bytes are interpreted in big endian order.
    #[inline]
    fn from(bytes: &'a [u8; 32]) -> Self {
        H256(*bytes)
    }
}

impl<'a> From<&'a mut [u8; 32]> for H256 {
    /// Constructs a hash type from the given reference
    /// to the mutable bytes array of fixed length.
    ///
    /// # Note
    ///
    /// The given bytes are interpreted in big endian order.
    #[inline]
    fn from(bytes: &'a mut [u8; 32]) -> Self {
        H256(*bytes)
    }
}

impl FromStr for H256 {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        if s.len() == 64 {
            <[u8; 32]>::from_hex(s)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "invalid hex format"))
                .map(H256::from)
        } else if s.len() == 66 && (s.starts_with("0x") || s.starts_with("0X")) {
            s[2..].parse()
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "invalid hex length"))
        }
    }
}

impl From<H256> for [u8; 32] {
    #[inline]
    fn from(s: H256) -> Self {
        s.0
    }
}

impl AsRef<[u8]> for H256 {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for H256 {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

impl H256 {
    /// Returns a new fixed hash where all bits are set to the given byte.
    #[inline]
    pub const fn repeat_byte(byte: u8) -> H256 {
        H256([byte; 32])
    }

    /// Returns a new zero-initialized fixed hash.
    #[inline]
    pub const fn zero() -> H256 {
        H256::repeat_byte(0u8)
    }

    /// Returns the size of this hash in bytes.
    #[inline]
    pub const fn len_bytes() -> usize {
        32
    }

    /// Extracts a byte slice containing the entire fixed hash.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Extracts a mutable byte slice containing the entire fixed hash.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    /// Extracts a reference to the byte array containing the entire fixed hash.
    #[inline]
    pub const fn as_fixed_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Extracts a reference to the byte array containing the entire fixed hash.
    #[inline]
    pub fn as_fixed_bytes_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }

    /// Returns the inner bytes array.
    #[inline]
    pub const fn to_fixed_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Returns a constant raw pointer to the value.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.as_bytes().as_ptr()
    }

    /// Returns a mutable raw pointer to the value.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.as_bytes_mut().as_mut_ptr()
    }

    /// Assign the bytes from the byte slice `src` to `self`.
    ///
    /// # Note
    ///
    /// The given bytes are interpreted in big endian order.
    ///
    /// # Panics
    ///
    /// If the length of `src` and the number of bytes in `self` do not match.
    pub fn assign_from_slice(&mut self, src: &[u8]) {
        assert_eq!(src.len(), 32);
        self.as_bytes_mut().copy_from_slice(src);
    }

    /// Create a new fixed-hash from the given slice `src`.
    ///
    /// # Note
    ///
    /// The given bytes are interpreted in big endian order.
    ///
    /// # Panics
    ///
    /// If the length of `src` and the number of bytes in `Self` do not match.
    pub fn from_slice(src: &[u8]) -> Self {
        assert_eq!(src.len(), 32);
        let mut ret = Self::zero();
        ret.assign_from_slice(src);
        ret
    }

    /// Returns `true` if all bits set in `b` are also set in `self`.
    #[inline]
    pub fn covers(&self, b: &Self) -> bool {
        &(b & self) == b
    }

    /// Returns `true` if no bits are set.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.as_bytes().iter().all(|&byte| byte == 0u8)
    }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::Display for H256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0[0..2] {
            write!(f, "{:02x}", i)?;
        }
        write!(f, "…")?;
        for i in &self.0[32 - 2..32] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl fmt::LowerHex for H256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for i in &self.0[..] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl fmt::UpperHex for H256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "0X")?;
        }
        for i in &self.0[..] {
            write!(f, "{:02X}", i)?;
        }
        Ok(())
    }
}

impl cmp::PartialOrd for H256 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl hash::Hash for H256 {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        state.write(&self.0);
        state.finish();
    }
}

impl<I> ops::Index<I> for H256
where
    I: slice::SliceIndex<[u8]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &I::Output {
        &self.as_bytes()[index]
    }
}

impl<I> ops::IndexMut<I> for H256
where
    I: slice::SliceIndex<[u8], Output = [u8]>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        &mut self.as_bytes_mut()[index]
    }
}

impl_ops_for_hash!(H256, BitOr, bitor, BitOrAssign, bitor_assign, |, |=);
impl_ops_for_hash!(H256, BitAnd, bitand, BitAndAssign, bitand_assign, &, &=);
impl_ops_for_hash!(H256, BitXor, bitxor, BitXorAssign, bitxor_assign, ^, ^=);

impl_cmp_for_fixed_hash!(H256);

/*
impl_byteorder_for_fixed_hash!(H256);
impl_rand_for_fixed_hash!(H256);
impl_rustc_hex_for_fixed_hash!(H256);
impl_quickcheck_for_fixed_hash!(H256);
*/
