use std::num::NonZeroU64;
use piston::{index};
use crate::state::{MAX_MOVE_COUNT, MOVE_TABLE_SIZE};
use crate::bitboard::{ANTIDIAGS, BITS, DIAGONALS, FILES, LUT_BISHOP, LUT_KING, LUT_KNIGHT, LUT_PAWN_CAPTURES, LUT_ROOK, RANKS, RAYS};
use crate::eval::PIECE_VALUES;
use crate::hash::{HASH_BLACK_LONG_CASTLE, HASH_BLACK_SHORT_CASTLE, HASH_ENPASSANT, HASH_PIECES, HASH_TURN, HASH_WHITE_LONG_CASTLE, HASH_WHITE_SHORT_CASTLE, zobrist_key};
use crate::movegen::*;
use crate::output::{string_to_index, Display};

pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ";

pub const WHITE_SHORT_CASTLE: u8 = 1;
pub const WHITE_LONG_CASTLE: u8 = 2;
pub const BLACK_SHORT_CASTLE: u8 = 4;
pub const BLACK_LONG_CASTLE: u8 = 8;

pub const WHITE_CASTLES: u8 = 3;
pub const BLACK_CASTLES: u8 = 12;

pub const WHITE_SHORT_CASTLE_BITS: u64 = 0x0900000000000000;
pub const WHITE_LONG_CASTLE_BITS: u64 = 0x8800000000000000;
pub const BLACK_SHORT_CASTLE_BITS: u64 = 0x09;
pub const BLACK_LONG_CASTLE_BITS: u64 = 0x88;


pub const FILE_CHARS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

#[derive(Default, Copy, Clone)]
pub struct Move {
    pub origin: u64,
    pub target: u64,
    pub tier: u8,
    pub code: u8,
}

impl Move {
    pub fn to_u32(&self) -> u32 {
        let mut int = 0;
        int |= index!(self.origin) as u32;
        int |= (index!(self.target) as u32) << 6;
        int |= (self.tier as u32) << 12;
        int |= (self.code as u32) << 16;
        int
    }

    pub fn from_u32(m: u32) -> Move {
        let origin = BITS[(m & 0x3F) as usize];
        let target = BITS[((m >> 6) & 0x3F) as usize];
        let tier = ((m >> 12) & 0x7) as u8;
        let code = ((m >> 16) & 0xF) as u8;
        Move { origin, target, tier, code }
    }

    pub fn tier(&self) -> usize {
        self.tier as usize
    }
}

#[derive(Default, Copy, Clone)]
pub struct PositionState {
    pub key: u64,
    pub material_balance: i16,
    pub en_passant: u64,
    pub castle_flags: u8,
    pub move_ptr: usize,
    pub move_cnt: usize,
    pub half_move: u8,
    pub turn: bool,
    pub last_move: Move,
}

