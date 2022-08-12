use std::sync::{Arc, Mutex};
use crate::position::{Move, Position, STARTPOS_FEN};
use crate::tt::{create_tt, TT, TT_DEFAULT_SIZE};

pub const MAX_PLY: usize = 64;
pub const MAX_MOVE_COUNT: usize = 256;
pub const MOVE_TABLE_SIZE: usize = MAX_PLY * MAX_MOVE_COUNT;


pub struct SearchState {
    pub root: Position,
    pub root_key: u64,
    pub root_age: u8,
    pub hash_table: TT,
    pub move_table: [Move; MOVE_TABLE_SIZE],
    pub node_count: u64,
    pub max_depth: u8,
}

impl SearchState {
    pub fn new() -> SearchState {
        SearchState {
            root: Position::build_from_fen(STARTPOS_FEN),
            root_key: 0,
            root_age: 0,
            hash_table: create_tt(TT_DEFAULT_SIZE),
            move_table: [Move::default(); MOVE_TABLE_SIZE],
            node_count: 0,
            max_depth: 0,
        }
    }
}

pub struct EngineState {
    pub root: Position,
    pub move_buffer: [Move; MAX_MOVE_COUNT],
    pub search_state: Arc<Mutex<SearchState>>,
}

impl EngineState {
    pub fn new() -> EngineState {
        EngineState {
            root: Position::build_from_fen(STARTPOS_FEN),
            move_buffer: [Move::default(); MAX_MOVE_COUNT],
            search_state: Arc::new(Mutex::new(SearchState::new())),
        }
    }
}