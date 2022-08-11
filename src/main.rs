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

fn main() {
    uci::uci_loop();
}
