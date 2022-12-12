use std::io::stdin;
use std::ops::{Deref};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use crate::bitboard::BITS;
use crate::output::string_to_index;
use crate::position::{Move, Position, STARTPOS_FEN};
use crate::state::{EngineState, MAX_MOVE_COUNT, SearchStats};
use crate::output::Display;
use crate::search::{perft, pvs};

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

        "stats" => {
            let ss_arc = es.search_state.clone();
            let lock_result = ss_arc.try_lock();
            if lock_result.is_ok() {
                lock_result.unwrap().stats.print();
            } else {
                println!("Cannot access statistics during search.");
            }
        }

        "d" => es.root.print(),

        "pm" => {
            let pos = &mut es.root.clone();
            let mut list = [Move::default(); MAX_MOVE_COUNT];
            pos.generate(&mut list[0..MAX_MOVE_COUNT]);
            pos.print_moves(&mut list[0..MAX_MOVE_COUNT]);
        }

        "pt" => {
            let pos = &mut es.root.clone();
            let mut list = [Move::default(); MAX_MOVE_COUNT];
            pos.generate_tactical(&mut list[0..MAX_MOVE_COUNT]);
            pos.print_moves(&mut list[0..MAX_MOVE_COUNT]);
        }

        "pq" => {
            let pos = &mut es.root.clone();
            let mut list = [Move::default(); MAX_MOVE_COUNT];
            pos.generate_quiet(&mut list[0..MAX_MOVE_COUNT]);
            pos.print_moves(&mut list[0..MAX_MOVE_COUNT]);
        }

        "exit" | "quit" => {
            es.terminate.store(true, Relaxed);
            std::process::exit(0);
        }


        "stop" => es.terminate.store(true, Relaxed),

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

    'outer: while let Some(token) = iter.next() {
        match token {

            "perft" => {
                let depth = iter.next().unwrap_or("1").parse::<u8>().unwrap();

                // temporary fix
                if depth <= 1 {
                    let pos = &mut es.root.clone();
                    let mut list = [Move::default(); MAX_MOVE_COUNT];
                    pos.generate(&mut list[0..MAX_MOVE_COUNT]);
                    println!("Nodes searched: {}", pos.state.move_cnt)
                } else {
                    let root_clone = es.root.clone();
                    let ss_arc = es.search_state.clone();
                    thread::spawn(move || {
                        let lock_result = ss_arc.try_lock();
                        if lock_result.is_ok() {
                            let mut state = lock_result.unwrap();

                            state.root = root_clone;
                            state.max_depth = depth;
                            state.stats = SearchStats::new();

                            perft(&mut state);
                        } else {
                            println!("A search is already in progress!");
                        }
                    });
                }
            }

            _ => {
                let (sender, receiver) = mpsc::channel();
                let root_clone = es.root.clone();
                let ss_arc = es.search_state.clone();

                thread::spawn(move || {
                    let lock_result = ss_arc.try_lock();
                    if lock_result.is_ok() {
                        let mut state = lock_result.unwrap();

                        state.root = root_clone;
                        state.stats = SearchStats::new();

                        pvs(&mut state, &receiver);
                    } else {
                        println!("A search is already in progress!");
                    }
                });

                let terminate_arc = es.terminate.clone();
                thread::spawn(move || {
                    let clock = Instant::now();
                    while clock.elapsed().as_millis() < 4000 && !terminate_arc.load(Relaxed) {}
                    sender.send(true).expect("Search thread has terminated unexpectedly.");
                    terminate_arc.store(false, Relaxed);
                });

                break 'outer;
            }
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