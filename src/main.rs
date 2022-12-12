#![allow(dead_code)]

mod bitboard;
mod position;
mod movegen;
mod state;
mod output;
mod search;
mod uci;
mod tt;
mod hash;
mod eval;
mod ordering;

fn main() {
    uci::uci_loop();
}
