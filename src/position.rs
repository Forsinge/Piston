use piston::{index, indexu8};
use crate::context::{MAX_MOVE_COUNT, SearchContext};
use crate::bitboard::{BITS, RANKS, shift_left, shift_right};
use crate::{Display, get_attack_bitboard};
use crate::movegen::{get_evasion_mask, get_pinned_bitboard, pseudo_bishop, pseudo_king, pseudo_knight, pseudo_left_capture, pseudo_push, pseudo_queen, pseudo_right_capture, pseudo_rook};
use crate::output::string_to_index;


pub const WHITE_SHORT_CASTLE: u8 = 1;
pub const WHITE_LONG_CASTLE: u8 = 2;
pub const BLACK_SHORT_CASTLE: u8 = 4;
pub const BLACK_LONG_CASTLE: u8 = 8;

pub const WHITE_CASTLES: u8 = 3;
pub const BLACK_CASTLES: u8 = 12;

pub const FILE_CHARS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

#[derive(Default, Copy, Clone)]
pub struct Move {
    pub origin: u8,
    pub target: u8,
    pub tier: u8,
    pub code: u8,
}

impl Move {
    pub fn origin_bb(&self) -> u64 {
        BITS[self.origin as usize]
    }

    pub fn target_bb(&self) -> u64 {
        BITS[self.target as usize]
    }

    pub fn tier(&self) -> usize {
        self.tier as usize
    }
}

#[derive(Default)]
pub struct PositionState {
    pub en_passant: u64,
    pub forward: i8,
    pub castle_flags: u8,
    pub move_ptr: usize,
    pub move_cnt: usize,
    pub half_move: usize,
    pub turn: bool,
}

impl PositionState {
    pub fn next(&self) -> PositionState {
        PositionState {
            en_passant: 0,
            forward: -self.forward,
            castle_flags: self.castle_flags,
            move_ptr: self.move_ptr + MAX_MOVE_COUNT,
            move_cnt: 0,
            half_move: self.half_move + 1,
            turn: !self.turn,
        }
    }
}

#[derive(Default)]
pub struct Position {
    pub sets: [u64; 6],
    pub all: u64,
    pub player: u64,
    pub enemy: u64,
    pub state: PositionState,
}

impl Position {
    pub fn build_from_fen(fen: &str) -> Position {
        let fen_split = fen.split(' ').collect::<Vec<&str>>();
        let mut pos = Position::default();

        if fen_split.len() < 4 {
            println!("Incomplete FEN-string");
            return pos;
        }

        let pieces = "PNBRQKpnbrqk";
        let mut pointer = BITS[0];
        for c in fen_split[0].replace('/', "").chars() {

            if c.is_alphabetic() {
                let bit = pointer.swap_bytes();
                let tier = pieces.find(c).unwrap() % 6;

                pos.sets[tier] |= bit;
                pos.all |= bit;

                if c.is_uppercase() {
                    pos.player |= bit;
                }

                pointer >>= 1;
            }

            else {
                pointer >>= c.to_digit(10).unwrap();
            }
        }

        if fen_split[1] == "w" {
            pos.state.turn = true;
            pos.enemy = pos.player ^ pos.all;
            pos.state.forward = 8;
        }

        else {
            pos.enemy = pos.player;
            pos.player ^= pos.all;
            pos.state.forward = -8;
        }

        for c in fen_split[2].chars() {
            match c {
                'K' => pos.state.castle_flags |= WHITE_SHORT_CASTLE,
                'Q' => pos.state.castle_flags |= WHITE_LONG_CASTLE,
                'k' => pos.state.castle_flags |= BLACK_SHORT_CASTLE,
                'q' => pos.state.castle_flags |= BLACK_LONG_CASTLE,
                _ => {}
            }
        }

        if fen_split[3] != "-" {
            pos.state.en_passant = BITS[string_to_index(fen_split[3])];
        }

        pos
    }

