use std::fmt::{Debug, Error, Formatter};
use std::ops::{Index, IndexMut};

use crate::basics::castling::CastlingType;
use crate::basics::layer::Layer;
use crate::basics::piece::Piece;
use crate::basics::player::Player;
use crate::basics::square::{File, Rank, Square};

#[derive(Clone)]
pub(crate) struct State {
	pub(crate) white: PlayerState,
	pub(crate) black: PlayerState,
	pub(crate) to_move: Player,
	pub(crate) en_passant: EnPassantState,
}

#[derive(Clone)]
pub(crate) struct PlayerState {
	pub(crate) pawn: Layer,
	pub(crate) bishop: Layer,
	pub(crate) knight: Layer,
	pub(crate) rook: Layer,
	pub(crate) queen: Layer,
	pub(crate) king: Layer,
	pub(crate) can_castle_kingside: bool,
	pub(crate) can_castle_queenside: bool,
}

#[derive(Clone)]
pub(crate) struct EnPassantState {
	pub(crate) move_to: Layer,
	pub(crate) capture: Layer,
}

impl State {
	pub(crate) fn initial() -> Self {
		use crate::basics::square::File::{A, B, C, D, E, F, G, H};
		use crate::basics::square::Rank::{N1, N2, N7, N8};

		// kinda weird to have this nested but oh well
		fn layer(file: File, rank: Rank) -> Layer {
			Layer::from_square(Square::from_rank_and_file(file, rank))
		}

		Self {
			white: PlayerState {
				pawn: {
					let mut out = Layer::empty();

					for file in File::ALL {
						out |= layer(file, N2);
					}

					out
				},
				bishop: layer(C, N1) | layer(F, N1),
				knight: layer(B, N1) | layer(G, N1),
				rook: layer(A, N1) | layer(H, N1),
				queen: layer(D, N1),
				king: layer(E, N1),
				can_castle_kingside: true,
				can_castle_queenside: true,
			},
			black: PlayerState {
				pawn: {
					let mut out = Layer::empty();

					for file in File::ALL {
						out |= layer(file, N7);
					}

					out
				},
				bishop: layer(C, N8) | layer(F, N8),
				knight: layer(B, N8) | layer(G, N8),
				rook: layer(A, N8) | layer(H, N8),
				queen: layer(D, N8),
				king: layer(E, N8),
				can_castle_kingside: true,
				can_castle_queenside: true,
			},
			en_passant: EnPassantState {
				move_to: Layer::empty(),
				capture: Layer::empty(),
			},
			to_move: Player::White,
		}
	}

