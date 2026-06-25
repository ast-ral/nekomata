// I'm not going to lie.
// This is some of the absolute worst, most fragile, and least elegant code I've ever written.

use std::fmt::{Debug, Error, Formatter};
use std::iter::from_fn;

#[derive(Clone)]
pub(crate) struct Board {
	board: [Option<(Player, Piece)>; 64],
	white_can_castle_kingside: bool,
	white_can_castle_queenside: bool,
	black_can_castle_kingside: bool,
	black_can_castle_queenside: bool,
	en_passant_available: Option<(Square, Square)>, // (<possible move>, <capture>)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Player {
	White,
	Black,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Piece {
	Pawn,
	Bishop,
	Knight,
	Rook,
	Queen,
	King,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Square {
	index: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Turn {
	from: Square,
	to: Square,
	promotion: Option<Piece>,
}

impl Board {
	pub(crate) fn new() -> Self {
		Self {
			board: [
				// rank 1
				Some((Player::White, Piece::Rook)),
				Some((Player::White, Piece::Knight)),
				Some((Player::White, Piece::Bishop)),
				Some((Player::White, Piece::Queen)),
				Some((Player::White, Piece::King)),
				Some((Player::White, Piece::Bishop)),
				Some((Player::White, Piece::Knight)),
				Some((Player::White, Piece::Rook)),
				// rank 2
				Some((Player::White, Piece::Pawn)),
				Some((Player::White, Piece::Pawn)),
				Some((Player::White, Piece::Pawn)),
				Some((Player::White, Piece::Pawn)),
				Some((Player::White, Piece::Pawn)),
				Some((Player::White, Piece::Pawn)),
				Some((Player::White, Piece::Pawn)),
				Some((Player::White, Piece::Pawn)),
				// rank 3
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				// rank 4
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				// rank 5
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				// rank 6
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
				// rank 7
				Some((Player::Black, Piece::Pawn)),
				Some((Player::Black, Piece::Pawn)),
				Some((Player::Black, Piece::Pawn)),
				Some((Player::Black, Piece::Pawn)),
				Some((Player::Black, Piece::Pawn)),
				Some((Player::Black, Piece::Pawn)),
				Some((Player::Black, Piece::Pawn)),
				Some((Player::Black, Piece::Pawn)),
				// rank 8
				Some((Player::Black, Piece::Rook)),
				Some((Player::Black, Piece::Knight)),
				Some((Player::Black, Piece::Bishop)),
				Some((Player::Black, Piece::Queen)),
				Some((Player::Black, Piece::King)),
				Some((Player::Black, Piece::Bishop)),
				Some((Player::Black, Piece::Knight)),
				Some((Player::Black, Piece::Rook)),
			],
			white_can_castle_kingside: true,
			white_can_castle_queenside: true,
			black_can_castle_kingside: true,
			black_can_castle_queenside: true,
			en_passant_available: None,
		}
	}

	pub(crate) fn perform_turn(&mut self, turn: Turn) {
		let (player, piece) = self.query(turn.from).unwrap();
		let piece = turn.promotion.unwrap_or(piece);

		match turn.to {
			Square::WHITE_KINGSIDE_ROOK_START => self.white_can_castle_kingside = false,
			Square::WHITE_QUEENSIDE_ROOK_START => self.white_can_castle_queenside = false,
			Square::BLACK_KINGSIDE_ROOK_START => self.black_can_castle_kingside = false,
			Square::BLACK_QUEENSIDE_ROOK_START => self.black_can_castle_queenside = false,
			_ => {},
		}

		match turn.from {
			Square::WHITE_KINGSIDE_ROOK_START => self.white_can_castle_kingside = false,
			Square::WHITE_QUEENSIDE_ROOK_START => self.white_can_castle_queenside = false,
			Square::BLACK_KINGSIDE_ROOK_START => self.black_can_castle_kingside = false,
			Square::BLACK_QUEENSIDE_ROOK_START => self.black_can_castle_queenside = false,
			_ => {},
		}

		if piece == Piece::King {
			match player {
				Player::White => {
					self.white_can_castle_kingside = false;
					self.white_can_castle_queenside = false;
				},
				Player::Black => {
					self.black_can_castle_kingside = false;
					self.black_can_castle_queenside = false;
				},
			}

			match (turn.from, turn.to) {
				(Square::WHITE_KING_START, Square::WHITE_KING_CASTLE_KINGSIDE) => {
					*self.query_mut(Square::WHITE_KINGSIDE_ROOK_START) = None;
					*self.query_mut(Square::WHITE_CASTLE_KINGSIDE_ROOK_RESULT) = Some((player, Piece::Rook));
				},
				(Square::WHITE_KING_START, Square::WHITE_KING_CASTLE_QUEENSIDE) => {
					*self.query_mut(Square::WHITE_QUEENSIDE_ROOK_START) = None;
					*self.query_mut(Square::WHITE_CASTLE_QUEENSIDE_ROOK_RESULT) = Some((player, Piece::Rook));
				},
				(Square::BLACK_KING_START, Square::BLACK_KING_CASTLE_KINGSIDE) => {
					*self.query_mut(Square::BLACK_KINGSIDE_ROOK_START) = None;
					*self.query_mut(Square::BLACK_CASTLE_KINGSIDE_ROOK_RESULT) = Some((player, Piece::Rook));
				},
				(Square::BLACK_KING_START, Square::BLACK_KING_CASTLE_QUEENSIDE) => {
					*self.query_mut(Square::BLACK_QUEENSIDE_ROOK_START) = None;
					*self.query_mut(Square::BLACK_CASTLE_QUEENSIDE_ROOK_RESULT) = Some((player, Piece::Rook));
				},
				_ => {},
			}
		}

		if piece == Piece::Pawn
			&& let Some((move_to, capture)) = self.en_passant_available
			&& turn.to == move_to
		{
			*self.query_mut(capture) = None;
		}

		self.en_passant_available = None;

		if piece == Piece::Pawn {
			if turn.from.is_on_starting_pawn_rank(player) && turn.from.rank().abs_diff(turn.to.rank()) > 1 {
				self.en_passant_available = Some((turn.from.pawn_direction(player).unwrap(), turn.to));
			}
		}

		*self.query_mut(turn.from) = None;
		*self.query_mut(turn.to) = Some((player, piece));
	}

	pub(crate) fn query(&self, square: Square) -> &Option<(Player, Piece)> {
		&self.board[square.index as usize]
	}

	pub(crate) fn query_mut(&mut self, square: Square) -> &mut Option<(Player, Piece)> {
		&mut self.board[square.index as usize]
	}

	pub(crate) fn find_king_square(&self, of_player: Player) -> Square {
		for (index, contents) in self.board.iter().copied().enumerate() {
			let square = Square::from_raw(index);

			let Some((player, Piece::King)) = contents else {
				continue;
			};

			if player == of_player {
				return square;
			}
		}

		panic!("king not found");
	}

	pub(crate) fn get_allied_piece_squares(&self, of_player: Player) -> impl Iterator<Item = Square> {
		self.board
			.iter()
			.copied()
			.enumerate()
			.filter_map(move |(square, contents)| {
				let square = Square::from_raw(square);
				let (player, _) = contents?;

				if player == of_player {
					Some(square)
				} else {
					return None;
				}
			})
	}

	pub(crate) fn get_enemy_piece_squares(&self, player: Player) -> impl Iterator<Item = Square> {
		self.get_allied_piece_squares(player.flipped())
	}

	pub(crate) fn king_in_check(&self, player: Player) -> bool {
		let king_square = self.find_king_square(player);

		for square in self.get_enemy_piece_squares(player) {
			for square in self.get_capture_destinations_for_piece_on_square(square) {
				if square == king_square {
					return true;
				}
			}
		}

		false
	}

	fn get_capture_destinations_for_piece_on_square(&self, square: Square) -> Vec<Square> {
		let (player, piece) = self.query(square).unwrap();

		let continue_direction = |update: fn(&Square) -> Option<Square>| {
			let mut square = square;
			let mut ending = false;

			from_fn(move || {
				if ending {
					return None;
				}

				square = update(&square)?;

				if let &Some((of_player, _)) = self.query(square) {
					if player == of_player {
						return None;
					}

					ending = true;
				}

				Some(square)
			})
		};

		match piece {
			Piece::Pawn => {
				let pawn_movement = square.pawn_direction(player);
				let possibilities = [
					pawn_movement.and_then(|x| x.left()),
					pawn_movement.and_then(|x| x.right()),
				];

				possibilities
					.into_iter()
					.filter_map(|x| x)
					.filter_map(|x| {
						self.query(x)
							.is_none_or(|(of_player, _)| player != of_player)
							.then_some(x)
					})
					.collect()
			},
			Piece::Bishop => {
				let directions = [Square::up_left, Square::up_right, Square::down_left, Square::down_right];

				directions.into_iter().flat_map(continue_direction).collect()
			},
			Piece::Knight => {
				let possibilities = [
					square.up().and_then(|x| x.up_left()),
					square.up().and_then(|x| x.up_right()),
					square.down().and_then(|x| x.down_left()),
					square.down().and_then(|x| x.down_right()),
					square.left().and_then(|x| x.up_left()),
					square.left().and_then(|x| x.down_left()),
					square.right().and_then(|x| x.up_right()),
					square.right().and_then(|x| x.down_right()),
				];

				possibilities
					.into_iter()
					.filter_map(|x| x)
					.filter_map(|x| {
						self.query(x)
							.is_none_or(|(of_player, _)| player != of_player)
							.then_some(x)
					})
					.collect()
			},
			Piece::Rook => {
				let directions = [Square::up, Square::down, Square::left, Square::right];

				directions.into_iter().flat_map(continue_direction).collect()
			},
			Piece::Queen => {
				let directions = [
					Square::up,
					Square::down,
					Square::left,
					Square::right,
					Square::up_left,
					Square::up_right,
					Square::down_left,
					Square::down_right,
				];

				directions.into_iter().flat_map(continue_direction).collect()
			},
			Piece::King => {
				let possibilities = [
					square.up(),
					square.down(),
					square.left(),
					square.right(),
					square.up_left(),
					square.up_right(),
					square.down_left(),
					square.down_right(),
				];

				possibilities
					.into_iter()
					.filter_map(|x| x)
					.filter_map(|x| {
						self.query(x)
							.is_none_or(|(of_player, _)| player != of_player)
							.then_some(x)
					})
					.collect()
			},
		}
	}

	pub(crate) fn get_possible_turns(&self, player: Player) -> Vec<Turn> {
		let mut out = Vec::new();

		for square in self.get_allied_piece_squares(player) {
			let Some((_, piece)) = *self.query(square) else {
				unreachable!();
			};

			match piece {
				Piece::Pawn => {
					let forward = square.pawn_direction(player).unwrap();

					if self.query(forward).is_none() {
						if square.is_on_starting_pawn_rank(player) {
							let double_forward = forward.pawn_direction(player).unwrap();

							if self.query(double_forward).is_none() {
								out.push(Turn {
									from: square,
									to: double_forward,
									promotion: None,
								});
							}
						}

						if forward.is_on_promotion_rank(player) {
							for promotion in [Piece::Bishop, Piece::Knight, Piece::Rook, Piece::Queen] {
								out.push(Turn {
									from: square,
									to: forward,
									promotion: Some(promotion),
								});
							}
						} else {
							out.push(Turn {
								from: square,
								to: forward,
								promotion: None,
							})
						}
					}

					for destination in self.get_capture_destinations_for_piece_on_square(square) {
						if self.query(destination).is_none()
							&& self
								.en_passant_available
								.is_none_or(|(en_passant_destination, _)| destination != en_passant_destination)
						{
							continue;
						}

						if destination.is_on_promotion_rank(player) {
							for promotion in [Piece::Bishop, Piece::Knight, Piece::Rook, Piece::Queen] {
								out.push(Turn {
									from: square,
									to: destination,
									promotion: Some(promotion),
								});
							}
						} else {
							out.push(Turn {
								from: square,
								to: destination,
								promotion: None,
							});
						}
					}
				},
				Piece::King => {
					let (
						can_castle_kingside,
						can_castle_queenside,
						kingside_intermediate_square,
						kingside_destination_square,
						queenside_intermediate_square,
						queenside_destination_square,
					) = match player {
						Player::White => (
							self.white_can_castle_kingside,
							self.white_can_castle_queenside,
							Square::WHITE_CASTLE_KINGSIDE_ROOK_RESULT,
							Square::WHITE_KING_CASTLE_KINGSIDE,
							Square::WHITE_CASTLE_QUEENSIDE_ROOK_RESULT,
							Square::WHITE_KING_CASTLE_QUEENSIDE,
						),
						Player::Black => (
							self.black_can_castle_kingside,
							self.black_can_castle_queenside,
							Square::BLACK_CASTLE_KINGSIDE_ROOK_RESULT,
							Square::BLACK_KING_CASTLE_KINGSIDE,
							Square::BLACK_CASTLE_QUEENSIDE_ROOK_RESULT,
							Square::BLACK_KING_CASTLE_QUEENSIDE,
						),
					};

					let in_check = self.king_in_check(player);

					'castle: {
						if in_check {
							break 'castle;
						}

						if !can_castle_kingside {
							break 'castle;
						}

						if !self.query(kingside_intermediate_square).is_none() {
							break 'castle;
						}

						if !self.query(kingside_destination_square).is_none() {
							break 'castle;
						}

						let intermediate_turn = Turn {
							from: square,
							to: kingside_intermediate_square,
							promotion: None,
						};
						let mut intermediate_board = self.clone();
						intermediate_board.perform_turn(intermediate_turn);

						if intermediate_board.king_in_check(player) {
							break 'castle;
						}

						out.push(Turn {
							from: square,
							to: kingside_destination_square,
							promotion: None,
						});
					}

					'castle: {
						if in_check {
							break 'castle;
						}

						if !can_castle_queenside {
							break 'castle;
						}

						if !self.query(queenside_intermediate_square).is_none() {
							break 'castle;
						}

						if !self.query(queenside_destination_square).is_none() {
							break 'castle;
						}

						let intermediate_turn = Turn {
							from: square,
							to: queenside_intermediate_square,
							promotion: None,
						};
						let mut intermediate_board = self.clone();
						intermediate_board.perform_turn(intermediate_turn);

						if intermediate_board.king_in_check(player) {
							break 'castle;
						}

						out.push(Turn {
							from: square,
							to: queenside_destination_square,
							promotion: None,
						});
					}

					out.extend(
						self.get_capture_destinations_for_piece_on_square(square)
							.into_iter()
							.map(|to| Turn {
								from: square,
								to,
								promotion: None,
							}),
					);
				},
				_ => {
					out.extend(
						self.get_capture_destinations_for_piece_on_square(square)
							.into_iter()
							.map(|to| Turn {
								from: square,
								to,
								promotion: None,
							}),
					);
				},
			}
		}

		out.retain(|&turn| {
			let mut result = self.clone();
			result.perform_turn(turn);

			!result.king_in_check(player)
		});

		out
	}
}

impl Player {
	pub(crate) fn flipped(self) -> Self {
		match self {
			Self::White => Self::Black,
			Self::Black => Self::White,
		}
	}
}

impl Square {
	pub(crate) fn from_name(name: &str) -> Self {
		let mut name = name.bytes();
		let file = name.next().unwrap();
		assert!(matches!(file, b'a' ..= b'h'));
		let rank = name.next().unwrap();
		assert!(matches!(rank, b'1' ..= b'8'));
		assert!(name.next().is_none());

		Self {
			index: (rank - b'1') * 8 + (file - b'a'),
		}
	}

