use crate::bitboard::BITS;
use crate::position::{BLACK_LONG_CASTLE, BLACK_SHORT_CASTLE, Position, WHITE_LONG_CASTLE, WHITE_SHORT_CASTLE};
use const_random::const_random;

// TODO: GENERATE THE NUMBER FROM SEED INSTEAD

pub const HASH_PIECES: [[u64; 64]; 12] = get_hash_values();
pub const HASH_TURN:               u64 = const_random!(u64);
pub const HASH_WHITE_SHORT_CASTLE: u64 = const_random!(u64);
pub const HASH_WHITE_LONG_CASTLE : u64 = const_random!(u64);
pub const HASH_BLACK_SHORT_CASTLE: u64 = const_random!(u64);
pub const HASH_BLACK_LONG_CASTLE : u64 = const_random!(u64);
pub const HASH_ENPASSANT:     [u64; 8] =
    [const_random!(u64), const_random!(u64), const_random!(u64), const_random!(u64),
        const_random!(u64), const_random!(u64), const_random!(u64), const_random!(u64)];

// Computes the Zobrist key for a position, from scratch
pub fn zobrist_key(pos: &Position) -> u64 {
    let white = if pos.state.turn { pos.player } else { pos.all ^ pos.player };
    let mut hash = 0;
    if pos.state.castle_flags & WHITE_SHORT_CASTLE != 0 { hash ^= HASH_WHITE_SHORT_CASTLE}
    if pos.state.castle_flags & WHITE_LONG_CASTLE  != 0 { hash ^= HASH_WHITE_LONG_CASTLE}
    if pos.state.castle_flags & BLACK_SHORT_CASTLE != 0 { hash ^= HASH_BLACK_SHORT_CASTLE}
    if pos.state.castle_flags & BLACK_LONG_CASTLE  != 0 { hash ^= HASH_BLACK_LONG_CASTLE }
    if !pos.state.turn { hash ^= HASH_TURN }
    if pos.state.en_passant != 0 { hash ^= HASH_ENPASSANT[(pos.state.en_passant.leading_zeros() & 7) as usize]}

    for i in 0..63 {
        let mut tier = 0;
        let bit = BITS[i];
        if bit & pos.all != 0 {
            if      bit & pos.pawns   != 0 { tier = 0 }
            else if bit & pos.knights != 0 { tier = 1 }
            else if bit & pos.bishops != 0 { tier = 2 }
            else if bit & pos.rooks   != 0 { tier = 3 }
            else if bit & pos.queens  != 0 { tier = 4 }
            else if bit & pos.kings   != 0 { tier = 5 }

            if bit & white != 0 { tier += 6 }

            hash ^= HASH_PIECES[tier][i];
        }
    }
    hash
}

// Random integers used to generate Zobrist keys
pub const fn get_hash_values() -> [[u64; 64]; 12] {
    macro_rules! random8 {
        ($arr: expr, $i: expr) => {
            $arr[$i + 0] = const_random!(u64);
            $arr[$i + 1] = const_random!(u64);
            $arr[$i + 2] = const_random!(u64);
            $arr[$i + 3] = const_random!(u64);
            $arr[$i + 4] = const_random!(u64);
            $arr[$i + 5] = const_random!(u64);
            $arr[$i + 6] = const_random!(u64);
            $arr[$i + 7] = const_random!(u64);
        }
    }

    macro_rules! random64 {
        ($arr: expr) => {
            random8!($arr, 0);
            random8!($arr, 8);
            random8!($arr, 16);
            random8!($arr, 24);
            random8!($arr, 32);
            random8!($arr, 40);
            random8!($arr, 48);
            random8!($arr, 56);
        }
    }

    let mut arr0:  [u64; 64] = [0; 64];  random64!(arr0);
    let mut arr1:  [u64; 64] = [0; 64];  random64!(arr1);
    let mut arr2:  [u64; 64] = [0; 64];  random64!(arr2);
    let mut arr3:  [u64; 64] = [0; 64];  random64!(arr3);
    let mut arr4:  [u64; 64] = [0; 64];  random64!(arr4);
    let mut arr5:  [u64; 64] = [0; 64];  random64!(arr5);
    let mut arr6:  [u64; 64] = [0; 64];  random64!(arr6);
    let mut arr7:  [u64; 64] = [0; 64];  random64!(arr7);
    let mut arr8:  [u64; 64] = [0; 64];  random64!(arr8);
    let mut arr9:  [u64; 64] = [0; 64];  random64!(arr9);
    let mut arr10:  [u64; 64] = [0; 64]; random64!(arr10);
    let mut arr11:  [u64; 64] = [0; 64]; random64!(arr11);

    [arr0, arr1, arr2, arr3, arr4, arr5, arr6, arr7, arr8, arr9, arr10, arr11]
}