	pub(crate) fn children(&self) -> Vec<State> {
		let mut out = Vec::new();

		let own_state = self[self.to_move].clone();
		let opponent_state = self[self.to_move.flipped()].clone();

		let mut push_state = |mut own_child_state: PlayerState, mut opponent_child_state: PlayerState, en_passant| {
			let own_rook_diff = own_child_state.rook ^ own_state.rook;
			let opponent_rook_diff = own_child_state.rook ^ opponent_state.rook;

			if (own_rook_diff & Layer::KINGSIDE_ROOK_MASK).is_nonempty() {
				own_child_state.can_castle_kingside = false;
			}
			if (own_rook_diff & Layer::QUEENSIDE_ROOK_MASK).is_nonempty() {
				own_child_state.can_castle_queenside = false;
			}
			if (opponent_rook_diff & Layer::KINGSIDE_ROOK_MASK).is_nonempty() {
				opponent_child_state.can_castle_kingside = false;
			}
			if (opponent_rook_diff & Layer::QUEENSIDE_ROOK_MASK).is_nonempty() {
				opponent_child_state.can_castle_queenside = false;
			}

			let mut child = match self.to_move {
				Player::White => Self {
					white: own_child_state,
					black: opponent_child_state,
					to_move: Player::White,
					en_passant,
				},
				Player::Black => Self {
					white: opponent_child_state,
					black: own_child_state,
					to_move: Player::Black,
					en_passant,
				},
			};

			if child.in_check() {
				return;
			}

			let promoting = child[child.to_move].pawn & Layer::promotions(child.to_move);

			if promoting.is_nonempty() {
				for piece in [Piece::Bishop, Piece::Knight, Piece::Rook, Piece::Queen] {
					let mut child = child.clone();
					let to_move = child.to_move;

					child[to_move].pawn &= !promoting;
					child[to_move][piece] |= promoting;

					child.to_move = to_move.flipped();
					out.push(child);
				}
			} else {
				child.to_move = child.to_move.flipped();
				out.push(child);
			}
		};

		let own_pieces = own_state.all_pieces();
		let opponent_pieces = opponent_state.all_pieces();
		let all_pieces = own_pieces | opponent_pieces;

		for pawn in own_state.pawn.as_singles() {
			let pawn_removed = own_state.pawn & !pawn;
			let pawn_forward = pawn.pawn_forward(self.to_move) & !all_pieces;
			let pawn_captures = pawn.pawn_captures(self.to_move) & !own_pieces;
			let en_passant = pawn_captures & self.en_passant.move_to;
			let pawn_captures = pawn_captures & opponent_pieces;
			let pawn_double_forward = pawn.pawn_double_forward(self.to_move, all_pieces);

			for forward in pawn_forward.as_singles() {
				let mut own_state = own_state.clone();
				let opponent_state = opponent_state.clone();

				own_state.pawn = pawn_removed | forward;

				push_state(own_state, opponent_state, EnPassantState::empty());
			}

			for capture in pawn_captures.as_singles() {
				let mut own_state = own_state.clone();
				let mut opponent_state = opponent_state.clone();

				own_state.pawn = pawn_removed | capture;
				opponent_state.perform_capture(capture);

				push_state(own_state, opponent_state, EnPassantState::empty());
			}

			for en_passant in en_passant.as_singles() {
				let mut own_state = own_state.clone();
				let mut opponent_state = opponent_state.clone();

				own_state.pawn = pawn_removed | en_passant;
				opponent_state.pawn &= !self.en_passant.capture;

				push_state(own_state, opponent_state, EnPassantState::empty());
			}

			if pawn_double_forward.is_nonempty() {
				let mut own_state = own_state.clone();
				let opponent_state = opponent_state.clone();

				own_state.pawn = pawn_removed | pawn_double_forward.capture;

				push_state(own_state, opponent_state, pawn_double_forward)
			}
		}

		for bishop in own_state.bishop.as_singles() {
			let bishop_removed = own_state.bishop & !bishop;
			let destinations = bishop.bishop_slides(all_pieces) & !own_pieces;

			for destination in destinations.as_singles() {
				let mut own_state = own_state.clone();
				let mut opponent_state = opponent_state.clone();

				own_state.bishop = bishop_removed | destination;
				opponent_state.perform_capture(destination);

				push_state(own_state, opponent_state, EnPassantState::empty());
			}
		}

		for knight in own_state.knight.as_singles() {
			let knight_removed = own_state.knight & !knight;
			let destinations = knight.knight_shifts() & !own_pieces;

			for destination in destinations.as_singles() {
				let mut own_state = own_state.clone();
				let mut opponent_state = opponent_state.clone();

				own_state.knight = knight_removed | destination;
				opponent_state.perform_capture(destination);

				push_state(own_state, opponent_state, EnPassantState::empty());
			}
		}

		for rook in own_state.rook.as_singles() {
			let rook_removed = own_state.rook & !rook;
			let destinations = rook.rook_slides(all_pieces) & !own_pieces;

			for destination in destinations.as_singles() {
				let mut own_state = own_state.clone();
				let mut opponent_state = opponent_state.clone();

				own_state.rook = rook_removed | destination;
				opponent_state.perform_capture(destination);

				push_state(own_state, opponent_state, EnPassantState::empty());
			}
		}

		for queen in own_state.queen.as_singles() {
			let queen_removed = own_state.queen & !queen;
			let destinations = queen.queen_slides(all_pieces) & !own_pieces;

			for destination in destinations.as_singles() {
				let mut own_state = own_state.clone();
				let mut opponent_state = opponent_state.clone();

				own_state.queen = queen_removed | destination;
				opponent_state.perform_capture(destination);

				push_state(own_state, opponent_state, EnPassantState::empty());
			}
		}

		for king in own_state.king.as_singles() {
			let destinations = king.king_shifts() & !own_pieces;

			for destination in destinations.as_singles() {
				let mut own_state = own_state.clone();
				let mut opponent_state = opponent_state.clone();

				own_state.king = destination;
				opponent_state.perform_capture(destination);

				own_state.can_castle_kingside = false;
				own_state.can_castle_queenside = false;

				push_state(own_state, opponent_state, EnPassantState::empty());
			}

			'castle_loop: for side in [CastlingType::Kingside, CastlingType::Queenside] {
				if !own_state[side] {
					continue;
				}

				let castling_data = king.castling_data(side);

				if (castling_data.needs_empty & all_pieces).is_nonempty() {
					continue;
				}

				for king_visits in castling_data.needs_safe.as_singles() {
					let mut own_state = own_state.clone();
					let opponent_state = opponent_state.clone();

					own_state.king = king_visits;

					let state = match self.to_move {
						Player::White => Self {
							white: own_state,
							black: opponent_state,
							to_move: Player::White,
							en_passant: EnPassantState::empty(),
						},
						Player::Black => Self {
							white: opponent_state,
							black: own_state,
							to_move: Player::Black,
							en_passant: EnPassantState::empty(),
						},
					};

					if state.in_check() {
						continue 'castle_loop;
					}
				}

				let mut own_state = own_state.clone();
				let opponent_state = opponent_state.clone();

				own_state.king = castling_data.king_result;
				own_state.rook &= !castling_data.delete_rook;
				own_state.rook |= castling_data.create_rook;

				own_state.can_castle_kingside = false;
				own_state.can_castle_queenside = false;

				push_state(own_state, opponent_state, EnPassantState::empty())
			}
		}

