use crate::output::Display;
use crate::position::Position;
use crate::state::EngineState;

pub fn perft(es: &mut EngineState, pos: &mut Position) {
    es.start_clock();
    es.reset_node_count();

    pos.generate(es);

    if es.max_depth > 1 {
        let mut i = 0;
        let last = pos.state.move_cnt;

        while i < last {
            let start = es.node_count;

            let m = es.move_table[i];
            let node = &mut pos.make_move(m);

            perft_internal(es, node, 1);

            print!("{} ", es.node_count - start);
            m.print();

            i += 1;
        }
    } else {
        es.node_count += pos.state.move_cnt as u64;
    }

    println!();
    println!("Total node count: {}", es.node_count);

    let stop_time = es.clock.elapsed().as_millis();
    println!("Time elapsed: {} ms", stop_time);
    println!("Nodes per second: {} million", es.node_count as u128 / (stop_time + 1) / 1000)
}

pub fn perft_internal(es: &mut EngineState, pos: &mut Position, depth: u8) {
    pos.generate(es);

    if depth + 1 == es.max_depth {
        es.node_count += pos.state.move_cnt as u64;
    } else {
        let mut i = pos.state.move_ptr;
        let last = pos.state.move_ptr + pos.state.move_cnt;

        while i < last {
            let m = es.move_table[i];
            let node = &mut pos.make_move(m);

            perft_internal(es, node, depth + 1);

            i += 1;
        }
    }
}