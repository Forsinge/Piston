use crate::context::SearchContext;
use crate::movegen::{get_attack_bitboard, get_evasion_mask};
use crate::output::Display;
use crate::position::{ Move, Position, FILE_CHARS, WHITE_SHORT_CASTLE, WHITE_LONG_CASTLE, BLACK_SHORT_CASTLE, BLACK_LONG_CASTLE};
use crate::search::perft;

#[allow(dead_code)]

mod bitboard;
mod position;
mod movegen;
mod context;
mod output;
mod search;

fn main() {
    let pos = &mut Position::build_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let sc = &mut SearchContext::new();


    sc.start_clock();
    perft(sc, pos, 6);
    println!("{}", sc.node_count);
    println!("Time elapsed: {}", sc.clock.elapsed().as_millis());
}
