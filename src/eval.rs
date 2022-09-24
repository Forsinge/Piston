use std::num::NonZeroU64;
use piston::index;
use crate::movegen::{pseudo_bishop};
use crate::position::Position;

pub const TERMINATE: i16 = 10001;
pub const LOSS: i16 = -10000;
pub const DRAW: i16 = 0;
pub const PIECE_VALUES: [i16; 6] = [100, 300, 300, 500, 1000, 0];

pub fn eval(pos: &Position) -> i16 {

    let mut player_mobility = 0;
    let mut enemy_mobility = 0;

    let mut player_bishops = pos.player & (pos.bishops | pos.queens);
    while player_bishops != 0 {
        let piece = player_bishops & (!player_bishops + 1);
        let index = index!(NonZeroU64::new(piece).unwrap());
        player_mobility += pseudo_bishop(piece, pos.all, index).count_ones();
        player_bishops &= player_bishops - 1;
    }

    let mut player_rooks = pos.player & pos.rooks;
    while player_rooks != 0 {
        let piece = player_rooks & (!player_rooks + 1);
        let index = index!(NonZeroU64::new(piece).unwrap());
        player_mobility += pseudo_bishop(piece, pos.all, index).count_ones();
        player_rooks &= player_rooks - 1;
    }

    let mut enemy_bishops = pos.enemy & (pos.bishops | pos.queens);
    while enemy_bishops != 0 {
        let piece = enemy_bishops & (!enemy_bishops + 1);
        let index = index!(NonZeroU64::new(piece).unwrap());
        enemy_mobility += pseudo_bishop(piece, pos.all, index).count_ones();
        enemy_bishops &= enemy_bishops - 1;
    }

    let mut enemy_rooks = pos.enemy & pos.rooks;
    while enemy_rooks != 0 {
        let piece = enemy_rooks & (!enemy_rooks + 1);
        let index = index!(NonZeroU64::new(piece).unwrap());
        enemy_mobility += pseudo_bishop(piece, pos.all, index).count_ones();
        enemy_rooks &= enemy_rooks - 1;
    }

    return pos.state.material_balance + player_mobility as i16 - enemy_mobility as i16;
}