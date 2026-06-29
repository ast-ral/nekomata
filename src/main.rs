#[cfg(not(target_arch = "x86_64"))]
compile_error!("only x86-64 support for the moment, sorry!");

mod basics;
mod ply_interface;
mod score;
mod search;
mod static_eval;

use std::io::stdin;

use crate::basics::{Player, State};
use crate::ply_interface::{apply_ply, diff_states};
use crate::search::{SearchParameters, SearchResult, TimeControl, search};

fn main() {
	let stdin = stdin().lines().map(|x| x.expect("error while reading"));

	let mut state = State::initial();

	for line in stdin {
		let mut line = line.split(" ");
		let command = line.next().unwrap();
		let mut arguments: Vec<_> = line.collect();

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
					let maybe_search_result = search(
						SearchParameters::standard_search(state.clone(), depth),
						&mut time_control,
					);
					if let Some(SearchResult { best_child, score }) = maybe_search_result {
						println!("info depth {depth}");
						println!("info time {}", time_control.elapsed());
						println!("info nodes {}", time_control.nodes_count());
						best_child_overall = Some(best_child.unwrap());
						time_control.some_move_found();

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
				assert_eq!(arguments[0], "startpos", "other modes unsupported");

				state = State::initial();

				if let Some(&"moves") = arguments.get(1) {
					for ply in arguments.drain(2 ..) {
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
