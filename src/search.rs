
use std::sync::MutexGuard;
use std::time::Instant;
use crate::output::Display;
use crate::position::Position;
use crate::state::{MAX_MOVE_COUNT, SearchState};


pub fn perft(state: &mut MutexGuard<SearchState>) {

    let clock = Instant::now();
    let mut pos = state.root.clone();

    let slice = &mut state.move_table[0..MAX_MOVE_COUNT];
    pos.generate(slice);

    if state.max_depth > 1 {
        let mut i = 0;
        let last = pos.state.move_cnt;

        while i < last {
            let start = state.node_count;

            let m = state.move_table[i];
            let node = &mut pos.make_move(m);

            perft_internal(state, node, 1);

            print!("{} ", state.node_count - start);
            m.print();

            i += 1;
        }
    } else {
        state.node_count += state.root.state.move_cnt as u64;
    }

    println!();
    println!("Nodes searched: {}", state.node_count);

    let stop_time = clock.elapsed().as_millis();
    println!("Time elapsed: {} ms", stop_time);
    println!("Nodes per second: {} million", state.node_count as u128 / (stop_time + 1) / 1000)
}

pub fn perft_internal(state: &mut MutexGuard<SearchState>, pos: &mut Position, depth: u8) {
    let slice = &mut state.move_table[pos.state.move_ptr..pos.state.move_ptr + MAX_MOVE_COUNT];
    pos.generate(slice);

    if depth + 1 == state.max_depth {
        state.node_count += pos.state.move_cnt as u64;
    } else {
        let mut i = pos.state.move_ptr;
        let last = pos.state.move_ptr + pos.state.move_cnt;

        while i < last {
            let m = state.move_table[i];
            let node = &mut pos.make_move(m);

            perft_internal(state, node, depth + 1);

            i += 1;
        }
    }
}

