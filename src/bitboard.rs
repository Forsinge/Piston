/**************************|
|*         UTILS          *|
|**************************/

pub const RANKS: [u64; 8] = [
    0xFF00000000000000,
    0x00FF000000000000,
    0x0000FF0000000000,
    0x000000FF00000000,
    0x00000000FF000000,
    0x0000000000FF0000,
    0x000000000000FF00,
    0x00000000000000FF];

pub const FILES: [u64; 8] = [
    0x8080808080808080,
    0x4040404040404040,
    0x2020202020202020,
    0x1010101010101010,
    0x0808080808080808,
    0x0404040404040404,
    0x0202020202020202,
    0x0101010101010101];

pub const DIAGONALS_TEMPLATE: [u64; 15] = [
    0x0000000000000080,
    0x0000000000008040,
    0x0000000000804020,
    0x0000000080402010,
    0x0000008040201008,
    0x0000804020100804,
    0x0080402010080402,
    0x8040201008040201,
    0x4020100804020100,
    0x2010080402010000,
    0x1008040201000000,
    0x0804020100000000,
    0x0402010000000000,
    0x0201000000000000,
    0x0100000000000000];

pub const ANTIDIAGS_TEMPLATE: [u64; 15] = [
    0x8000000000000000,
    0x4080000000000000,
    0x2040800000000000,
    0x1020408000000000,
    0x0810204080000000,
    0x0408102040800000,
    0x0204081020408000,
    0x0102040810204080,
    0x0001020408102040,
    0x0000010204081020,
    0x0000000102040810,
    0x0000000001020408,
    0x0000000000010204,
    0x0000000000000102,
    0x0000000000000001];


pub const CENTER: u64 = 0x0000001818000000;
pub const WHITE_OFFENSIVE_ZONE: u64 = 0xFFFFFFFFFF000000;
pub const BLACK_OFFENSIVE_ZONE: u64 = 0x000000FFFFFFFFFF;
pub const EDGES: u64 = 0xFF818181818181FF;

pub const fn shift_left(bb: u64, val: i8) -> u64 {
    if val > 0 {bb << val} else {bb >> -val}
}

pub const fn shift_right(bb: u64, val: i8) -> u64 {
    if val > 0 {bb >> val} else {bb << -val}
}

pub const fn shift_forward(bb: u64, val: u8, color: bool) -> u64 {
    if color {bb >> val} else {bb << val}
}

pub const fn shift_backward(bb: u64, val: u8, color: bool) -> u64 {
    if color {bb << val} else {bb >> val}
}


/**************************|
|*     LOOKUP TABLES      *|
|**************************/

pub const BITS:              [u64; 64]       = get_bit_lut();
pub const DIAGONALS:         [u64; 64]       = get_diagonals();
pub const ANTIDIAGS:         [u64; 64]       = get_antidiags();
pub const RAYS:              [[u64; 64]; 64] = get_rays_lut();
pub const LUT_KNIGHT:        [u64; 64]       = get_lut_knight();
pub const LUT_KING:          [u64; 64]       = get_lut_king();
pub const LUT_PAWN_CAPTURES: [[u64; 64]; 2]  = get_lut_pawn_captures();
pub const LUT_BISHOP:        [u64; 64]       = get_lut_bishop();
pub const LUT_ROOK:          [u64; 64]       = get_lut_rook();
pub const LUT_RANK_SLIDE:    [[u64; 256]; 8]  = get_lut_rank_slide();

// LUT for single bits
const fn get_bit_lut() -> [u64; 64] {
    let mut arr:  [u64; 64] = [0; 64];
    let high_bit: u64       = 0x8000000000000000;
    let mut i:    usize     = 0;

    loop {
        arr[i] = high_bit >> i;

        i += 1;
        if i == 64 { break }
    }
    arr
}

// LUT for diagonals, A1-H8
const fn get_diagonals() -> [u64; 64] {
    let mut arr: [u64; 64] = [0; 64];
    let mut i:   usize     = 0;

    loop {
        arr[i] = DIAGONALS_TEMPLATE[((56 ^ i & 56) >> 3) + (i & 7)];

        i += 1;
        if i == 64 { break }
    }
    arr
}

// LUT for antidiagonals, A8-H1
const fn get_antidiags() -> [u64; 64] {
    let mut arr: [u64; 64] = [0; 64];
    let mut i:   usize     = 0;

    loop {
        arr[i] = ANTIDIAGS_TEMPLATE[((i & 56) >> 3) + (i & 7)];

        i += 1;
        if i == 64 { break }
    }
    arr
}


