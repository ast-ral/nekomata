#[cfg(not(target_arch = "x86_64"))]
compile_error!("only x86-64 support for the moment, sorry!");

mod basics;
mod ply_interface;
mod score;
mod search;
mod static_eval;

use std::io::stdin;

use crate::basics::State;
use crate::ply_interface::{apply_ply, diff_states};
use crate::search::{SearchParameters, SearchResult, search};

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
				let SearchResult { best_child, .. } = search(SearchParameters::standard_search(state.clone(), 5));
				let ply = diff_states(&state, &best_child.unwrap());

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
