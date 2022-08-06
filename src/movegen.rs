use piston::index;
use crate::bitboard::{ANTIDIAGS, BITS, DIAGONALS, FILES, LUT_KING, LUT_KNIGHT, LUT_PAWN_CAPTURES, RANKS, RAYS, shift_right};
use crate::Position;

pub fn pseudo_push(pawns: u64, all: u64, forward: i8) -> u64 {
    shift_right(pawns, forward) & !all
}

pub fn pseudo_left_capture(pawns: u64, forward: i8) -> u64 {
    shift_right(pawns, forward - 1)& !FILES[7]
}

pub fn pseudo_right_capture(pawns: u64, forward: i8) -> u64 {
    shift_right(pawns, forward + 1) & !FILES[0]
}

// Pseudo-legal knight moves, uses lookup table
pub fn pseudo_knight(p: u64) -> u64 {
    LUT_KNIGHT[index!(p)]
}

// Pseudo-legal bishop moves, uses hyperbola quintessence
pub fn pseudo_bishop(p: u64, all: u64) -> u64 {
    let i:       usize = index!(p);
    let mask_d:  u64   = DIAGONALS[i];
    let mask_a:  u64   = ANTIDIAGS[i];
    let d:       u64   = all & mask_d;
    let a:       u64   = all & mask_a;
    let di:      u64   = d.swap_bytes();
    let ai:      u64   = a.swap_bytes();
    let pi:      u64   = p.swap_bytes() << 1;
    let moves_d: u64   = d.wrapping_sub(p << 1) ^ di.wrapping_sub(pi).swap_bytes();
    let moves_a: u64   = a.wrapping_sub(p << 1) ^ ai.wrapping_sub(pi).swap_bytes();

    (moves_d & mask_d) | (moves_a & mask_a)
}

// Pseudo-legal rook moves, uses hyperbola quintessence
pub fn pseudo_rook(p: u64, all: u64) -> u64 {
    let i:       usize = index!(p);
    let mask_l:  u64   = FILES[i & 7];
    let mask_r:  u64   = RANKS[i >> 3];
    let l:       u64   = all & mask_l;
    let li:      u64   = l.swap_bytes();
    let pi:      u64   = p.swap_bytes() << 1;
    let msb1_e:  u64   = BITS[0] >> ((p - 1) & all | 1).leading_zeros();
    let moves_l: u64   = l.wrapping_sub(p << 1) ^ li.wrapping_sub(pi).swap_bytes();
    let moves_r: u64   = (all.wrapping_sub(p << 1) ^ all) | (p - msb1_e);

    (moves_l & mask_l) | (moves_r & mask_r)
}

// Pseudo-legal queen moves, combines rook and bishop
pub fn pseudo_queen(p: u64, all: u64) -> u64 {
    pseudo_bishop(p, all) | pseudo_rook(p, all)
}

// Pseudo-legal king moves, uses lookup table
pub fn pseudo_king(p: u64) -> u64 {
    LUT_KING[index!(p)]
}

pub fn get_pinned_bitboard(pos: &Position) -> u64 {
    let mut pinned = 0;

    let king = pos.player & pos.sets[5];
    let king_index = index!(king);

    let diag_snipers = pos.enemy & (pos.sets[2] | pos.sets[4]);
    let line_snipers = pos.enemy & (pos.sets[3] | pos.sets[4]);


    let blockers = pseudo_queen(king, pos.all) & pos.all;
    let cleared = pos.all ^ (blockers & pos.player);
    let mut snipers = ((pseudo_rook(king, cleared) & line_snipers)
                | (pseudo_bishop(king, cleared) & diag_snipers)) & !blockers;

    while snipers != 0 {
        let piece = snipers & 0u64.wrapping_sub(snipers);
        pinned |= blockers & RAYS[king_index][index!(piece)];
        snipers &= snipers - 1;
    }

    pinned
}

pub fn get_attack_bitboard(pos: &Position, stm: bool) -> u64 {
    let mut bb = 0;

    let player = if stm { pos.player } else { pos.enemy };
    let forward = if stm { pos.state.forward } else { -pos.state.forward };
    let all = pos.all & !(pos.sets[5] & (pos.all ^ player));

    let pawns = player & pos.sets[0];
    let mut knights = player & pos.sets[1];
    let mut bishops = player & (pos.sets[2] | pos.sets[4]);
    let mut rooks = player & (pos.sets[3] | pos.sets[4]);

    bb |= pseudo_left_capture(pawns, forward);
    bb |= pseudo_right_capture(pawns, forward);
    bb |= pseudo_king(player & pos.sets[5]);

    while knights != 0 {
        let piece = knights & 0u64.wrapping_sub(knights);
        bb |= pseudo_knight(piece);
        knights &= knights - 1;
    }

    while bishops != 0 {
        let piece = bishops & 0u64.wrapping_sub(bishops);
        bb |= pseudo_bishop(piece, all);
        bishops &= bishops - 1;
    }

    while rooks != 0 {
        let piece = rooks & 0u64.wrapping_sub(rooks);
        bb |= pseudo_rook(piece, all);
        rooks &= rooks - 1;
    }

    bb
}

pub fn get_evasion_mask(pos: &Position) -> u64 {
    let king = pos.player & pos.sets[5];
    let king_index = index!(king);

    let attacking_pawns = LUT_PAWN_CAPTURES[!pos.state.turn as usize][king_index];

    let attacking_knights = pseudo_knight(king) & pos.enemy & pos.sets[1];

    let bishops = pos.enemy & (pos.sets[2] | pos.sets[4]);
    let attacking_bishops = pseudo_bishop(king, pos.all) & bishops;

    let rooks = pos.enemy & (pos.sets[3] | pos.sets[4]);
    let attacking_rooks = pseudo_rook(king, pos.all) & rooks;

    let attackers_mask = attacking_pawns | attacking_knights | attacking_bishops | attacking_rooks;

    return if attackers_mask & (attackers_mask - 1) != 0 {
        0
    } else {
        let attacking_sliders = attacking_bishops | attacking_rooks;
        if attacking_sliders != 0 {
            RAYS[king_index][index!(attacking_sliders)]
        } else {
            attackers_mask
        }
    }
}