// LUT for rays, origin bit not set
pub const fn get_rays_lut() -> [[u64; 64]; 64]{
    let mut arr: [[u64; 64]; 64] = [[0; 64]; 64];
    let mut i:   usize           = 0;

    loop {
        let origin = BITS[i];
        let mut j: usize = 0;

        loop {
            let target = BITS[j];

            let mut ray = 0;
            let mut bit = origin;

            let direction =
                if target & RANKS[i >> 3] != 0 { 1 }
                else if target & ANTIDIAGS[i]  != 0 { 7 }
                else if target & FILES[i & 7]  != 0 { 8 }
                else if target & DIAGONALS[i]  != 0 { 9 }
                else                                { 0 };

            let mut k = 0;
            loop {
                if target > origin {
                    bit <<= direction
                } else {
                    bit >>= direction
                }

                ray |= bit;

                if bit == target {
                    arr[i][j] = ray;
                    break;
                }

                k += 1;
                if k == 8 { break }
            }

            j += 1;
            if j == 64 { break }
        }

        i += 1;
        if i == 64 { break }
    }
    arr
}

const fn get_lut_bishop() -> [u64; 64] {
    let mut arr: [u64; 64] = [0; 64];
    let mut i:   usize     = 0;

    loop {
        arr[i] |= DIAGONALS_TEMPLATE[((56 ^ i & 56) >> 3) + (i & 7)];
        arr[i] |= ANTIDIAGS_TEMPLATE[((i & 56) >> 3) + (i & 7)];
        i += 1;
        if i == 64 { break }
    }
    arr
}

const fn get_lut_rook() -> [u64; 64] {
    let mut arr: [u64; 64] = [0; 64];
    let mut i:   usize     = 0;

    loop {
        arr[i] |= FILES[i & 7];
        arr[i] |= RANKS[i >> 3];
        i += 1;
        if i == 64 { break }
    }
    arr
}

// LUT for knight attack bitboards
const fn get_lut_knight() -> [u64; 64] {
    let mask:        u64       = 0xA1100110A;
    let clear_right: u64       = 0xFCFCFCFCFCFCFCFC;
    let clear_left:  u64       = 0x3F3F3F3F3F3F3F3F;
    let mut lut:     [u64; 64] = [0; 64];
    let mut i:       usize     = 0;

    loop {
        let mut bb = shift_left(mask, 45 - (i as i8));

        if i % 8 <= 1 {
            bb = bb & clear_right
        }

        else if i % 8 >= 6 {
            bb = bb & clear_left
        }

        lut[i] = bb;

        i += 1;
        if i == 64 { break }
    }
    lut
}

// LUT for king attack bitboards
const fn get_lut_king() -> [u64; 64] {
    let mask:        u64       = 0x70507;
    let clear_right: u64       = 0xF0F0F0F0F0F0F0F0;
    let clear_left:  u64       = 0x0F0F0F0F0F0F0F0F;
    let mut lut:     [u64; 64] = [0; 64];
    let mut i:       usize     = 0;

    loop {
        let mut bb = shift_left(mask, 54 - (i as i8));

        if i % 8 <= 1 {
            bb = bb & clear_right
        }

        if i % 8 >= 6 {
            bb = bb & clear_left
        }

        lut[i] = bb;

        i += 1;
        if i == 64 { break }
    }
    lut
}

// LUT for pawn capture bitboards, indexed by side to move
const fn get_lut_pawn_captures() -> [[u64; 64]; 2] {
    let mut lut:     [[u64; 64]; 2] = [[0; 64]; 2];
    let clear_left:  u64            = 0x7F7F7F7F7F7F7F7F;
    let clear_right: u64            = 0xFEFEFEFEFEFEFEFE;
    let mut bit:     u64            = 0x8000000000000000;
    let mut i:       usize          = 0;

    loop {
        let mut bb1 = (bit >> 7) | (bit >> 9);
        let mut bb2 = (bit << 7) | (bit << 9);

        if i & 7 == 7 {
            bb1 &= clear_left;
            bb2 &= clear_left;
        }

        if i & 7 == 0 {
            bb1 &= clear_right;
            bb2 &= clear_right;
        }

        lut[0][i] = bb1;
        lut[1][i] = bb2;

        bit >>= 1;
        i += 1;
        if i == 64 { break }
    }

    lut
}

const fn get_lut_rank_slide() -> [[u64; 256]; 8] {
    let mut lut: [[u64; 256]; 8] = [[0; 256]; 8];

    let mut r = 0;
    loop {
        let mut i: u64 = 0;
        loop {
            let p = BITS[56+r];
            let msb1  = BITS[0] >> ((p - 1) & i | 1).leading_zeros();
            let slide   = ((i.wrapping_sub(p << 1) ^ i) | (p - msb1)) & 0xFF;

            lut[r as usize][i as usize] = slide;

            i += 1;
            if i == 256 {
                break
            }
        }

        r += 1;
        if r == 8 {
            break
        }
    }
    lut
}
