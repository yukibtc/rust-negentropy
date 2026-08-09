// Copyright (c) 2023 Doug Hoyte
// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use alloc::vec::Vec;
use core::cmp::Ordering;
use core::convert::{TryFrom, TryInto};
use core::ops::Deref;

use crate::encoding::encode_var_int;
use crate::{sha256, Error, Id, FINGERPRINT_SIZE, ID_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Mode {
    Skip = 0,
    Fingerprint = 1,
    IdList = 2,
}

impl Mode {
    #[inline]
    pub(crate) fn as_u64(&self) -> u64 {
        *self as u64
    }
}

impl TryFrom<u64> for Mode {
    type Error = Error;

    fn try_from(mode: u64) -> Result<Self, Self::Error> {
        match mode {
            0 => Ok(Mode::Skip),
            1 => Ok(Mode::Fingerprint),
            2 => Ok(Mode::IdList),
            m => Err(Error::UnexpectedMode(m)),
        }
    }
}

/// Item
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Item {
    /// timestamp
    pub timestamp: u64,
    /// Id
    pub id: Id,
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.timestamp != other.timestamp {
            self.timestamp.cmp(&other.timestamp)
        } else {
            self.id.cmp(&other.id)
        }
    }
}

impl Item {
    /// new Item
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// new Item with just timestamp, id is 0s
    pub fn with_timestamp(timestamp: u64) -> Self {
        let mut item = Self::new();
        item.timestamp = timestamp;
        item
    }

    /// new Item with timestamp and id
    #[inline]
    pub const fn with_timestamp_and_id(timestamp: u64, id: Id) -> Self {
        Self { timestamp, id }
    }