	pub(crate) fn to_name(&self) -> String {
		let mut out = String::new();

		const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
		const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

		out.push(FILES[self.file() as usize]);
		out.push(RANKS[self.rank() as usize]);

		out
	}

	const fn from_raw(index: usize) -> Self {
		Self { index: index as u8 }
	}

	pub(crate) fn rank(&self) -> u8 {
		self.index / 8
	}

	pub(crate) fn file(&self) -> u8 {
		self.index % 8
	}

	fn up(&self) -> Option<Self> {
		(self.rank() != 7).then(|| Self { index: self.index + 8 })
	}

	fn down(&self) -> Option<Self> {
		(self.rank() != 0).then(|| Self { index: self.index - 8 })
	}

	fn left(&self) -> Option<Self> {
		(self.file() != 0).then(|| Self { index: self.index - 1 })
	}

	fn right(&self) -> Option<Self> {
		(self.file() != 7).then(|| Self { index: self.index + 1 })
	}

	fn up_left(&self) -> Option<Self> {
		self.up()?.left()
	}

	fn up_right(&self) -> Option<Self> {
		self.up()?.right()
	}

	fn down_left(&self) -> Option<Self> {
		self.down()?.left()
	}

	fn down_right(&self) -> Option<Self> {
		self.down()?.right()
	}

