use crate::{Display, Position, SearchContext};

pub fn perft(sc: &mut SearchContext, pos: &mut Position, depth_left: u8) {
    pos.generate(sc);
    if depth_left <= 1 {
        sc.node_count += pos.state.move_cnt as u64;
    } else {
        let mut i = pos.state.move_ptr;
        let last = pos.state.move_ptr + pos.state.move_cnt;
        while i < last {
            let m = sc.move_table[i];
            let start = sc.node_count;
            let node = &mut pos.make_move(m);
            perft(sc, node, depth_left - 1);
            i += 1;
            if pos.state.half_move == 0 {
                print!("{} ", sc.node_count - start);
                m.print();
            }
        }
    }
}