    /// Get id
    #[inline]
    pub const fn get_id(&self) -> &Id {
        &self.id
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Bound
pub struct Bound {
    /// Item
    pub item: Item,
    /// ID Len
    pub id_len: usize,
}

impl Bound {
    /// New Bound
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// new Bound from item
    pub fn from_item(item: &Item) -> Self {
        let mut bound = Self::new();
        bound.item = *item;
        bound.id_len = ID_SIZE;
        bound
    }

    /// new Bound from timestamp, id len is 0
    pub fn with_timestamp(timestamp: u64) -> Self {
        let mut bound = Self::new();
        bound.item.timestamp = timestamp;
        bound.id_len = 0;
        bound
    }

    /// New Bound from timestamp and id
    pub fn with_timestamp_and_id<T>(timestamp: u64, id: T) -> Result<Self, Error>
    where
        T: AsRef<[u8]>,
    {
        let id: &[u8] = id.as_ref();
        let len: usize = id.len();

        if len > ID_SIZE {
            return Err(Error::IdTooBig);
        }

        let mut out = Bound::new();
        out.item.timestamp = timestamp;
        out.item.id[..len].copy_from_slice(id);
        out.id_len = len;

        Ok(out)
    }
}

impl PartialOrd for Bound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bound {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item.cmp(&other.item)
    }
}

/// Fingerprint
#[derive(Debug, Clone, Copy, Default)]
pub struct Fingerprint {
    /// Buffer
    buf: [u8; FINGERPRINT_SIZE],
}

impl Deref for Fingerprint {
    type Target = [u8; FINGERPRINT_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl Fingerprint {
    #[inline]
    pub fn to_bytes(self) -> [u8; FINGERPRINT_SIZE] {
        self.buf
    }
}

/// Accumulator
#[derive(Debug, Clone, Copy, Default)]
pub struct Accumulator {
    buf: [u8; ID_SIZE],
}

impl Accumulator {
    /// New Accumulator
    #[inline]
    pub fn new() -> Self {
        Self { buf: [0; ID_SIZE] }
    }

    /// Add
    pub fn add(&mut self, buf: &[u8; ID_SIZE]) -> Result<(), Error> {
        // 256-bit little-endian add, four 64-bit lanes with carry. Both inputs
        // are fixed-size arrays, so the lane splits cannot fail, and the result
        // is written straight back rather than through a scratch copy.
        let mut carry: u64 = 0;

        for i in 0..4 {
            let lo: usize = i * 8;
            let hi: usize = lo + 8;

            let a: u64 = u64::from_le_bytes(self.buf[lo..hi].try_into()?);
            let b: u64 = u64::from_le_bytes(buf[lo..hi].try_into()?);

            let (sum, c1) = a.overflowing_add(b);
            let (sum, c2) = sum.overflowing_add(carry);

            self.buf[lo..hi].copy_from_slice(&sum.to_le_bytes());
            carry = (c1 | c2) as u64;
        }

        Ok(())
    }

    /// Compute fingerprint, given set size
    pub fn get_fingerprint(&self, n: u64) -> Result<Fingerprint, Error> {
        let var_int: Vec<u8> = encode_var_int(n);

        let mut input: Vec<u8> = Vec::with_capacity(ID_SIZE + var_int.len());
        input.extend(&self.buf);
        input.extend(var_int);

        let hash: [u8; 32] = sha256::hash(input);

        Ok(Fingerprint {
            buf: hash[0..FINGERPRINT_SIZE].try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A carry leaving one 64-bit lane must land in the next one.
    #[test]
    fn test_accumulator_carry_between_lanes() {
        for lane in 0..3 {
            let mut acc = Accumulator::new();

            let mut full_lane = [0u8; ID_SIZE];
            full_lane[lane * 8..(lane + 1) * 8].copy_from_slice(&u64::MAX.to_le_bytes());
            acc.add(&full_lane).unwrap();

            let mut one = [0u8; ID_SIZE];
            one[lane * 8] = 1;
            acc.add(&one).unwrap();

            let mut expected = [0u8; ID_SIZE];
            expected[(lane + 1) * 8] = 1;
            assert_eq!(acc.buf, expected, "carry out of lane {}", lane);
        }
    }

    /// An incoming carry can overflow a lane on its own, when that lane's two
    /// operands already sum to `u64::MAX`. Both overflow sources must be
    /// propagated, not just the one from adding the operands.
    #[test]
    fn test_accumulator_carry_propagates_through_full_lane() {
        let mut acc = Accumulator::new();

        let mut lanes = [0u8; ID_SIZE];
        lanes[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        lanes[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        acc.add(&lanes).unwrap();

        let mut one = [0u8; ID_SIZE];
        one[0] = 1;
        acc.add(&one).unwrap();

        // Lane 0 wraps to zero, its carry wraps lane 1 to zero in turn, and
        // that second carry reaches lane 2.
        let mut expected = [0u8; ID_SIZE];
        expected[16] = 1;
        assert_eq!(acc.buf, expected);
    }

    /// A carry out of the top lane is discarded: the sum is modulo 2^256.
    #[test]
    fn test_accumulator_wraps_at_256_bits() {
        let mut acc = Accumulator::new();
        acc.add(&[0xFF; ID_SIZE]).unwrap();

        let mut one = [0u8; ID_SIZE];
        one[0] = 1;
        acc.add(&one).unwrap();

        assert_eq!(acc.buf, [0u8; ID_SIZE]);
    }

    /// Range fingerprints are computed by summing ids in storage order, while
    /// the peer may sum the same ids in a different order, so the accumulator
    /// must not depend on the order it is fed.
    #[test]
    fn test_accumulator_is_order_independent() {
        let ids: [[u8; ID_SIZE]; 4] = [
            [0x11; ID_SIZE],
            [0xFF; ID_SIZE],
            [0x00; ID_SIZE],
            [0xA5; ID_SIZE],
        ];

        let mut forward = Accumulator::new();
        for id in ids.iter() {
            forward.add(id).unwrap();
        }

        let mut backward = Accumulator::new();
        for id in ids.iter().rev() {
            backward.add(id).unwrap();
        }

        assert_eq!(forward.buf, backward.buf);
    }
}
