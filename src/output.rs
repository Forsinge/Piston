use std::ops::Add;
use piston::index;
use crate::bitboard::BITS;
use crate::{WHITE_SHORT_CASTLE, WHITE_LONG_CASTLE, BLACK_SHORT_CASTLE, BLACK_LONG_CASTLE, FILE_CHARS, Move};
use crate::position::{PositionState, WHITE_CASTLES};

pub fn string_to_index(str: &str) -> usize {
    if str == "-" {
        return 64
    }

    let mut chars = str.chars();
    let file = FILE_CHARS.binary_search(&chars.next().unwrap()).unwrap();
    let rank = chars.next().unwrap().to_digit(10).unwrap() << 3;
    file + rank as usize
}

pub fn index_to_string(index: usize) -> String {
    if index == 64 {
        return "-".to_string()
    }

    FILE_CHARS[index & 7].to_string().add((index >> 3).to_string().as_str())
}

pub trait Display {
    fn print(&self);
}

impl Display for u64 {
    fn print(&self) {
        for i in 0 .. 64 {
            let bit: u64 = BITS[i].swap_bytes();
            let symbol: &str = if bit & self != 0 { "■ " } else { ". " };
            print!("{}", symbol);
            if i % 8 == 7 { println!(); }
        }
        println!();
    }
}

impl Display for Move {
    fn print(&self) {
        let origin_file = (self.origin & 7) as usize;
        let origin_rank = (self.origin >> 3) + 1;
        let target_file = (self.target & 7) as usize;
        let target_rank = (self.target >> 3) + 1;
        println!("{}{}{}{}", FILE_CHARS[origin_file], origin_rank, FILE_CHARS[target_file], target_rank);
    }
}

impl Display for PositionState {
    fn print(&self) {
        println!("Turn: {}", self.turn);
        println!("Half move: {}", self.half_move);
        println!("White short castle: {}", self.castle_flags & WHITE_SHORT_CASTLE != 0);
        println!("White long castle: {}", self.castle_flags & WHITE_LONG_CASTLE != 0);
        println!("Black short castle: {}", self.castle_flags & BLACK_SHORT_CASTLE != 0);
        println!("Black long castle: {}", self.castle_flags & BLACK_LONG_CASTLE != 0);
        println!("En-passant: {}", index_to_string(index!(self.en_passant)));
        println!("Forward offset: {}", self.forward);
        println!("Move pointer: {}", self.move_ptr);
        println!("Move count: {}", self.move_cnt);
    }
}