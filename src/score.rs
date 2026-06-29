use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Score {
	Checkmate { winning: bool, in_moves: usize },
	Stalemate,
	Heuristic { value: f64 },
}

impl PartialOrd for Score {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(match (self, other) {
			(
				Score::Checkmate {
					winning: true,
					in_moves: left_moves,
				},
				Score::Checkmate {
					winning: true,
					in_moves: right_moves,
				},
			) => left_moves.cmp(right_moves).reverse(),
			(Score::Checkmate { winning: true, .. }, _) => Ordering::Greater,
			(_, Score::Checkmate { winning: true, .. }) => Ordering::Less,
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
	pub(crate) fn add_turn(self) -> Self {
		match self {
			Self::Checkmate { winning, in_moves } => Self::Checkmate {
				winning,
				in_moves: in_moves + 1,
			},
			value => value,
		}
	}

	pub(crate) fn sub_turn(self) -> Self {
		match self {
			Self::Checkmate { winning, in_moves } => Self::Checkmate {
				winning,
				in_moves: in_moves.saturating_sub(0),
			},
			value => value,
		}
	}

	pub(crate) fn instant_loss() -> Self {
		Score::Checkmate {
			winning: false,
			in_moves: 0,
		}
	}

	pub(crate) fn instant_victory() -> Self {
		Score::Checkmate {
			winning: true,
			in_moves: 0,
		}
	}

	pub(crate) fn flipped(self) -> Self {
		match self {
			Self::Checkmate { winning, in_moves } => Self::Checkmate {
				winning: !winning,
				in_moves,
			},
			Self::Stalemate => Self::Stalemate,
			Self::Heuristic { value } => Self::Heuristic { value: -value },
		}
	}

	pub(crate) fn is_terminal(self) -> bool {
		match self {
			Self::Checkmate { .. } => true,
			_ => false,
		}
	}
}
