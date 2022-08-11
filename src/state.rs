use std::time::Instant;
use crate::position::{Move, Position, STARTPOS_FEN};
use crate::tt::{create_tt, TT, TT_DEFAULT_SIZE};

pub const MAX_PLY: usize = 64;
pub const MAX_MOVE_COUNT: usize = 256;
pub const MOVE_TABLE_SIZE: usize = MAX_PLY * MAX_MOVE_COUNT;


pub struct EngineState {
    pub root: Position,
    pub root_key: u64,
    pub root_age: u8,
    pub clock: Instant,
    pub terminate: bool,
    pub hash_table: TT,
    pub move_table: [Move; MOVE_TABLE_SIZE],
    pub node_count: u64,
    pub max_depth: u8,
}

impl EngineState {
    pub fn new() -> EngineState {
        EngineState {
            root: Position::build_from_fen(STARTPOS_FEN),
            root_key: 0,
            root_age: 0,
            clock: Instant::now(),
            terminate: false,
            hash_table: create_tt(TT_DEFAULT_SIZE),
            move_table: [Move::default(); MOVE_TABLE_SIZE],
            node_count: 0,
            max_depth: 0,
        }
    }

    pub fn reset_node_count(&mut self) { self.node_count = 0 }

    pub fn start_clock(&mut self) {
        self.clock = Instant::now();
    }

    pub fn set_depth(&mut self, depth: u8) {
        self.max_depth = depth;
    }
}