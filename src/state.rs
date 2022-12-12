use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use crate::position::{Move, Position, STARTPOS_FEN};
use crate::tt::{create_tt, TT, TT_DEFAULT_SIZE};

pub const MAX_PLY: usize = 64;
pub const MAX_MOVE_COUNT: usize = 256;
pub const MOVE_TABLE_SIZE: usize = MAX_PLY * MAX_MOVE_COUNT;

pub struct SearchStats {
    pub perft_nodes: u64,
    pub pvs_nodes: u64,
    pub qs_nodes: u64,
    pub beta_cutoffs: u64,
    pub table_probes: u64,
    pub table_hits: u64,
}

impl SearchStats {
    pub fn new() -> SearchStats {
        SearchStats {
            perft_nodes: 0,
            pvs_nodes: 0,
            qs_nodes: 0,
            beta_cutoffs: 0,
            table_probes: 0,
            table_hits: 0,
        }
    }
}

pub struct SearchState {
    pub root: Position,
    pub root_key: u64,
    pub root_age: u8,
    pub hash_table: TT,
    pub move_table: [Move; MOVE_TABLE_SIZE],
    pub killer_table: [[Move; 2]; 64], // needs to match max_depth
    pub max_depth: u8,
    pub stats: SearchStats,
}

impl SearchState {
    pub fn new() -> SearchState {
        SearchState {
            root: Position::build_from_fen(STARTPOS_FEN),
            root_key: 0,
            root_age: 0,
            hash_table: create_tt(TT_DEFAULT_SIZE),
            move_table: [Move::default(); MOVE_TABLE_SIZE],
            killer_table: [[Move::default(); 2]; 64],
            max_depth: 0,
            stats: SearchStats::new(),
        }
    }
}

pub struct EngineState {
    pub root: Position,
    pub move_buffer: [Move; MAX_MOVE_COUNT],
    pub terminate: Arc<AtomicBool>,
    pub search_state: Arc<Mutex<SearchState>>,
}

impl EngineState {
    pub fn new() -> EngineState {
        EngineState {
            root: Position::build_from_fen(STARTPOS_FEN),
            move_buffer: [Move::default(); MAX_MOVE_COUNT],
            terminate: Arc::new(AtomicBool::new(false)),
            search_state: Arc::new(Mutex::new(SearchState::new())),
        }
    }
}