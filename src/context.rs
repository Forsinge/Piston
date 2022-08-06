use std::time::Instant;
use crate::position::{Move, Position};

pub const MAX_PLY: usize = 64;
pub const MAX_MOVE_COUNT: usize = 256;
pub const MOVE_TABLE_SIZE: usize = MAX_PLY * MAX_MOVE_COUNT;


pub struct SearchContext {
    pub clock: Instant,
    pub terminate: bool,
    pub move_table: [Move; MOVE_TABLE_SIZE],
    pub node_count: u64,
    pub max_depth: usize,
}

impl SearchContext {
    pub fn new() -> SearchContext {
        SearchContext {
            clock: Instant::now(),
            terminate: false,
            move_table: [Move::default(); MOVE_TABLE_SIZE],
            node_count: 0,
            max_depth: 0,
        }
    }

    pub fn start_clock(&mut self) {
        self.clock = Instant::now();
    }
}