impl PositionState {
    pub fn next(&self, m: Move) -> PositionState {
        PositionState {
            key: self.key ^ HASH_TURN,
            material_balance: -self.material_balance,
            en_passant: 0,
            castle_flags: self.castle_flags,
            move_ptr: (self.move_ptr + MAX_MOVE_COUNT) & (MOVE_TABLE_SIZE - 1),
            move_cnt: 0,
            half_move: self.half_move + 1,
            turn: !self.turn,
            last_move: m,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Position {
    pub pawns: u64,
    pub knights: u64,
    pub bishops: u64,
    pub rooks: u64,
    pub queens: u64,
    pub kings: u64,
    pub all: u64,
    pub player: u64,
    pub enemy: u64,
    pub state: PositionState,
}

impl Position {
    pub fn next(&self, cleared_bits: u64, m: Move) -> Position {
        Position {
            pawns: self.pawns & cleared_bits,
            knights: self.knights & cleared_bits,
            bishops: self.bishops & cleared_bits,
            rooks: self.rooks & cleared_bits,
            queens: self.queens & cleared_bits,
            kings: self.kings & cleared_bits,
            all: self.all,
            player: self.player,
            enemy: self.enemy,
            state: self.state.next(m),
        }
    }

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

                match tier {
                    0 => pos.pawns |= bit,
                    1 => pos.knights |= bit,
                    2 => pos.bishops |= bit,
                    3 => pos.rooks |= bit,
                    4 => pos.queens |= bit,
                    5 => pos.kings |= bit,
                    _ => {},
                }

                pos.all |= bit;

                if c.is_uppercase() {
                    pos.state.material_balance += PIECE_VALUES[tier];
                    pos.player |= bit;
                } else {
                    pos.state.material_balance -= PIECE_VALUES[tier];
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
        }

        else {
            pos.state.material_balance = -pos.state.material_balance;
            pos.enemy = pos.player;
            pos.player ^= pos.all;
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
            let offset = if pos.state.turn { -8 } else { 8 };
            pos.state.en_passant = BITS[(string_to_index(fen_split[3]) as i8 - offset) as usize];
        }

        pos.state.key = zobrist_key(&pos);
        pos
    }

    pub fn push_move_with_code(&mut self, move_slice: &mut [Move], origin: u64, target: u64, tier: u8, code: u8) {
        move_slice[self.state.move_cnt] = Move { origin, target, tier, code };
        self.state.move_cnt += 1;
    }

    pub fn push_move(&mut self, move_slice: &mut [Move], origin: u64, target: u64, tier: u8) {
        move_slice[self.state.move_cnt] = Move { origin, target, tier, code: 0};
        self.state.move_cnt += 1;
    }

    pub fn square_tier(&self, square: u64) -> usize {
        if square & self.pawns != 0 { return 0 }
        if square & self.knights != 0 { return 1 }
        if square & self.bishops != 0 { return 2 }
        if square & self.rooks != 0 { return 3 }
        if square & self.queens != 0 { return 4 }
        if square & self.kings != 0 { return 5 }
        6
    }

    pub fn is_attacked(&self, square: u64) -> bool {
        let index = index!(NonZeroU64::new(square).unwrap());

        if LUT_KNIGHT[index] & self.knights & self.enemy != 0 {
            return true;
        }

        if LUT_KING[index] & self.kings & self.enemy != 0 {
            return true;
        }

        if LUT_PAWN_CAPTURES[!self.state.turn as usize][index] & self.pawns & self.enemy != 0 {
            return true;
        }

        if pseudo_rook(square, self.all, index) & (self.rooks | self.queens) & self.enemy != 0 {
            return true;
        }

        if pseudo_bishop(square, self.all, index) & (self.bishops | self.queens) & self.enemy != 0 {
            return true;
        }

        false
    }

    pub fn get_attack_bitboard(&self, player: u64, shift_offset: u8) -> u64 {
        let mut bb = 0;

        let all = self.all & !(self.kings & (self.all ^ player));

        let pawns = player & self.pawns;
        let mut knights = player & self.knights;
        let mut bishops = player & (self.bishops | self.queens);
        let mut rooks = player & (self.rooks | self.queens);

        bb |= pseudo_left_capture(pawns, shift_offset);
        bb |= pseudo_right_capture(pawns, shift_offset);
        bb |= pseudo_king(index!(NonZeroU64::new(player & self.kings).unwrap()));

        while knights != 0 {
            let index = 63 - NonZeroU64::new(knights).unwrap().trailing_zeros() as usize;
            bb |= pseudo_knight(index);
            knights &= knights - 1;
        }

        while bishops != 0 {
            let index = 63 - NonZeroU64::new(bishops).unwrap().trailing_zeros() as usize;
            let piece = bishops & (!bishops + 1);
            bb |= pseudo_bishop(piece, all, index);
            bishops &= bishops - 1;
        }

        while rooks != 0 {
            let index = 63 - NonZeroU64::new(rooks).unwrap().trailing_zeros() as usize;
            let piece = rooks & (!rooks + 1);
            bb |= pseudo_rook(piece, all, index);
            rooks &= rooks - 1;
        }

        bb
    }

    pub fn get_pinned_bitboard(&self, king: u64, king_index: usize) -> u64 {
        let mut pinned = 0;

        let diag_snipers = self.enemy & (self.bishops | self.queens);
        let line_snipers = self.enemy & (self.rooks | self.queens);

        let blockers = pseudo_queen(king, self.all, king_index) & self.all;
        let cleared = self.all ^ (blockers & self.player);

        let mut snipers = ((pseudo_rook(king, cleared, king_index) & line_snipers)
            | (pseudo_bishop(king, cleared, king_index) & diag_snipers)) & !blockers;

        while snipers != 0 {
            let piece = NonZeroU64::new(snipers & (!snipers + 1)).unwrap();
            pinned |= blockers & RAYS[king_index][index!(piece)];
            snipers &= snipers - 1;
        }

        pinned
    }

    pub fn get_evasion_mask(&self, king: u64, king_index: usize) -> u64 {

        let attacking_pawns = LUT_PAWN_CAPTURES[!self.state.turn as usize][king_index] & self.enemy & self.pawns;

        let attacking_knights = pseudo_knight(king_index) & self.enemy & self.knights;

        let bishops = self.enemy & (self.bishops | self.queens);
        let attacking_bishops = pseudo_bishop(king, self.all, king_index) & bishops;

        let rooks = self.enemy & (self.rooks | self.queens);
        let attacking_rooks = pseudo_rook(king, self.all, king_index) & rooks;

        let attackers_mask = attacking_pawns | attacking_knights | attacking_bishops | attacking_rooks;

        return if attackers_mask & (attackers_mask - 1) == 0 {
            if attacking_bishops != 0 {
                let attacker_index = index!(NonZeroU64::new(attacking_bishops).unwrap());
                pseudo_bishop(king, self.all, king_index)
                    & pseudo_bishop(attacking_bishops, self.all, attacker_index)
                    | attacking_bishops
            } else if attacking_rooks != 0 {
                let attacker_index = index!(NonZeroU64::new(attacking_rooks).unwrap());
                pseudo_rook(king, self.all, king_index)
                    & pseudo_rook(attacking_rooks, self.all, attacker_index)
                    | attacking_rooks
            } else {
                attackers_mask
            }
        } else {
            0
        }
    }

    pub fn generate(&mut self, move_slice: &mut[Move]) {
        let king = self.player & self.kings;
        let king_index = index!(NonZeroU64::new(king).unwrap());
        let shift_offset = (self.state.turn as u8) << 4;

        let enemy_attacks = self.get_attack_bitboard(self.enemy, shift_offset ^ 16);

        let evasion_mask;
        let space = !self.player;

        let mut king_moves = pseudo_king(king_index) & space & !enemy_attacks;
        while king_moves != 0 {
            self.push_move(move_slice, king, king_moves & (!king_moves + 1), 5);
            king_moves &= king_moves - 1;
        }

        if enemy_attacks & king != 0 {
            evasion_mask = self.get_evasion_mask(king, king_index);
        } else {
            evasion_mask = !0u64;

            let castle_bit = if self.state.turn { WHITE_SHORT_CASTLE } else { BLACK_SHORT_CASTLE };
            if self.state.castle_flags & castle_bit != 0 {
                let king_slide = (king >> 1) | (king >> 2);
                if king_slide & (enemy_attacks | self.all) == 0 {
                    self.push_move_with_code(move_slice, king, king >> 2, 5, 6);
                }
            }
            if self.state.castle_flags & (castle_bit << 1) != 0 {
                let king_slide = (king << 1) | (king << 2);
                let rook_slide = king_slide | (king << 3);
                if king_slide & enemy_attacks == 0 && rook_slide & self.all == 0 {
                    self.push_move_with_code(move_slice, king, king << 2, 5, 7);
                }
            }

        }

        if evasion_mask != 0 {

            let move_mask = evasion_mask & space;
            let pinned = self.get_pinned_bitboard(king, king_index);
            let free = self.player & !pinned;

            let pawns = free & self.pawns;
            let mut knights = free & self.knights;
            let mut bishops = free & self.bishops;
            let mut rooks = free & self.rooks;
            let mut queens = free & self.queens;

            if self.state.turn {
                let mut single_push = ((pawns | (self.pawns & pinned & FILES[king_index & 7])) >> 8) & !self.all;
                let mut double_push = ((single_push) >> 8) & RANKS[3] & evasion_mask & !self.all;
                let mut left_captures = ((pawns | (self.pawns & pinned & ANTIDIAGS[king_index])) >> 7) & !FILES[7] & evasion_mask & self.enemy;
                let mut right_captures = ((pawns | (self.pawns & pinned & DIAGONALS[king_index])) >> 9) & !FILES[0] & evasion_mask & self.enemy;
                single_push &= evasion_mask;


                if pawns & RANKS[6] != 0 {
                    let rank8 = RANKS[7];
                    let mut single_promo = single_push & rank8;
                    let mut left_capture_promo = left_captures & rank8;
                    let mut right_capture_promo = right_captures & rank8;

                    single_push &= !rank8;
                    left_captures &= !rank8;
                    right_captures &= !rank8;

                    while single_promo != 0 {
                        let target = single_promo & (!single_promo + 1);
                        let origin = target << 8;
                        self.push_move_with_code(move_slice, origin, target, 0, 1);
                        self.push_move_with_code(move_slice, origin, target, 0, 2);
                        self.push_move_with_code(move_slice, origin, target, 0, 3);
                        self.push_move_with_code(move_slice, origin, target, 0, 4);
                        single_promo &= single_promo - 1;
                    }

                    while left_capture_promo != 0 {
                        let target = left_capture_promo & (!left_capture_promo + 1);
                        let origin = target << 7;
                        self.push_move_with_code(move_slice, origin, target, 0, 1);
                        self.push_move_with_code(move_slice, origin, target, 0, 2);
                        self.push_move_with_code(move_slice, origin, target, 0, 3);
                        self.push_move_with_code(move_slice, origin, target, 0, 4);
                        left_capture_promo &= left_capture_promo - 1;
                    }

                    while right_capture_promo != 0 {
                        let target = right_capture_promo & (!right_capture_promo + 1);
                        let origin = target << 9;
                        self.push_move_with_code(move_slice, origin, target, 0, 1);
                        self.push_move_with_code(move_slice, origin, target, 0, 2);
                        self.push_move_with_code(move_slice, origin, target, 0, 3);
                        self.push_move_with_code(move_slice, origin, target, 0, 4);
                        right_capture_promo &= right_capture_promo - 1;
                    }
                }


                let ps = self.state.en_passant;
                if ps & evasion_mask != 0 {
                    let left_capturer = (ps << 1) & !FILES[7] & self.pawns & self.player;
                    let right_capturer = (ps >> 1) & !FILES[0] & self.pawns & self.player;

                    if left_capturer != 0 && (left_capturer & pinned == 0 || left_capturer & DIAGONALS[king_index] != 0) {
                        let target = ps >> 8;
                        let en_passant_mask = left_capturer | ps | target;
                        let line_sliders = self.enemy & (self.rooks | self.queens);
                        if pseudo_rook(king, self.all ^ en_passant_mask, king_index) & line_sliders == 0 {
                            self.push_move_with_code(move_slice, left_capturer, target, 0, 8);
                        }
                    }

                    if right_capturer != 0 && (right_capturer & pinned == 0 || right_capturer & ANTIDIAGS[king_index] != 0) {
                        let target = ps >> 8;
                        let en_passant_mask = right_capturer | ps | target;

                        let line_sliders = self.enemy & (self.rooks | self.queens);
                        if pseudo_rook(king, self.all ^ en_passant_mask, king_index) & line_sliders == 0 {
                            self.push_move_with_code(move_slice, right_capturer, target, 0, 8);
                        }
                    }
                }

                while single_push != 0 {
                    let target = single_push & (!single_push + 1);
                    let origin = target << 8;
                    self.push_move(move_slice, origin, target, 0);
                    single_push &= single_push - 1;
                }

                while double_push != 0 {
                    let target = double_push & (!double_push + 1);
                    let origin = target << 16;
                    self.push_move_with_code(move_slice, origin, target, 0, 5);
                    double_push &= double_push - 1;
                }

                while left_captures != 0 {
                    let target = left_captures & (!left_captures + 1);
                    let origin = target << 7;
                    self.push_move(move_slice, origin, target, 0);
                    left_captures &= left_captures - 1;
                }

                while right_captures != 0 {
                    let target = right_captures & (!right_captures + 1);
                    let origin = target << 9;
                    self.push_move(move_slice, origin, target, 0);
                    right_captures &= right_captures - 1;
                }
            }

            else {
                let mut single_push = ((pawns | (self.pawns & pinned & FILES[king_index & 7])) << 8) & !self.all;
                let mut double_push = ((single_push) << 8) & RANKS[4] & evasion_mask & !self.all;
                let mut left_captures = ((pawns | (self.pawns & pinned & DIAGONALS[king_index])) << 9) & !FILES[7] & evasion_mask & self.enemy;
                let mut right_captures = ((pawns | (self.pawns & pinned & ANTIDIAGS[king_index])) << 7) & !FILES[0] & evasion_mask & self.enemy;
                single_push &= evasion_mask;


                if pawns & RANKS[1] != 0 {
                    let rank8 = RANKS[0];
                    let mut single_promo = single_push & rank8;
                    let mut left_capture_promo = left_captures & rank8;
                    let mut right_capture_promo = right_captures & rank8;

                    single_push &= !rank8;
                    left_captures &= !rank8;
                    right_captures &= !rank8;

                    while single_promo != 0 {
                        let target = single_promo & (!single_promo + 1);
                        let origin = target >> 8;
                        self.push_move_with_code(move_slice, origin, target, 0, 1);
                        self.push_move_with_code(move_slice, origin, target, 0, 2);
                        self.push_move_with_code(move_slice, origin, target, 0, 3);
                        self.push_move_with_code(move_slice, origin, target, 0, 4);
                        single_promo &= single_promo - 1;
                    }

                    while left_capture_promo != 0 {
                        let target = left_capture_promo & (!left_capture_promo + 1);
                        let origin = target >> 9;
                        self.push_move_with_code(move_slice, origin, target, 0, 1);
                        self.push_move_with_code(move_slice, origin, target, 0, 2);
                        self.push_move_with_code(move_slice, origin, target, 0, 3);
                        self.push_move_with_code(move_slice, origin, target, 0, 4);
                        left_capture_promo &= left_capture_promo - 1;
                    }

                    while right_capture_promo != 0 {
                        let target = right_capture_promo & (!right_capture_promo + 1);
                        let origin = target >> 7;
                        self.push_move_with_code(move_slice, origin, target, 0, 1);
                        self.push_move_with_code(move_slice, origin, target, 0, 2);
                        self.push_move_with_code(move_slice, origin, target, 0, 3);
                        self.push_move_with_code(move_slice, origin, target, 0, 4);
                        right_capture_promo &= right_capture_promo - 1;
                    }
                }


                let ps = self.state.en_passant;
                if ps & evasion_mask != 0 {
                    let left_capturer = (ps << 1) & !FILES[7] & self.pawns & self.player;
                    let right_capturer = (ps >> 1) & !FILES[0] & self.pawns & self.player;

                    if left_capturer != 0 && (left_capturer & pinned == 0 || left_capturer & ANTIDIAGS[king_index] != 0) {
                        let target = ps << 8;
                        let en_passant_mask = left_capturer | ps | target;
                        let line_sliders = self.enemy & (self.rooks | self.queens);
                        if pseudo_rook(king, self.all ^ en_passant_mask, king_index) & line_sliders == 0 {
                            self.push_move_with_code(move_slice, left_capturer, target, 0, 8);
                        }
                    }

                    if right_capturer != 0 && (right_capturer & pinned == 0 || right_capturer & DIAGONALS[king_index] != 0) {
                        let target = ps << 8;
                        let en_passant_mask = right_capturer | ps | target;

                        let line_sliders = self.enemy & (self.rooks | self.queens);
                        if pseudo_rook(king, self.all ^ en_passant_mask, king_index) & line_sliders == 0 {
                            self.push_move_with_code(move_slice, right_capturer, target, 0, 8);
                        }
                    }
                }

                while single_push != 0 {
                    let target = single_push & (!single_push + 1);
                    let origin = target >> 8;
                    self.push_move(move_slice, origin, target, 0);
                    single_push &= single_push - 1;
                }

                while double_push != 0 {
                    let target = double_push & (!double_push + 1);
                    let origin = target >> 16;
                    self.push_move_with_code(move_slice, origin, target, 0, 5);
                    double_push &= double_push - 1;
                }

                while left_captures != 0 {
                    let target = left_captures & (!left_captures + 1);
                    let origin = target >> 9;
                    self.push_move(move_slice, origin, target, 0);
                    left_captures &= left_captures - 1;
                }

                while right_captures != 0 {
                    let target = right_captures & (!right_captures + 1);
                    let origin = target >> 7;
                    self.push_move(move_slice, origin, target, 0);
                    right_captures &= right_captures - 1;
                }
            }

            while knights != 0 {
                let origin = knights & (!knights + 1);
                let index = NonZeroU64::new(origin).unwrap().leading_zeros() as usize;
                let mut moves = pseudo_knight(index) & move_mask;
                while moves != 0 {
                    self.push_move(move_slice, origin, moves & (!moves + 1), 1);
                    moves &= moves - 1;
                }
                knights &= knights - 1;
            }

            while bishops != 0 {
                let piece = bishops & (!bishops + 1);
                let origin_index = NonZeroU64::new(piece).unwrap().leading_zeros() as usize;
                let mut moves = pseudo_bishop(piece, self.all, origin_index) & move_mask;
                while moves != 0 {
                    self.push_move(move_slice, piece, moves & (!moves + 1), 2);
                    moves &= moves - 1;
                }
                bishops &= bishops - 1;
            }

            while rooks != 0 {
                let piece = rooks & (!rooks + 1);
                let origin_index = NonZeroU64::new(piece).unwrap().leading_zeros() as usize;
                let mut moves = pseudo_rook(piece, self.all, origin_index) & move_mask;
                while moves != 0 {
                    self.push_move(move_slice, piece, moves & (!moves + 1), 3);
                    moves &= moves - 1;
                }
                rooks &= rooks - 1;
            }


            while queens != 0 {
                let piece = queens & (!queens + 1);
                let origin_index = NonZeroU64::new(piece).unwrap().leading_zeros() as usize;
                let mut moves = pseudo_queen(piece, self.all, origin_index) & move_mask;
                while moves != 0 {
                    self.push_move(move_slice, piece, moves & (!moves + 1), 4);
                    moves &= moves - 1;
                }
                queens &= queens - 1;
            }

            if evasion_mask == !0u64 {
                let line_pins = pinned & (LUT_ROOK[king_index]);
                let mut line_sliders = self.player & self.rooks & line_pins;
                while line_sliders != 0 {
                    let piece = line_sliders & (!line_sliders + 1);
                    let origin_index = NonZeroU64::new(piece).unwrap().leading_zeros() as usize;
                    let mut moves = pseudo_rook(piece, self.all, origin_index) & space & LUT_ROOK[king_index];
                    while moves != 0 {
                        let target = moves & (!moves + 1);
                        self.push_move(move_slice, piece, target, 3);
                        moves &= moves - 1;
                    }
                    line_sliders &= line_sliders - 1;
                }

                let mut line_sliders = self.player & self.queens & line_pins;
                while line_sliders != 0 {
                    let piece = line_sliders & (!line_sliders + 1);
                    let origin_index = NonZeroU64::new(piece).unwrap().leading_zeros() as usize;
                    let mut moves = pseudo_rook(piece, self.all, origin_index) & space & LUT_ROOK[king_index];
                    while moves != 0 {
                        let target = moves & (!moves + 1);
                        self.push_move(move_slice, piece, target, 4);
                        moves &= moves - 1;
                    }
                    line_sliders &= line_sliders - 1;
                }

                let diag_pins = pinned & (LUT_BISHOP[king_index]);

                let mut diag_sliders = self.player & self.bishops & diag_pins;
                while diag_sliders != 0 {
                    let piece = diag_sliders & (!diag_sliders + 1);
                    let origin_index = NonZeroU64::new(piece).unwrap().leading_zeros() as usize;
                    let mut moves = pseudo_bishop(piece, self.all, origin_index) & space & LUT_BISHOP[king_index];
                    while moves != 0 {
                        let target = moves & (!moves + 1);
                        self.push_move(move_slice, piece, target, 2);
                        moves &= moves - 1;
                    }
                    diag_sliders &= diag_sliders - 1;
                }

                let mut diag_sliders = self.player & self.queens & diag_pins;
                while diag_sliders != 0 {
                    let piece = diag_sliders & (!diag_sliders + 1);
                    let origin_index = NonZeroU64::new(piece).unwrap().leading_zeros() as usize;
                    let mut moves = pseudo_bishop(piece, self.all, origin_index) & space & LUT_BISHOP[king_index];
                    while moves != 0 {
                        let target = moves & (!moves + 1);
                        self.push_move(move_slice, piece, target, 4);
                        moves &= moves - 1;
                    }
                    diag_sliders &= diag_sliders - 1;
                }
            }
        }
    }

    pub fn make_move(&self, m: Move) -> Position {
        let origin = m.origin;
        let target = m.target;
        let origin_index = index!(NonZeroU64::new(origin).unwrap());
        let target_index = index!(NonZeroU64::new(target).unwrap());

        let player_tier = self.state.turn as usize * 6;
        let tier = m.tier();
        let move_mask = origin | target;

        let mut pos = self.next(!move_mask, m);

        match tier {
            0 => {
                pos.pawns |= target;
                pos.state.key ^= HASH_PIECES[0 + player_tier][origin_index];
                pos.state.key ^= HASH_PIECES[0 + player_tier][target_index];
            }
            1 => {
                pos.knights |= target;
                pos.state.key ^= HASH_PIECES[1 + player_tier][origin_index];
                pos.state.key ^= HASH_PIECES[1 + player_tier][target_index];
            }
            2 => {
                pos.bishops |= target;
                pos.state.key ^= HASH_PIECES[2 + player_tier][origin_index];
                pos.state.key ^= HASH_PIECES[2 + player_tier][target_index];
            },
            3 => {
                pos.rooks |= target;
                pos.state.key ^= HASH_PIECES[3 + player_tier][origin_index];
                pos.state.key ^= HASH_PIECES[3 + player_tier][target_index];
            },
            4 => {
                pos.queens |= target;
                pos.state.key ^= HASH_PIECES[4 + player_tier][origin_index];
                pos.state.key ^= HASH_PIECES[4 + player_tier][target_index];
            },
            5 => {
                pos.kings |= target;
                pos.state.key ^= HASH_PIECES[5 + player_tier][origin_index];
                pos.state.key ^= HASH_PIECES[5 + player_tier][target_index];
            },
            _ => {}
        }

        pos.all ^= origin;
        pos.all |= target;
        pos.player ^= move_mask;
        pos.enemy &= !target;

        match m.code {
            // Normal moves
            0 => {}

            // Promotions
            1 => {
                pos.pawns &= !target;
                pos.knights |= target;
                pos.state.key ^= HASH_PIECES[0 + player_tier][target_index];
                pos.state.key ^= HASH_PIECES[1 + player_tier][target_index];
                pos.state.material_balance += PIECE_VALUES[0];
                pos.state.material_balance -= PIECE_VALUES[1];
            }

            2 => {
                pos.pawns &= !target;
                pos.bishops |= target;
                pos.state.key ^= HASH_PIECES[0 + player_tier][target_index];
                pos.state.key ^= HASH_PIECES[2 + player_tier][target_index];
                pos.state.material_balance += PIECE_VALUES[0];
                pos.state.material_balance -= PIECE_VALUES[2];
            }

            3 => {
                pos.pawns &= !target;
                pos.rooks |= target;
                pos.state.key ^= HASH_PIECES[0 + player_tier][target_index];
                pos.state.key ^= HASH_PIECES[3 + player_tier][target_index];
                pos.state.material_balance += PIECE_VALUES[0];
                pos.state.material_balance -= PIECE_VALUES[3];
            }

            4 => {
                pos.pawns &= !target;
                pos.queens |= target;
                pos.state.key ^= HASH_PIECES[0 + player_tier][target_index];
                pos.state.key ^= HASH_PIECES[4 + player_tier][target_index];
                pos.state.material_balance += PIECE_VALUES[0];
                pos.state.material_balance -= PIECE_VALUES[4];
            }

            // Double pushes
            5 => {
                pos.state.en_passant = target;
                pos.state.key ^= HASH_ENPASSANT[target_index & 7];
            }

            // Short castle
            6 => {
                let rook_mask = (target << 1) | (target >> 1);
                pos.all ^= rook_mask;
                pos.player ^= rook_mask;
                pos.rooks ^= rook_mask;
                pos.state.key ^= HASH_PIECES[3 + player_tier][target_index - 1];
                pos.state.key ^= HASH_PIECES[3 + player_tier][target_index + 1];
            }

            // Long castle
            7 => {
                let rook_mask = (target << 2) | (target >> 1);
                pos.all ^= rook_mask;
                pos.player ^= rook_mask;
                pos.rooks ^= rook_mask;
                pos.state.key ^= HASH_PIECES[3 + player_tier][target_index - 2];
                pos.state.key ^= HASH_PIECES[3 + player_tier][target_index + 1];
            }

            // En-passant
            8 => {
                pos.all ^= self.state.en_passant;
                pos.enemy ^= self.state.en_passant;
                pos.rooks ^= self.state.en_passant;
                pos.state.key ^= HASH_PIECES[0 + (player_tier ^ 6)][index!(NonZeroU64::new(self.state.en_passant).unwrap())];
                pos.state.material_balance -= PIECE_VALUES[0];
            }

            _ => {}
        }

        if self.all & target != 0 {
            let captured_tier = self.square_tier(target);
            pos.state.key ^= HASH_PIECES[captured_tier + (player_tier ^ 6)][target_index];
            pos.state.material_balance -= PIECE_VALUES[captured_tier];
        }

        if self.state.castle_flags & WHITE_SHORT_CASTLE != 0 && move_mask & WHITE_SHORT_CASTLE_BITS != 0 {
            pos.state.castle_flags &= !WHITE_SHORT_CASTLE;
            pos.state.key ^= HASH_WHITE_SHORT_CASTLE;
        }

        if self.state.castle_flags & WHITE_LONG_CASTLE != 0 && move_mask & WHITE_LONG_CASTLE_BITS != 0 {
            pos.state.castle_flags &= !WHITE_LONG_CASTLE;
            pos.state.key ^= HASH_WHITE_LONG_CASTLE;
        }

        if self.state.castle_flags & BLACK_SHORT_CASTLE != 0 && move_mask & BLACK_SHORT_CASTLE_BITS != 0 {
            pos.state.castle_flags &= !BLACK_SHORT_CASTLE;
            pos.state.key ^= HASH_BLACK_SHORT_CASTLE;
        }

        if self.state.castle_flags & BLACK_LONG_CASTLE != 0 && move_mask & BLACK_LONG_CASTLE_BITS != 0 {
            pos.state.castle_flags &= !BLACK_LONG_CASTLE;
            pos.state.key ^= HASH_BLACK_LONG_CASTLE;
        }

        if self.state.last_move.code == 5 {
            pos.state.key ^= HASH_ENPASSANT[index!(self.state.last_move.target) & 7];
        }

        pos.enemy = pos.player;
        pos.player = pos.all ^ pos.enemy;

        pos
    }

    pub fn print_moves(&self, move_slice: &mut [Move]) {
        let mut i = 0;
        while i < self.state.move_cnt {
            let m = move_slice[i];
            if m.origin != m.target {
                m.print();
                println!();
            }
            i += 1;
        }
    }
}