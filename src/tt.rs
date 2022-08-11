use crate::state::EngineState;

pub const TT_DEFAULT_SIZE: u64 = 2097152;
pub const TT_DEFAULT_INDEX_BITS: u32 = 21;

pub fn get_key_high(key: u64) -> u64 {
    (key >> TT_DEFAULT_INDEX_BITS) & 0xFFFFFFFFFF
}

// Transposition table entry
#[derive(Copy, Clone, Default)]
pub struct TTEntry {
    pub data:       u128,    // Order: REFUTATION - DEPTH - AGE - OUTCOME - EVAL - KEY
                             // Bits:       24         5      2       2       16    40

                             // Outcome: 0 = all-node, 1 = pv_node, 2 = cut-node
}

impl TTEntry {
    pub fn get_key(&self) -> u64 {(self.data & 0xFFFFFFFFFF) as u64}

    pub fn get_eval(&self) -> i16 {
        ((self.data >> 36) & 0xFFFF) as i16
    }

    pub fn get_outcome(&self) -> u8 { ((self.data >> 52) & 0x3) as u8 }

    pub fn get_age(&self) -> u8 {
        ((self.data >> 54) & 0x3) as u8
    }

    pub fn get_depth(&self) -> u8 { ((self.data >> 56) & 0x1F) as u8 }

    pub fn get_refutation(&self) -> u32 { ((self.data >> 61) & 0xFFFFFF) as u32 }
}


pub fn create_entry(key: u64, eval: i16, outcome: u8, age: u8, depth: u8, refutation: u32) -> TTEntry {
    let mut data = (key as u128 >> TT_DEFAULT_INDEX_BITS) & 0xFFFFFFFFF; // 40 BITS, TOTAL = 40
    data |= (eval as u128 & 0xFFFF) << 40;                                     // 16 BITS, TOTAL = 56
    data |= (outcome as u128 & 0x3) << 56;                                     // 2  BITS, TOTAL = 58
    data |= (age as u128 & 0x3) << 58;                                         // 2  BITS, TOTAL = 60
    data |= (depth as u128 & 0x1F) << 60;                                      // 5  BITS, TOTAL = 65
    data |= (refutation as u128 & 0xFFFFFF) << 65;                             // 24 BITS, TOTAL = 81
    TTEntry {data}
}

pub struct TT {
    pub table: Vec<TTEntry>,
    pub mask:  u64,
}

impl TT {
    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let key_high = get_key_high(key);
        let index = (key & self.mask) as usize;
        let stored = self.table[index];

        if stored.get_key() == key_high {
            return Some(stored);
        }

        return None;
    }

    pub unsafe fn place(&mut self, es: &EngineState, key: u64, eval: i16, outcome: u8, depth: u8, refutation: u32) {
        let root_key_high = get_key_high(es.root_key);
        let key_high = get_key_high(key);
        let index = (key & self.mask) as usize;
        let stored = self.table[index];

        if stored.get_key() != root_key_high || key_high == root_key_high {
            let entry = create_entry(key, eval, outcome, es.root_age, depth, refutation);
            self.table[index] = entry;
        }
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