		out
	}

	pub(crate) fn in_check(&self) -> bool {
		let king = self[self.to_move].king;

		let own_state = self[self.to_move].clone();
		let opponent_state = self[self.to_move.flipped()].clone();

		let own_pieces = own_state.all_pieces();
		let opponent_pieces = opponent_state.all_pieces();
		let all_pieces = own_pieces | opponent_pieces;

		if (king.pawn_captures(self.to_move) & opponent_state.pawn).is_nonempty() {
			return true;
		}

		if (king.bishop_slides(all_pieces) & opponent_state.bishop).is_nonempty() {
			return true;
		}

		if (king.knight_shifts() & opponent_state.knight).is_nonempty() {
			return true;
		}

		if (king.rook_slides(all_pieces) & opponent_state.rook).is_nonempty() {
			return true;
		}

		if (king.queen_slides(all_pieces) & opponent_state.queen).is_nonempty() {
			return true;
		}

		if (king.king_shifts() & opponent_state.king).is_nonempty() {
			return true;
		}

		false
	}

	pub(crate) fn from_fen(fen: &str) -> Self {
		let mut out = Self {
			white: PlayerState {
				pawn: Layer::empty(),
				bishop: Layer::empty(),
				knight: Layer::empty(),
				rook: Layer::empty(),
				queen: Layer::empty(),
				king: Layer::empty(),
				can_castle_kingside: false,
				can_castle_queenside: true,
			},
			black: PlayerState {
				pawn: Layer::empty(),
				bishop: Layer::empty(),
				knight: Layer::empty(),
				rook: Layer::empty(),
				queen: Layer::empty(),
				king: Layer::empty(),
				can_castle_kingside: false,
				can_castle_queenside: true,
			},
			to_move: Player::White,
			en_passant: EnPassantState::empty(),
		};

		let split: Vec<_> = fen.split(" ").collect();
		let board = split[0];
		let to_play = split[1];
		let castles = split[2];
		let en_passant = split[3];

		let mut file_i = 0;
		let mut rank_i = 7;

		for ch in board.chars() {
			let square = Square::from_rank_and_file(File::ALL[file_i % 8], Rank::ALL[rank_i]);

			match ch {
				'P' => {
					out.white.pawn |= Layer::from_square(square);
					file_i += 1;
				},
				'B' => {
					out.white.bishop |= Layer::from_square(square);
					file_i += 1;
				},
				'N' => {
					out.white.knight |= Layer::from_square(square);
					file_i += 1;
				},
				'R' => {
					out.white.rook |= Layer::from_square(square);
					file_i += 1;
				},
				'Q' => {
					out.white.queen |= Layer::from_square(square);
					file_i += 1;
				},
				'K' => {
					out.white.king |= Layer::from_square(square);
					file_i += 1;
				},
				'p' => {
					out.black.pawn |= Layer::from_square(square);
					file_i += 1;
				},
				'b' => {
					out.black.bishop |= Layer::from_square(square);
					file_i += 1;
				},
				'n' => {
					out.black.knight |= Layer::from_square(square);
					file_i += 1;
				},
				'r' => {
					out.black.rook |= Layer::from_square(square);
					file_i += 1;
				},
				'q' => {
					out.black.queen |= Layer::from_square(square);
					file_i += 1;
				},
				'k' => {
					out.black.king |= Layer::from_square(square);
					file_i += 1;
				},
				'1' => file_i += 1,
				'2' => file_i += 2,
				'3' => file_i += 3,
				'4' => file_i += 4,
				'5' => file_i += 5,
				'6' => file_i += 6,
				'7' => file_i += 7,
				'8' => file_i += 8,
				'/' => {
					file_i = 0;
					rank_i -= 1
				},
				_ => panic!(),
			};
		}

		match to_play {
			"w" => out.to_move = Player::White,
			"b" => out.to_move = Player::Black,
			_ => panic!(),
		}

		for ch in castles.chars() {
			match ch {
				'K' => out.white.can_castle_kingside = true,
				'Q' => out.white.can_castle_queenside = true,
				'k' => out.black.can_castle_kingside = true,
				'q' => out.black.can_castle_queenside = true,
				'-' => {},
				_ => panic!(),
			}
		}

		if en_passant != "-" {
			let move_to: Square = en_passant.parse().unwrap();
			let capture = Square::from_rank_and_file(move_to.file(), match move_to.rank() {
				Rank::N3 => Rank::N4,
				Rank::N6 => Rank::N5,
				_ => panic!(),
			});

			out.en_passant = EnPassantState {
				move_to: Layer::from_square(move_to),
				capture: Layer::from_square(capture),
			}
		}

		out
	}
}

