use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::basics::{EnPassantState, File, Layer, Piece, Rank, Square, State};

#[derive(Clone, Copy)]
pub(crate) struct Ply {
	from: Square,
	to: Square,
	promotion: Option<Piece>,
}

#[derive(Debug)]
pub(crate) struct PlyParseError;

impl FromStr for Ply {
	type Err = PlyParseError;

	fn from_str(ply: &str) -> Result<Self, PlyParseError> {
		let from = ply[0 .. 2].parse().ok().ok_or(PlyParseError)?;
		let to = ply[2 .. 4].parse().ok().ok_or(PlyParseError)?;

		let promotion = match &ply[4 ..] {
			"" => None,
			"b" => Some(Piece::Bishop),
			"n" => Some(Piece::Knight),
			"r" => Some(Piece::Rook),
			"q" => Some(Piece::Queen),
			_ => return Err(PlyParseError),
		};

		Ok(Self { from, to, promotion })
	}
}

impl Display for Ply {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{}{}", self.from, self.to)?;

		if let Some(piece) = self.promotion {
			write!(f, "{}", piece.representation())?;
		}

		Ok(())
	}
}

// I kinda hate this function
pub(crate) fn apply_ply(state: &mut State, ply: Ply) {
	let from = Layer::from_square(ply.from);
	let to = Layer::from_square(ply.to);

	let player_to_move = state.to_move;

	let mut new_en_passant = EnPassantState::empty();

	for piece in Piece::ALL {
		if (state[player_to_move][piece] & from).is_empty() {
			continue;
		}

		let destination_piece = ply.promotion.unwrap_or(piece);

		state[player_to_move][piece] &= !from;
		state[player_to_move][destination_piece] |= to;

		state[player_to_move.flipped()].perform_capture(to);

		if piece == Piece::King {
			state[player_to_move].can_castle_kingside = false;
			state[player_to_move].can_castle_queenside = false;
		}

		let white_kingside = (
			Square::from_rank_and_file(File::E, Rank::N1),
			Square::from_rank_and_file(File::G, Rank::N1),
			Piece::King,
		);
		if (ply.from, ply.to, piece) == white_kingside {
			state.white.rook &= !Layer::from_square(Square::from_rank_and_file(File::H, Rank::N1));
			state.white.rook |= Layer::from_square(Square::from_rank_and_file(File::F, Rank::N1));
		}

		let white_queenside = (
			Square::from_rank_and_file(File::E, Rank::N1),
			Square::from_rank_and_file(File::C, Rank::N1),
			Piece::King,
		);
		if (ply.from, ply.to, piece) == white_queenside {
			state.white.rook &= !Layer::from_square(Square::from_rank_and_file(File::A, Rank::N1));
			state.white.rook |= Layer::from_square(Square::from_rank_and_file(File::D, Rank::N1));
		}

		let black_kingside = (
			Square::from_rank_and_file(File::E, Rank::N8),
			Square::from_rank_and_file(File::G, Rank::N8),
			Piece::King,
		);
		if (ply.from, ply.to, piece) == black_kingside {
			state.black.rook &= !Layer::from_square(Square::from_rank_and_file(File::H, Rank::N8));
			state.black.rook |= Layer::from_square(Square::from_rank_and_file(File::F, Rank::N8));
		}

		let black_queenside = (
			Square::from_rank_and_file(File::E, Rank::N8),
			Square::from_rank_and_file(File::C, Rank::N8),
			Piece::King,
		);
		if (ply.from, ply.to, piece) == black_queenside {
			state.black.rook &= !Layer::from_square(Square::from_rank_and_file(File::A, Rank::N8));
			state.black.rook |= Layer::from_square(Square::from_rank_and_file(File::D, Rank::N8));
		}

		if piece == Piece::Pawn && (to & state.en_passant.move_to).is_nonempty() {
			let capture = state.en_passant.capture;
			state[player_to_move.flipped()].perform_capture(capture);
		}

		if piece == Piece::Pawn
			&& Some(ply.to)
				== ply
					.from
					.pawn_direction(player_to_move)
					.and_then(|x| x.pawn_direction(player_to_move))
		{
			new_en_passant = EnPassantState {
				move_to: Layer::from_square(ply.from.pawn_direction(player_to_move).unwrap()),
				capture: to,
			};
		}

		break;
	}

	if (state[player_to_move].rook & Layer::KINGSIDE_ROOK_MASK).is_empty() {
		state[player_to_move].can_castle_kingside = false;
	}
	if (state[player_to_move].rook & Layer::QUEENSIDE_ROOK_MASK).is_empty() {
		state[player_to_move].can_castle_queenside = false;
	}
	if (state[player_to_move.flipped()].rook & Layer::KINGSIDE_ROOK_MASK).is_empty() {
		state[player_to_move.flipped()].can_castle_kingside = false;
	}
	if (state[player_to_move.flipped()].rook & Layer::QUEENSIDE_ROOK_MASK).is_empty() {
		state[player_to_move.flipped()].can_castle_queenside = false;
	}

	state.to_move = player_to_move.flipped();
	state.en_passant = new_en_passant;
}

pub(crate) fn diff_states(from_state: &State, to_state: &State) -> Ply {
	let player_to_move = from_state.to_move;

	let mut from = None;
	let mut to = None;

	for piece in Piece::ALL.into_iter().rev() {
		let missing = from_state[player_to_move][piece] & !to_state[player_to_move][piece];

		if missing.is_empty() {
			continue;
		}

		from = Some((missing.iter_squares().next().unwrap(), piece));

		break;
	}

	for piece in Piece::ALL.into_iter().rev() {
		let missing = to_state[player_to_move][piece] & !from_state[player_to_move][piece];

		if missing.is_empty() {
			continue;
		}

		to = Some((missing.iter_squares().next().unwrap(), piece));

		break;
	}

	let (from, from_piece) = from.unwrap();
	let (to, to_piece) = to.unwrap();

	let mut promotion = None;

	if from_piece != to_piece {
		promotion = Some(to_piece);
	}

	Ply { from, to, promotion }
}
