use std::cmp::Ordering;

use crate::board::{Board, Piece, Player};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Score {
	Checkmate { winning_player: Player, in_moves: usize },
	Stalemate,
	Heuristic { value: f64 },
}

impl PartialOrd for Score {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(match (self, other) {
			(
				Score::Checkmate {
					winning_player: Player::White,
					in_moves: left_moves,
				},
				Score::Checkmate {
					winning_player: Player::White,
					in_moves: right_moves,
				},
			) => left_moves.cmp(right_moves).reverse(),
			(
				Score::Checkmate {
					winning_player: Player::White,
					..
				},
				_,
			) => Ordering::Greater,
			(
				_,
				Score::Checkmate {
					winning_player: Player::White,
					..
				},
			) => Ordering::Less,
			(Score::Heuristic { value: left_value }, Score::Heuristic { value: right_value })
				if left_value.total_cmp(&0.0).is_gt() && right_value.total_cmp(&0.0).is_gt() =>
			{
				left_value.total_cmp(right_value)
			},
			(Score::Heuristic { value }, _) if value.total_cmp(&0.0).is_gt() => Ordering::Greater,
			(_, Score::Heuristic { value }) if value.total_cmp(&0.0).is_gt() => Ordering::Less,
			(Score::Stalemate, Score::Stalemate) => Ordering::Equal,
			(Score::Stalemate, _) => Ordering::Greater,
			(_, Score::Stalemate) => Ordering::Less,
			(Score::Heuristic { value: left_value }, Score::Heuristic { value: right_value }) => {
				left_value.total_cmp(right_value)
			},
			(Score::Heuristic { .. }, _) => Ordering::Greater,
			(_, Score::Heuristic { .. }) => Ordering::Less,
			(
				Score::Checkmate {
					in_moves: left_moves, ..
				},
				Score::Checkmate {
					in_moves: right_moves, ..
				},
			) => left_moves.cmp(right_moves),
		})
	}
}

impl Ord for Score {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap()
	}
}

impl PartialEq for Score {
	fn eq(&self, other: &Self) -> bool {
		matches!(self.partial_cmp(other), Some(Ordering::Equal))
	}
}

impl Eq for Score {}

impl Score {
	pub(crate) fn add_turn(self) -> Score {
		match self {
			Self::Checkmate {
				winning_player,
				in_moves,
			} => Self::Checkmate {
				winning_player,
				in_moves: in_moves + 1,
			},
			value => value,
		}
	}

	pub(crate) fn worst_for_player(player: Player) -> Score {
		Score::Checkmate {
			winning_player: player.flipped(),
			in_moves: 0,
		}
	}
}

impl Board {
	fn count_material(&self, player: Player) -> u64 {
		let mut out = 0;

		for square in self.get_allied_piece_squares(player) {
			let (_, piece) = self.query(square).unwrap();

			out += match piece {
				Piece::Pawn => 1,
				Piece::Bishop => 3,
				Piece::Knight => 3,
				Piece::Rook => 5,
				Piece::Queen => 9,
				Piece::King => 0,
			};
		}

		out
	}

	pub(crate) fn eval(&self) -> Score {
		let position =
			self.get_possible_turns(Player::White).len() as f64 - self.get_possible_turns(Player::Black).len() as f64;
		let material = self.count_material(Player::White) as f64 - self.count_material(Player::Black) as f64;

		Score::Heuristic {
			value: position * 0.1 + material,
		}
	}
}