impl PlayerState {
	fn all_pieces(&self) -> Layer {
		self.pawn | self.bishop | self.knight | self.rook | self.queen | self.king
	}

	pub(crate) fn perform_capture(&mut self, capture_mask: Layer) {
		self.pawn &= !capture_mask;
		self.bishop &= !capture_mask;
		self.knight &= !capture_mask;
		self.rook &= !capture_mask;
		self.queen &= !capture_mask;
		self.king &= !capture_mask;
	}
}

impl EnPassantState {
	pub(crate) fn empty() -> Self {
		Self {
			move_to: Layer::empty(),
			capture: Layer::empty(),
		}
	}

	pub(crate) fn is_nonempty(&self) -> bool {
		self.move_to.is_nonempty()
	}
}

impl Index<Player> for State {
	type Output = PlayerState;

	fn index(&self, player: Player) -> &Self::Output {
		match player {
			Player::White => &self.white,
			Player::Black => &self.black,
		}
	}
}

impl IndexMut<Player> for State {
	fn index_mut(&mut self, player: Player) -> &mut Self::Output {
		match player {
			Player::White => &mut self.white,
			Player::Black => &mut self.black,
		}
	}
}

impl Index<Piece> for PlayerState {
	type Output = Layer;

	fn index(&self, piece: Piece) -> &Layer {
		match piece {
			Piece::Pawn => &self.pawn,
			Piece::Bishop => &self.bishop,
			Piece::Knight => &self.knight,
			Piece::Rook => &self.rook,
			Piece::Queen => &self.queen,
			Piece::King => &self.king,
		}
	}
}

