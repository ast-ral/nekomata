use crate::board::{Board, Player};
use crate::eval::Score;

pub(crate) fn minimax(board: Board, player: Player, level: usize) -> Score {
	if level == 0 {
		return board.eval();
	}

	let turns = board.get_possible_turns(player);

	if turns.len() == 0 {
		return if board.king_in_check(player) {
			Score::Checkmate {
				winning_player: player.flipped(),
				in_moves: 0,
			}
		} else {
			Score::Stalemate
		};
	}

	let mut score = Score::worst_for_player(player);

	for turn in turns {
		let mut board = board.clone();
		board.perform_turn(turn);

		let subscore = minimax(board, player.flipped(), level - 1).add_turn();

		score = match player {
			Player::White => score.max(subscore),
			Player::Black => score.min(subscore),
		};
	}

	score
}