    pub fn next(&self) -> Position {
        Position {
            sets: self.sets.clone(),
            all: self.all,
            player: self.player,
            enemy: self.enemy,
            state: self.state.next(),
        }
    }

    pub fn push_move(&mut self, sc: &mut SearchContext, origin: u8, target: u8, tier: u8, code: u8) {
        sc.move_table[self.state.move_ptr + self.state.move_cnt] = Move { origin, target, tier, code };
        self.state.move_cnt += 1;
    }

    pub fn generate(&mut self, sc: &mut SearchContext) {
        let enemy_attacks = get_attack_bitboard(&self, false);
        let pinned = get_pinned_bitboard(&self);
        let mut evasion_mask = !0u64;
        let space = !self.player;
        let rank4 = RANKS[4 - self.state.turn as usize];

        let pawns = self.player & self.sets[0] & !pinned;
        let mut knights = self.player & self.sets[1] & !pinned;
        let mut bishops = self.player & self.sets[2] & !pinned;
        let mut rooks = self.player & self.sets[3] & !pinned;
        let mut queens = self.player & self.sets[4] & !pinned;
        let king = self.player & self.sets[5];

        let mut king_moves = pseudo_king(king) & space & !enemy_attacks;
        let originu8 = indexu8!(king);
        while king_moves != 0 {
            let target = king_moves & 0u64.wrapping_sub(king_moves);
            self.push_move(sc, originu8, indexu8!(target), 5, 0);
            king_moves &= king_moves - 1;
        }

        if enemy_attacks & king != 0 {
            evasion_mask = get_evasion_mask(&self);
        }

        if evasion_mask != 0 {
            let mut single_push = pseudo_push(pawns, self.all, self.state.forward) & evasion_mask;
            let mut double_push = pseudo_push(single_push, self.all, self.state.forward) & rank4 & evasion_mask;
            let mut left_captures = pseudo_left_capture(pawns, self.state.forward) & self.enemy  & evasion_mask;
            let mut right_captures = pseudo_right_capture(pawns , self.state.forward) & self.enemy & evasion_mask;

            while single_push != 0 {
                let target = single_push & 0u64.wrapping_sub(single_push);
                let origin = shift_left(target, self.state.forward);
                self.push_move(sc, indexu8!(origin), indexu8!(target), 0, 0);
                single_push &= single_push - 1;
            }

            let double_push_offset = self.state.forward * 2;
            while double_push != 0 {
                let target = double_push & 0u64.wrapping_sub(double_push);
                let origin = shift_left(target, double_push_offset);
                self.push_move(sc, indexu8!(origin), indexu8!(target), 0, 5);
                double_push &= double_push - 1;
            }

            let left_capture_offset = self.state.forward - 1;
            while left_captures != 0 {
                let target = left_captures & 0u64.wrapping_sub(left_captures);
                let origin = shift_left(target, left_capture_offset);
                self.push_move(sc, indexu8!(origin), indexu8!(target), 0, 0);
                left_captures &= left_captures - 1;
            }

            let right_capture_offset = self.state.forward + 1;
            while right_captures != 0 {
                let target = right_captures & 0u64.wrapping_sub(right_captures);
                let origin = shift_left(target, right_capture_offset);
                self.push_move(sc, indexu8!(origin), indexu8!(target), 0, 0);
                right_captures &= right_captures - 1;
            }

            while knights != 0 {
                let piece = knights & 0u64.wrapping_sub(knights);
                let originu8 = indexu8!(piece);
                let mut moves = pseudo_knight(piece) & space & evasion_mask;
                while moves != 0 {
                    let target = moves & 0u64.wrapping_sub(moves);
                    self.push_move(sc, originu8, indexu8!(target), 1, 0);
                    moves &= moves - 1;
                }
                knights &= knights - 1;
            }

            while bishops != 0 {
                let piece = bishops & 0u64.wrapping_sub(bishops);
                let originu8 = indexu8!(piece);
                let mut moves = pseudo_bishop(piece, self.all) & space & evasion_mask;
                while moves != 0 {
                    let target = moves & 0u64.wrapping_sub(moves);
                    self.push_move(sc, originu8, indexu8!(target), 2, 0);
                    moves &= moves - 1;
                }
                bishops &= bishops - 1;
            }

            while rooks != 0 {
                let piece = rooks & 0u64.wrapping_sub(rooks);
                let originu8 = indexu8!(piece);
                let mut moves = pseudo_rook(piece, self.all) & space & evasion_mask;
                while moves != 0 {
                    let target = moves & 0u64.wrapping_sub(moves);
                    self.push_move(sc, originu8, indexu8!(target), 3, 0);
                    moves &= moves - 1;
                }
                rooks &= rooks - 1;
            }

            while queens != 0 {
                let piece = queens & 0u64.wrapping_sub(queens);
                let originu8 = indexu8!(piece);
                let mut moves = pseudo_queen(piece, self.all) & space & evasion_mask;
                while moves != 0 {
                    let target = moves & 0u64.wrapping_sub(moves);
                    self.push_move(sc, originu8, indexu8!(target), 4, 0);
                    moves &= moves - 1;
                }
                queens &= queens - 1;
            }
        }
    }