impl IndexMut<Piece> for PlayerState {
	fn index_mut(&mut self, piece: Piece) -> &mut Layer {
		match piece {
			Piece::Pawn => &mut self.pawn,
			Piece::Bishop => &mut self.bishop,
			Piece::Knight => &mut self.knight,
			Piece::Rook => &mut self.rook,
			Piece::Queen => &mut self.queen,
			Piece::King => &mut self.king,
		}
	}
}

impl Index<CastlingType> for PlayerState {
	type Output = bool;

	fn index(&self, castling_type: CastlingType) -> &bool {
		match castling_type {
			CastlingType::Kingside => &self.can_castle_kingside,
			CastlingType::Queenside => &self.can_castle_queenside,
		}
	}
}

impl IndexMut<CastlingType> for PlayerState {
	fn index_mut(&mut self, castling_type: CastlingType) -> &mut bool {
		match castling_type {
			CastlingType::Kingside => &mut self.can_castle_kingside,
			CastlingType::Queenside => &mut self.can_castle_queenside,
		}
	}
}

impl Debug for State {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f, "[")?;

		let mut empties = 0;

		for rank in Rank::ALL.into_iter().rev() {
			for file in File::ALL {
				let square = Square::from_rank_and_file(file, rank);

				'search: {
					for player in [Player::White, Player::Black] {
						for piece in Piece::ALL {
							if (self[player][piece] & Layer::from_square(square)).is_empty() {
								continue;
							}

							if empties != 0 {
								write!(f, "{}", empties)?;
								empties = 0;
							}

							write!(f, "{}", match (player, piece) {
								(Player::White, Piece::Pawn) => 'P',
								(Player::White, Piece::Bishop) => 'B',
								(Player::White, Piece::Knight) => 'N',
								(Player::White, Piece::Rook) => 'R',
								(Player::White, Piece::Queen) => 'Q',
								(Player::White, Piece::King) => 'K',
								(Player::Black, Piece::Pawn) => 'p',
								(Player::Black, Piece::Bishop) => 'b',
								(Player::Black, Piece::Knight) => 'n',
								(Player::Black, Piece::Rook) => 'r',
								(Player::Black, Piece::Queen) => 'q',
								(Player::Black, Piece::King) => 'k',
							})?;

							break 'search;
						}
					}

					empties += 1;
				}
			}

			if empties != 0 {
				write!(f, "{}", empties)?;
				empties = 0;
			}

			if rank != Rank::N1 {
				write!(f, "/")?;
			}
		}

		write!(f, " ")?;

		write!(f, "{}", match self.to_move {
			Player::White => 'w',
			Player::Black => 'b',
		})?;

		write!(f, " ")?;

		let mut has_castling_rights = false;

		for player in [Player::White, Player::Black] {
			for side in [CastlingType::Kingside, CastlingType::Queenside] {
				if self[player][side] {
					has_castling_rights = true;

					write!(f, "{}", match (player, side) {
						(Player::White, CastlingType::Kingside) => 'K',
						(Player::White, CastlingType::Queenside) => 'Q',
						(Player::Black, CastlingType::Kingside) => 'k',
						(Player::Black, CastlingType::Queenside) => 'q',
					})?;
				}
			}
		}

		if !has_castling_rights {
			write!(f, "-")?;
		}

		write!(f, " ")?;

		if self.en_passant.is_nonempty() {
			write!(f, "{}", self.en_passant.move_to.iter_squares().next().unwrap())?;
		} else {
			write!(f, "-")?;
		}

		write!(f, "]")?;

		Ok(())
	}
}
