use std::io::stdin;
use std::ops::Deref;
use crate::bitboard::BITS;
use crate::output::string_to_index;
use crate::position::{Move, Position, STARTPOS_FEN};
use crate::state::EngineState;
use crate::output::Display;
use crate::search::perft;

const CMD_ERR: &str = "Error parsing command.";

pub fn uci_loop() {
    let es = &mut EngineState::new();
    println!("Piston dev build");
    println!();

    loop {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let tokens: Vec<&str> = buffer.trim().split(" ").collect();
        handle_command(es, &tokens);
    }
}

pub fn parse_move(es: &EngineState, m: &str) -> Move {
    let origin = BITS[string_to_index(&m[0..2])];
    let target = BITS[string_to_index(&m[2..4])];
    let tier = es.root.square_tier(origin) as u8;
    let mut code = 0;
    if tier == 0 {
        if target & es.root.state.en_passant != 0 {
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

pub fn handle_command(es: &mut EngineState, tokens: &Vec<&str>) {
    if tokens.len() == 1 {
        match tokens.get(0).unwrap().deref() {
            "uci" => {
                println!("id name Piston 0.7");
                println!("id author Carl");
                println!("uciok");
            }

            "isready" => println!("readyok"),

            "state" => es.root.state.print(),

            "d" => es.root.print(),

            "pm" => {
                let pos = &mut es.root.clone();
                pos.generate(es);
                pos.print_moves(es);
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
                        es.root = pos;
                    }

                    _ => {
                        let fen_vec = Vec::from_iter(tokens[2..].iter().cloned());
                        let fen = fen_vec.join(" ");
                        let pos = Position::build_from_fen(fen.as_str());
                        es.root = pos;
                    }
                }
            }

            "move" => {
                let m = parse_move(es, tokens.get(1).unwrap());
                let mut pos = es.root.clone().make_move(m);
                pos.state.move_ptr = 0;
                es.root = pos;
            }

            "go" => {
                match tokens.get(1).unwrap().deref() {
                    "perft" => {
                        es.set_depth(tokens.get(2).unwrap().deref().parse::<u8>().unwrap());
                        perft(es, &mut es.root.clone());
                    }
                    _ => {}
                }
            }

            _ => {},
        }
    }
}