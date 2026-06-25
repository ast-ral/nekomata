mod board;
mod eval;
mod minimax;

use std::io::stdin;

use crate::board::{Board, Player, Turn};
use crate::eval::Score;
use crate::minimax::minimax;

fn main() {
	let stdin = stdin().lines().map(|x| x.expect("error while reading"));

	let mut board = Board::new();
	let mut current_player = Player::White;

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
				dbg!(arguments);

				let possible_turns = board.get_possible_turns(current_player);

				let mut best_turn = None;
				let mut best_eval = Score::worst_for_player(current_player);

				for turn in possible_turns {
					let mut board = board.clone();
					board.perform_turn(turn);

					let eval = minimax(board, current_player.flipped(), 2);

					let better = match current_player {
						Player::White => eval > best_eval,
						Player::Black => eval < best_eval,
					};

					if better {
						best_turn = Some(turn);
						best_eval = eval;
					}
				}

				println!("bestmove {}", best_turn.unwrap().to_uci());
			},
			"ponderhit" => {
				// we don't do pondering I don't think
			},
			"position" => {
				assert_eq!(arguments[0], "startpos", "other modes unsupported");

				board = Board::new();
				current_player = Player::White;

				if let Some(&"moves") = arguments.get(1) {
					for turn in arguments.drain(2 ..) {
						board.perform_turn(Turn::from_uci(turn));
						current_player = current_player.flipped();
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
