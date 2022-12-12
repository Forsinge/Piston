use crate::position::Move;

pub const TT_DEFAULT_SIZE: u64 = 2097152;
pub const TT_DEFAULT_INDEX_BITS: u32 = 21;
pub const BUCKET_SIZE: usize = 4;
pub const BUCKET_MASK: u64 = !((BUCKET_SIZE as u64)-1);

// Transposition table entry
#[derive(Copy, Clone, Default)]
pub struct TTEntry {
    pub data:       u128,    // Order: REFUTATION - DEPTH - AGE - OUTCOME - EVAL - KEY
                             // Bits:      20         5      2       2       16    64

                             // Outcome: 0 = all-node, 1 = pv_node, 2 = cut-node
}

impl TTEntry {
    pub fn get_key(&self) -> u64 {(self.data & 0xFFFFFFFFFFFFFFFF) as u64}

    pub fn get_eval(&self) -> i16 {
        ((self.data >> 64) & 0xFFFF) as i16
    }

    pub fn get_outcome(&self) -> u8 { ((self.data >> 80) & 0x3) as u8 }

    pub fn get_age(&self) -> u8 {
        ((self.data >> 82) & 0x3) as u8
    }

    pub fn get_depth(&self) -> u8 { ((self.data >> 84) & 0x1F) as u8 }

    pub fn get_refutation(&self) -> Move { Move::from_u32(((self.data >> 89) & 0xFFFFF) as u32) }
}

pub fn create_entry(key: u64, eval: i16, outcome: u8, age: u8, depth: u8, refutation: u32) -> TTEntry {
    let mut data = key as u128;                                          // 64 BITS, TOTAL = 64
    data |= (eval as u128 & 0xFFFF) << 64;                                     // 16 BITS, TOTAL = 80
    data |= (outcome as u128 & 0x3) << 80;                                     // 2  BITS, TOTAL = 82
    data |= (age as u128 & 0x3) << 82;                                         // 2  BITS, TOTAL = 84
    data |= (depth as u128 & 0x1F) << 84;                                      // 5  BITS, TOTAL = 89
    data |= (refutation as u128 & 0xFFFFF) << 89;                              // 20 BITS, TOTAL = 109
    TTEntry {data}
}

pub struct TT {
    pub table: Vec<TTEntry>,
    pub mask:  u64,
}

impl TT {
    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let index = (key & self.mask & BUCKET_MASK) as usize;

        for i in index..index+BUCKET_SIZE {
            let stored = self.table[i];
            if stored.get_key() == key {
                return Some(stored);
            }
        }

        return None;
    }

    pub fn place(&mut self, _root_key: u64, root_age: u8, key: u64, eval: i16, outcome: u8, depth: u8, refutation: u32) {
        let index = (key & self.mask & BUCKET_MASK) as usize;

        let mut lowest = u8::MAX;
        let mut li = index;
        for i in index..index+BUCKET_SIZE {
            let depth = self.table[i].get_depth();
            let recency = ((self.table[i].get_age() == root_age) as u8) << 1;
            let value = depth + recency;
            if value < lowest {
                lowest = value;
                li = i;
            }
        }

        let entry = create_entry(key, eval, outcome, root_age, depth, refutation);
        self.table[li] = entry;
    }

    pub fn reset(&mut self) {
        for i in 0..self.table.len() {
            self.table[i] = TTEntry::default();
        }
    }

    pub fn hashfull(&self) -> u64 {
        let mut counter: u64 = 0;
        for i in 0..self.table.len() {
            if self.table[i].data != 0 { counter += 1 }
        }
        counter / (TT_DEFAULT_SIZE / 1000)
    }
}

pub fn create_tt (size: u64) -> TT {
    assert!(size.is_power_of_two());
    let mask: u64 = size - 1;
    let table = vec![TTEntry::default(); size as usize];
    TT {mask, table}
}