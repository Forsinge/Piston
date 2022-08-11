use crate::bitboard::{ANTIDIAGS, DIAGONALS, FILES, LUT_KING, LUT_KNIGHT, LUT_RANK_SLIDE};

pub fn pseudo_push(pawns: u64, all: u64, shift_offset: u8) -> u64 {
    ((pawns << 8) >> shift_offset) & !all
}
pub fn pseudo_left_capture(pawns: u64, shift_offset: u8) -> u64 {
    ((pawns << 9) >> shift_offset) & !FILES[7]
}

pub fn pseudo_right_capture(pawns: u64, shift_offset: u8) -> u64 {
    ((pawns << 7) >> shift_offset) & !FILES[0]
}

// Pseudo-legal knight moves, uses lookup table
pub fn pseudo_knight(i: usize) -> u64 {
    LUT_KNIGHT[i]
}

// Pseudo-legal bishop moves, uses hyperbola quintessence
pub fn pseudo_bishop(p: u64, all: u64, i: usize) -> u64 {
    let mask_d:  u64   = DIAGONALS[i];
    let mask_a:  u64   = ANTIDIAGS[i];
    let d:       u64   = all & mask_d;
    let a:       u64   = all & mask_a;
    let pi:      u64   = p.swap_bytes() << 1;
    let moves_d: u64   = d.wrapping_sub(p << 1) ^ d.swap_bytes().wrapping_sub(pi).swap_bytes();
    let moves_a: u64   = a.wrapping_sub(p << 1) ^ a.swap_bytes().wrapping_sub(pi).swap_bytes();

    (moves_d & mask_d) | (moves_a & mask_a)
}

// Pseudo-legal rook moves, uses hyperbola quintessence + rank lookup
pub fn pseudo_rook(p: u64, all: u64, i: usize) -> u64 {
    let file:       usize = i & 7;
    let mask_l:     u64   = FILES[file];
    let rank_shift: usize = 56 ^ i & 56;
    let l:          u64   = all & mask_l;
    let moves_l:    u64   = l.wrapping_sub(p << 1) ^ (l.swap_bytes()).wrapping_sub(p.swap_bytes() << 1).swap_bytes();
    let moves_r:    u64   = LUT_RANK_SLIDE[file][((all >> rank_shift) & 0xFF) as usize];

    (moves_l & mask_l) | (moves_r << rank_shift)
}

// Pseudo-legal queen moves, combines rook and bishop
pub fn pseudo_queen(p: u64, all: u64, i: usize) -> u64 {
    pseudo_bishop(p, all, i) | pseudo_rook(p, all, i)
}

// Pseudo-legal king moves, uses lookup table
pub fn pseudo_king(i: usize) -> u64 {
    LUT_KING[i]
}

pub fn pseudo_rank(i: usize, all: u64) -> u64 {
    let rank_shift: usize = 56 ^ i & 56;
    LUT_RANK_SLIDE[i & 7][((all >> rank_shift) & 0xFF) as usize] << rank_shift
}