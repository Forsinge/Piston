use std::io::stdin;
use std::ops::Deref;
use std::thread;
use crate::bitboard::BITS;
use crate::output::string_to_index;
use crate::position::{Move, Position, STARTPOS_FEN};
use crate::state::{EngineState, MAX_MOVE_COUNT};
use crate::output::Display;
use crate::search::perft;

const CMD_ERR: &str = "Error parsing command.";

pub fn uci_loop() {
    println!("Piston dev build");
    println!();

    let es = &mut EngineState::new();

    loop {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let tokens: Vec<&str> = buffer.trim().split(" ").collect();

        if let Some(&token) = tokens.get(0) {
            match token {
                "go" => handle_go(es, tokens),
                "move" => handle_move(es, tokens),
                "position" => handle_position(es, tokens),
                _ => handle_info_cmd(es, tokens)
            }
        }
    }
}

pub fn handle_info_cmd(es: &EngineState, tokens: Vec<&str>) {
    match tokens.get(0).unwrap().deref() {
        "uci" => {
            println!("id name Piston Dev");
            println!("id author Carl");
            println!("uciok");
        }

        "isready" => println!("readyok"),

        "state" => es.root.state.print(),

        "d" => es.root.print(),

        "pm" => {
            let pos = &mut es.root.clone();
            let mut list = [Move::default(); MAX_MOVE_COUNT];
            pos.generate(&mut list[0..MAX_MOVE_COUNT]);
            pos.print_moves(&mut list[0..MAX_MOVE_COUNT]);
        }

        "exit" => std::process::exit(0),

        _ => {}
    }
}

pub fn handle_position(es: &mut EngineState, tokens: Vec<&str>) {
    let mut pos = es.root.clone();
    let mut iter = tokens.into_iter();
    iter.next();

    if let Some(first_token) = iter.next() {
        match first_token {

            "startpos" => {
                pos = Position::build_from_fen(STARTPOS_FEN);
                iter.next();
            }

            "fen" => {
                let mut fen = String::new();
                while let Some(token) = iter.next() {
                    if token == "moves" {
                        break
                    } else {
                        fen.push_str(token);
                        fen.push_str(" ");
                    }
                }
                pos = Position::build_from_fen(fen.trim());
            }
            _ => {}
        }

        while let Some(token) = iter.next() {
            let m = parse_move(&pos, token);
            pos = pos.make_move(m);
        }
    }
    es.root = pos;
    es.root.state.move_ptr = 0;
}

pub fn handle_move(es: &mut EngineState, tokens: Vec<&str>) {
    let mut pos = es.root.clone();
    if let Some(&m) = tokens.get(1) {
        pos = pos.make_move(parse_move(&pos, m));
    }
    es.root = pos;
    es.root.state.move_ptr = 0;
}

pub fn handle_go(es: &mut EngineState, tokens: Vec<&str>) {
    let mut iter = tokens.into_iter();
    iter.next();

    while let Some(token) = iter.next() {
        match token {

            "perft" => {
                let root_clone = es.root.clone();
                let ss_arc = es.search_state.clone();
                let depth = iter.next().unwrap_or("1").parse::<u8>().unwrap();

                thread::spawn(move || {
                    let lock_result = ss_arc.try_lock();
                    if lock_result.is_ok() {
                        let mut state = lock_result.unwrap();

                        state.root = root_clone;
                        state.max_depth = depth;
                        state.node_count = 0;

                        perft(&mut state);
                    } else {
                        println!("A search is already in progress!");
                    }
                });
            }

            _ => {}
        }
    }
}


pub fn parse_move(root: &Position, m: &str) -> Move {
    let origin = BITS[string_to_index(&m[0..2])];
    let target = BITS[string_to_index(&m[2..4])];
    let tier = root.square_tier(origin) as u8;
    let mut code = 0;
    if tier == 0 {
        if target & root.state.en_passant != 0 {
            code = 8;
        } else if origin == target << 2 || origin == target >> 2 {
            code = 5;
        } else if m.len() > 4 {
            let promos = " nbrq";
            code = promos.find(&m[4..5]).unwrap() as u8;
        }
    } else if tier == 5 {
        if origin == target >> 2 {
            code = 7;
        } else if origin == target << 2 {
            code = 6;
        }
    }
    Move { origin, target, tier, code }
}

/*
pub fn handle_command(mut state: MutexGuard<EngineState>, tokens: &Vec<&str>) {
    if tokens.len() == 1 {
        match tokens.get(0).unwrap().deref() {
            "uci" => {
                println!("id name Piston 0.7");
                println!("id author Carl");
                println!("uciok");
            }

            "isready" => println!("readyok"),

            "state" => state.root.state.print(),

            "d" => state.root.print(),

            "pm" => {
                let pos = &mut state.root.clone();
                pos.generate(&mut state.move_table);
                pos.print_moves(&mut state.move_table);
            }

            "exit" => std::process::exit(0),

            _ => {}
        }
    }
    if tokens.len() >= 2 {
        match tokens.get(0).unwrap().deref() {
            "position" => {
                match tokens.get(1).unwrap().deref() {
                    
                    "startpos" => {
                        let pos = Position::build_from_fen(STARTPOS_FEN);
                        state.root = pos;
                    }

                    _ => {
                        let fen_vec = Vec::from_iter(tokens[2..].iter().cloned());
                        let fen = fen_vec.join(" ");
                        let pos = Position::build_from_fen(fen.as_str());
                        state.root = pos;
                    }
                }
            }

            "move" => {
                let m = parse_move(&state.root, tokens.get(1).unwrap());
                let mut pos = state.root.clone().make_move(m);
                pos.state.move_ptr = 0;
                state.root = pos;
            }

            "go" => {
                match tokens.get(1).unwrap().deref() {
                    "perft" => {
                        state.max_depth = tokens.get(2).unwrap().deref().parse::<u8>().unwrap();
                        perft(state);
                    }
                    _ => {}
                }
            }

            _ => {},
        }
    }
}

 */