use crate::basics::{Layer, Piece, PlayerState, State};
use crate::score::Score;

fn count_material(state: &PlayerState) -> f64 {
	let mut out = 0.0;

	for piece in [Piece::Pawn, Piece::Bishop, Piece::Knight, Piece::Rook, Piece::Queen] {
		let value = match piece {
			Piece::Pawn => 1.0,
			Piece::Bishop => 3.0,
			Piece::Knight => 3.0,
			Piece::Rook => 5.0,
			Piece::Queen => 9.0,
			Piece::King => unreachable!(),
		};

		out += value * f64::from(state[piece].num_squares());
		out += value * f64::from((state[piece] & Layer::MIDSCREEN).num_squares()) * 0.2;
	}

	out
}

pub(crate) fn static_eval(state: &State) -> Score {
	Score::Heuristic {
		value: count_material(&state[state.to_move]) - count_material(&state[state.to_move.flipped()]),
	}
}