	fn pawn_direction(&self, player: Player) -> Option<Self> {
		match player {
			Player::White => self.up(),
			Player::Black => self.down(),
		}
	}

	fn is_on_starting_pawn_rank(&self, player: Player) -> bool {
		let starting_pawn_rank = match player {
			Player::White => 1,
			Player::Black => 6,
		};

		self.rank() == starting_pawn_rank
	}

	fn is_on_promotion_rank(&self, player: Player) -> bool {
		let promotion_rank = match player {
			Player::White => 7,
			Player::Black => 0,
		};

		self.rank() == promotion_rank
	}

	const WHITE_KING_START: Self = Self::from_raw(4);
	const BLACK_KING_START: Self = Self::from_raw(60);

	const WHITE_KING_CASTLE_KINGSIDE: Self = Self::from_raw(6);
	const WHITE_KING_CASTLE_QUEENSIDE: Self = Self::from_raw(2);
	const BLACK_KING_CASTLE_KINGSIDE: Self = Self::from_raw(62);
	const BLACK_KING_CASTLE_QUEENSIDE: Self = Self::from_raw(58);

	const WHITE_KINGSIDE_ROOK_START: Self = Self::from_raw(7);
	const WHITE_QUEENSIDE_ROOK_START: Self = Self::from_raw(0);
	const BLACK_KINGSIDE_ROOK_START: Self = Self::from_raw(63);
	const BLACK_QUEENSIDE_ROOK_START: Self = Self::from_raw(56);

