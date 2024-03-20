#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct BitSet {
    bitmask: [u64; 4],
}

impl BitSet {
    pub const INVALID: BitSet = BitSet {
        bitmask: [u64::MAX, u64::MAX, u64::MAX, u64::MAX],
    };

    const EMPTY: BitSet = BitSet {
        bitmask: [u64::MIN, u64::MIN, u64::MIN, u64::MIN],
    };

    pub fn new() -> Self {
        Self { bitmask: [0u64; 4] }
    }

    pub fn insert_id(&mut self, id: u8) -> &mut Self {
        let index = id / 64;
        let position = id - index * 64;
        self.bitmask[index as usize] = self.bitmask[index as usize] | 1 << position;

        self
    }

    pub fn remove_id(&mut self, id: u8) -> &mut Self {
        let index = id / 64;
        let position = id - index * 64;
        let bit_to_remove = 1 << position;
        self.bitmask[index as usize] = !(self.bitmask[index as usize] ^ bit_to_remove);

        self
    }

    pub fn from_id(id: u8) -> Self {
        let mut bitmask = [0u64; 4];
        let index = id / 64;
        let position = id - index * 64;
        bitmask[index as usize] = 1 << position;

        Self { bitmask }
    }

    #[inline(always)]
    pub fn contains(&self, other: &BitSet) -> bool {
        let mut result = true;

        for (a, b) in self.bitmask.iter().zip(other.bitmask.iter()) {
            result &= a & b == *b
        }

        result
    }

    #[inline(always)]
    pub fn contains_id(&self, id: u8) -> bool {
        let index = id / 64;
        let position = id - index * 64;

        self.bitmask[index as usize] & 1 << position > 0
    }
}
