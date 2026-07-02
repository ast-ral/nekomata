#[cfg(not(target_arch = "x86_64"))]
compile_error!("only x86-64 support for the moment, sorry!");

mod basics;
mod ply_interface;
mod score;
mod search;
mod static_eval;

use std::io::stdin;

use crate::basics::{Player, State};
use crate::score::Score;
use crate::ply_interface::{apply_ply, diff_states};
use crate::search::{SearchParameters, SearchResult, TimeControl, PvTable, search};

fn main() {
	let stdin = stdin().lines().map(|x| x.expect("error while reading"));

	let mut state = State::initial();

	for line in stdin {
		let mut line = line.split(" ");
		let command = line.next().unwrap();
		let arguments: Vec<_> = line.collect();

		match command {
			"uci" => {
				println!("id name nekomata");
				println!("id author ast-ral");

				println!("uciok");
			},
			"debug" => {
				// silently ignore
			},
			"isready" => {
				println!("readyok");
			},
			"setoption" => {
				// we don't have any options atm
			},
			"ucinewgame" => {
				// no special handling atm
			},
			"go" => {
				let mut time: u64 = 0;
				let mut increment: u64 = 0;

				let mut arguments = arguments.into_iter();

				loop {
					match arguments.next() {
						Some("movetime") => increment = arguments.next().unwrap().parse().unwrap(),
						Some("wtime") => {
							if state.to_move == Player::White {
								time = arguments.next().unwrap().parse().unwrap();
							}
						},
						Some("winc") => {
							if state.to_move == Player::White {
								increment = arguments.next().unwrap().parse().unwrap();
							}
						},
						Some("btime") => {
							if state.to_move == Player::Black {
								time = arguments.next().unwrap().parse().unwrap();
							}
						},
						Some("binc") => {
							if state.to_move == Player::Black {
								increment = arguments.next().unwrap().parse().unwrap();
							}
						},
						None => break,
						_ => {},
					}
				}

				let time_usage = time.saturating_sub(increment) / 50 + increment;
				let mut time_control = TimeControl::ms_from_now(time_usage);

				let mut best_child_overall = None;

				for depth in 1 .. {
					let mut pv_table = PvTable::new();

					let maybe_search_result = search(
						SearchParameters::standard_search(state.clone(), depth, &mut pv_table),
						&mut time_control,
					);
					if let Some(SearchResult { score, pv_found }) = maybe_search_result {
						assert!(pv_found);
						time_control.some_move_found();

						print!("info depth {depth} time {time} nodes {nodes}",
							time = time_control.elapsed(),
							nodes = time_control.nodes_count(),
						);

						match score {
							Score::Checkmate { winning: true, in_moves } => {
								print!(" score mate {}", in_moves / 2 + 1);
							},
							Score::Checkmate { winning: false, in_moves } => {
								print!(" score mate -{}", in_moves / 2);
							},
							Score::Stalemate => {
								print!(" score cp 0");
							},
							Score::Heuristic { value } => {
								print!(" score cp {}", (value * 100.0).round() as i64);
							},
						}

						let pv = pv_table.extract_pv();
						best_child_overall = Some(pv[0].clone());

						print!(" pv");

						let mut current_state = &state;

						for state in pv {
							print!(" {}", diff_states(current_state, state));
							current_state = &state;
						}

						println!();

						if score.is_terminal() {
							break;
						}
					} else {
						break;
					}
				}

				let ply = diff_states(&state, &best_child_overall.unwrap());

				println!("bestmove {ply}");
			},
			"ponderhit" => {
				// we don't do pondering I don't think
			},
			"position" => {
				let mut arguments = arguments.into_iter();

				state = match arguments.next().unwrap() {
					"startpos" => State::initial(),
					"fen" => {
						let pieces: Vec<_> = (&mut arguments).take(6).collect();
						let fen = pieces.join(" ");

						State::from_fen(&fen)
					},
					_ => panic!("unrecognized subcommand after position"),
				};

				if let Some("moves") = arguments.next() {
					for ply in arguments {
						let ply = ply.parse().unwrap();
						apply_ply(&mut state, ply);
					}
				}
			},
			"quit" => {
				break;
			},
			_ => {
				eprintln!("unrecognized command: {:?}", command);
			},
		}
	}
}
