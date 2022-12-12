use std::sync::MutexGuard;
use crate::position::{Move, Position};
use crate::eval::PIECE_VALUES;
use crate::ordering::PickerStage::*;
use crate::state::{MAX_MOVE_COUNT, SearchState};

pub fn add_killer(ply: u8, state: &mut MutexGuard<SearchState>, m: Move) {
    let table = &mut state.killer_table;
    let index = ply as usize % table.len();
    let ply_arr = &mut table[index];
    ply_arr[1] = ply_arr[0];
    ply_arr[0] = m;
}

#[derive(Copy, Clone, PartialEq)]
pub enum PickerStage {
    TTMove,
    Killer1,
    Killer2,
    GenTactical,
    HighPrio,
    GenQuiet,
    Quiet,
    LowPrio,
    End,
}

pub struct PVSPicker<'a> {
    pub pos: &'a mut Position,
    pub ttmove: Option<Move>,
    pub stage: PickerStage,
    pub tactical_count: usize,
    pub quiet_count: usize,
    pub scores: [i16; 256],
    pub ply: usize,
}

impl PVSPicker<'_> {
    pub fn new(pos: &mut Position, ttmove: Option<Move>, ply: usize) -> PVSPicker {
        PVSPicker {
            pos,
            ttmove,
            stage: TTMove,
            tactical_count: 0,
            quiet_count: 0,
            scores: [0; 256],
            ply: ply % 64,
        }
    }

    // use buffer and counters for each stage to generalize
    // use a score array to pick from, end when counter is 0
    pub fn next(&mut self, state: &mut MutexGuard<SearchState>) -> (Option<Move>, Option<Position>) {
        let list = &mut state.move_table[self.pos.state.move_ptr..self.pos.state.move_ptr+MAX_MOVE_COUNT];
        match self.stage {
            TTMove => {

                self.stage = Killer1;
                if self.ttmove.is_some() {
                    let m = self.ttmove.unwrap();
                    return (Some(m), Some(self.pos.make_move(m)));
                }
                return self.next(state);
            }

            Killer1 => {
                let m = state.killer_table[self.ply][0];
                self.stage = Killer2;
                return if self.pos.killer_is_legal(m) {
                    (Some(m), Some(self.pos.make_move(m)))
                } else {
                    self.next(state)
                }

            }

            Killer2 => {

                let m = state.killer_table[self.ply][1];
                self.stage = GenTactical;
                return if self.pos.killer_is_legal(m) {
                    (Some(m), Some(self.pos.make_move(m)))
                } else {
                    self.next(state)
                }
            }

            GenTactical => {

                self.pos.generate_tactical(list);
                self.tactical_count = self.pos.state.move_cnt;

                for i in 0..self.tactical_count {
                    let m = list[i];
                    self.scores[i] = tactical_score(&self.pos, m);
                }

                self.stage = HighPrio;
                self.next(state)
            }

            HighPrio => {
                let mut i = 0;
                let mut max = i16::MIN;
                for j in 0..self.tactical_count {
                    if self.scores[j] > max {
                        i = j;
                        max = self.scores[j];
                    }
                }

                if max < 0 {
                    self.stage = GenQuiet;
                    return self.next(state);
                } else {
                    self.scores[i] = i16::MIN;
                    let m = list[i];
                    return (Some(m), Some(self.pos.make_move(m)));
                }
            }

            GenQuiet => {

                self.pos.generate_quiet(list);
                self.quiet_count = self.pos.state.move_cnt - self.tactical_count;

                for i in self.tactical_count..self.pos.state.move_cnt {
                    let m = list[i];
                    self.scores[i] = quiet_score(&self.pos, m);
                }

                self.stage = Quiet;
                self.next(state)
            }

            Quiet => {

                let mut i = 0;
                let mut max = i16::MIN;
                for j in self.tactical_count..self.pos.state.move_cnt {
                    if self.scores[j] > max {
                        i = j;
                        max = self.scores[j];
                    }
                }

                if max == i16::MIN {
                    self.stage = LowPrio;
                    return self.next(state);
                } else {
                    self.scores[i] = i16::MIN;
                    let m = list[i];
                    return (Some(m), Some(self.pos.make_move(m)));
                }
            }

            LowPrio => {
                let mut i = 0;
                let mut max = i16::MIN;
                for j in 0..self.tactical_count {
                    if self.scores[j] > max {
                        i = j;
                        max = self.scores[j];
                    }
                }

                if max == i16::MIN {
                    self.stage = End;
                    return self.next(state);
                } else {
                    self.scores[i] = i16::MIN;
                    let m = list[i];
                    return (Some(m), Some(self.pos.make_move(m)));
                }
            }

            End => {
                (None, None)
            }
        }
    }
}

pub fn target_value(pos: &Position, target: u64) -> i16 {
    if pos.pawns & target != 0 {
        return PIECE_VALUES[0];
    }

    if pos.knights & target != 0 {
        return PIECE_VALUES[1];
    }

    if pos.bishops & target != 0 {
        return PIECE_VALUES[2];
    }

    if pos.rooks & target != 0 {
        return PIECE_VALUES[3];
    }

    if pos.queens & target != 0 {
        return PIECE_VALUES[4];
    }

    0
}

pub fn tactical_score(pos: &Position, m: Move) -> i16 {
    if m.code == 8 {
        return PIECE_VALUES[0];
    }

    let target_value = target_value(pos, m.target);
    let self_value = PIECE_VALUES[m.tier as usize];

    if m.code != 0 && m.code <= 4 {
        return PIECE_VALUES[m.code as usize] + target_value;
    }

    if m.target & pos.state.attack_mask == 0 {
        return target_value;
    }

    if self_value <= target_value {
        return target_value - self_value;
    }

    -1
}

pub fn quiet_score(pos: &Position, m: Move) -> i16 {
    if m.code == 5 {
        return 160;
    }

    if m.code == 6 || m.code == 7 {
        return 400;
    }

    if m.origin & pos.state.attack_mask != 0 {
        return PIECE_VALUES[m.tier as usize];
    }

    if m.target & pos.state.attack_mask == 0 && m.tier != 5 {
        return PIECE_VALUES[m.tier as usize] >> 1;
    }

    0
}