	const WHITE_CASTLE_KINGSIDE_ROOK_RESULT: Self = Self::from_raw(5);
	const WHITE_CASTLE_QUEENSIDE_ROOK_RESULT: Self = Self::from_raw(3);
	const BLACK_CASTLE_KINGSIDE_ROOK_RESULT: Self = Self::from_raw(61);
	const BLACK_CASTLE_QUEENSIDE_ROOK_RESULT: Self = Self::from_raw(59);
}

impl Turn {
	pub(crate) fn from_uci(text: &str) -> Self {
		Turn {
			from: Square::from_name(&text[0 .. 2]),
			to: Square::from_name(&text[2 .. 4]),
			promotion: match &text[4 ..] {
				"" => None,
				"b" => Some(Piece::Bishop),
				"n" => Some(Piece::Knight),
				"r" => Some(Piece::Rook),
				"q" => Some(Piece::Queen),
				_ => panic!("unrecognized promotion"),
			},
		}
	}

	pub(crate) fn to_uci(&self) -> String {
		let mut out = String::new();

		out.push_str(&self.from.to_name());
		out.push_str(&self.to.to_name());

		match self.promotion {
			Some(Piece::Bishop) => out.push('b'),
			Some(Piece::Knight) => out.push('n'),
			Some(Piece::Rook) => out.push('r'),
			Some(Piece::Queen) => out.push('q'),
			None => {},

			_ => unreachable!(),
		}

		out
	}
}

impl Debug for Board {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f, "[")?;

		let mut already_written = false;

		for (i, contents) in self.board.iter().copied().enumerate() {
			let square = Square::from_raw(i);

			if let Some((player, piece)) = contents {
				if already_written {
					write!(f, " ")?;
				}

				write!(
					f,
					"{}:{}{}",
					square.to_name(),
					match player {
						Player::White => "w",
						Player::Black => "b",
					},
					match piece {
						Piece::Pawn => "p",
						Piece::Bishop => "b",
						Piece::Knight => "n",
						Piece::Rook => "r",
						Piece::Queen => "q",
						Piece::King => "k",
					}
				)?;
				already_written = true;
			}
		}

		write!(f, "]")?;

		Ok(())
	}
}

impl Debug for Square {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f, "{}", self.to_name())?;

		Ok(())
	}
}