    pub fn make_move(&self, m: Move) -> Position {
        let mut pos = self.next();

        let origin = m.origin_bb();
        let target = m.target_bb();
        let tier = m.tier();
        let move_mask = origin | target;

        pos.sets[0] &= !move_mask;
        pos.sets[1] &= !move_mask;
        pos.sets[2] &= !move_mask;
        pos.sets[3] &= !move_mask;
        pos.sets[4] &= !move_mask;
        pos.sets[5] &= !move_mask;
        pos.sets[tier] |= target;

        pos.all ^= origin;
        pos.all |= target;

        pos.player ^= move_mask;

        pos.enemy &= !target;

        match m.code {
            // Normal moves
            0 => {}

            // Promotions
            1 | 2 | 3 | 4 => {
                pos.sets[0] &= !target;
                pos.sets[m.code as usize] |= target;
            }

            // Double pushes
            5 => {
                pos.state.en_passant = shift_left(origin, self.state.forward);
            }

            // Short castle
            6 => {
                let rook_mask = (target << 1) | (target >> 1);
                pos.all ^= rook_mask;
                pos.player ^= rook_mask;
                pos.sets[3] ^= rook_mask;
            }

            // Long castle
            7 => {
                let rook_mask = (target << 2) | (target >> 1);
                pos.all ^= rook_mask;
                pos.player ^= rook_mask;
                pos.sets[3] ^= rook_mask;
            }

            // En-passant
            8 => {
                let captured_pawn = shift_right(self.state.en_passant, self.state.forward);
                pos.all ^= captured_pawn;
                pos.sets[0] ^= captured_pawn;
            }

            _ => {}
        }

        match m.origin {
            0 => pos.state.castle_flags &= !WHITE_LONG_CASTLE,
            4 => pos.state.castle_flags &= !WHITE_CASTLES,
            7 => pos.state.castle_flags &= !WHITE_SHORT_CASTLE,
            56 => pos.state.castle_flags &= !BLACK_LONG_CASTLE,
            60 => pos.state.castle_flags &= !BLACK_CASTLES,
            63 => pos.state.castle_flags &= !BLACK_SHORT_CASTLE,
            _ => {}
        }

        match m.target {
            0 => pos.state.castle_flags &= !WHITE_LONG_CASTLE,
            7 => pos.state.castle_flags &= !WHITE_SHORT_CASTLE,
            56 => pos.state.castle_flags &= !BLACK_LONG_CASTLE,
            63 => pos.state.castle_flags &= !BLACK_SHORT_CASTLE,
            _ => {}
        }

        pos.enemy = pos.player;
        pos.player = pos.all ^ pos.enemy;

        pos
    }
}