use std::sync::mpsc::Receiver;
use std::sync::MutexGuard;
use std::time::Instant;
use crate::eval::{DRAW, LOSS, TERMINATE, eval};
use crate::output::{Display, print_pv};
use crate::position::{Move, Position};
use crate::state::{MAX_MOVE_COUNT, SearchState};

pub fn pvs(state: &mut MutexGuard<SearchState>, receiver: &Receiver<bool>) {
    let mut pos = state.root.clone();

    state.root_key = pos.state.key;
    state.root_age = pos.state.half_move;


    let slice = &mut state.move_table[0..MAX_MOVE_COUNT];
    pos.generate(slice);

    let clock = Instant::now();
    let mut depth = 1;
    let mut bestmove = Move::default();
    let mut besteval;

    let mut ordered_moves = [(Move::default(), 0); MAX_MOVE_COUNT];
    for i in 0..MAX_MOVE_COUNT {
        ordered_moves[i] = (state.move_table[i], 0);
    }

    'outer: loop {
        besteval = LOSS;
        let mut ptr = 0;
        while ptr < pos.state.move_cnt {

            let m = ordered_moves[ptr].0;
            let node = &mut pos.make_move(m);
            let eval = -pvs_internal(node, state, LOSS, -besteval, depth-1, receiver);

            if eval == -TERMINATE {
                break 'outer;
            }

            ordered_moves[ptr].1 = eval;

            if eval > besteval {
                bestmove = m;
                besteval = eval;
            }

            ptr += 1;
        }

        state.hash_table.place(pos.state.key, pos.state.half_move, pos.state.key, besteval, 1, depth, bestmove.to_u32());

        print!("info score cp {} nodes {} time {} depth {} ", besteval, state.stats.pvs_nodes, clock.elapsed().as_millis(), depth);
        print_pv(&pos, state);
        println!();

        ordered_moves[0..pos.state.move_cnt].sort_by(|l, r | r.1.cmp(&l.1));
        depth += 1;
    }

    print!("bestmove ");
    bestmove.print();
    println!();
}

pub fn pvs_internal(pos: &mut Position, state: &mut MutexGuard<SearchState>,
                       alpha: i16, beta: i16, depth_left: u8, receiver: &Receiver<bool>) -> i16 {

    if receiver.try_recv().is_ok() {
        return TERMINATE;
    }

    if depth_left == 0 {
        return quiesce(pos, state, alpha, beta);
    }

    state.stats.pvs_nodes += 1;

    let slice = &mut state.move_table[pos.state.move_ptr..pos.state.move_ptr + MAX_MOVE_COUNT];
    pos.generate(slice);

    if pos.state.move_cnt == 0 {
        return if pos.is_attacked(pos.player & pos.kings) {
            LOSS
        } else {
            DRAW
        }
    }

    let mut ptr = pos.state.move_ptr;
    let end = pos.state.move_ptr + pos.state.move_cnt;
    let pvmove;

    if let Some(entry) = state.hash_table.probe(pos.state.key) {
        if beta == alpha + 1 && entry.get_depth() >= depth_left {
            let entryeval = entry.get_eval();
            let outcome = entry.get_outcome();
            if outcome == 1 {
                return entryeval.clamp(alpha, beta);
            }
            if outcome == 0 && entryeval <= alpha {
                return alpha;
            }
            if outcome == 2 && entryeval >= beta {
                return beta;
            }
        }
        pvmove = entry.get_refutation();
    } else {
        pvmove = state.move_table[ptr];
        ptr += 1;
    }

    let mut besteval = alpha;
    let mut bestmove= pvmove;

    let node = &mut pos.make_move(pvmove);
    let eval = -pvs_internal(node, state, -beta, -besteval, depth_left-1, receiver);

    if eval == -TERMINATE {
        return TERMINATE;
    }

    if eval >= beta {
        let rk = state.root_key;
        let ra = state.root_age;
        state.hash_table.place(rk, ra, pos.state.key, beta, 2, depth_left, pvmove.to_u32());

        return beta;
    }

    if eval > besteval {
        besteval = eval;
    }

    while ptr < end {
        let m = state.move_table[ptr];
        let node = &mut pos.make_move(m);

        let mut eval = -pvs_internal(node, state, -besteval-1, -besteval, depth_left-1, receiver);
        if eval > besteval {
            eval = -pvs_internal(node, state, -beta, -besteval, depth_left-1, receiver);
        }

        if eval == -TERMINATE {
            return TERMINATE;
        }

        if eval >= beta {
            let rk = state.root_key;
            let ra = state.root_age;
            state.hash_table.place(rk, ra, pos.state.key, beta, 2, depth_left, m.to_u32());
            state.stats.beta_cutoffs += 1;
            return beta;
        }

        if eval > besteval {
            bestmove = m;
            besteval = eval;
        }

        ptr += 1;
    }

    let rk = state.root_key;
    let ra = state.root_age;
    state.hash_table.place(rk, ra, pos.state.key,
                           beta, (besteval != alpha) as u8, depth_left, bestmove.to_u32());
    besteval
}

pub fn quiesce(pos: &mut Position, state: &mut MutexGuard<SearchState>, alpha: i16, beta: i16) -> i16 {
    state.stats.qs_nodes += 1;

    let slice = &mut state.move_table[pos.state.move_ptr..pos.state.move_ptr + MAX_MOVE_COUNT];
    pos.generate(slice);

    if pos.state.move_cnt == 0 {
        return if pos.is_attacked(pos.player & pos.kings) {
            LOSS
        } else {
            DRAW
        }
    }

    let standing = eval(pos);
    if standing >= beta {
        return beta;
    }

    let mut besteval = alpha;
    if standing > besteval {
        besteval = standing;
    }



    let mut ptr = pos.state.move_ptr;
    let end = pos.state.move_ptr + pos.state.move_cnt;


    while ptr < end {
        let m = state.move_table[ptr];

        if m.target & pos.all != 0 {
            let node = &mut pos.make_move(m);
            let eval = -quiesce(node, state, -beta, -besteval);

            if eval >= beta {
                return beta;
            }

            if eval > besteval {
                besteval = eval;
            }
        }
        ptr += 1;
    }
    return besteval;
}


pub fn perft(state: &mut MutexGuard<SearchState>) {

    let clock = Instant::now();
    let mut pos = state.root.clone();

    let slice = &mut state.move_table[0..MAX_MOVE_COUNT];
    pos.generate(slice);

    if state.max_depth > 1 {
        let mut i = 0;
        let last = pos.state.move_cnt;

        while i < last {
            let start = state.stats.perft_nodes;

            let m = state.move_table[i];
            let node = &mut pos.make_move(m);

            perft_internal(state, node, 1);


            m.print();
            println!(": {}", state.stats.perft_nodes - start);
            i += 1;
        }
    } else {
        state.stats.perft_nodes += state.root.state.move_cnt as u64;
    }

    let stop_time = clock.elapsed().as_millis();
    println!();
    println!("Nodes searched: {}", state.stats.perft_nodes);
    println!("Time elapsed: {} ms", stop_time);
    println!("Nodes per second: {} million", state.stats.perft_nodes as u128 / (stop_time + 1) / 1000)
}

pub fn perft_internal(state: &mut MutexGuard<SearchState>, pos: &mut Position, depth: u8) {
    let slice = &mut state.move_table[pos.state.move_ptr..pos.state.move_ptr + MAX_MOVE_COUNT];
    pos.generate(slice);

    if depth + 1 == state.max_depth {
        state.stats.perft_nodes += pos.state.move_cnt as u64;
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

