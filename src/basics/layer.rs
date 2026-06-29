use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::fmt::{Debug, Error, Formatter};
use std::iter::from_fn;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::basics::castling::{CastlingData, CastlingType};
use crate::basics::player::Player;
use crate::basics::square::Square;
use crate::basics::state::EnPassantState;

fn extract_bits(value: u64, mask: u64) -> u64 {
	unsafe { _pext_u64(value, mask) }
}

fn deposit_bits(value: u64, mask: u64) -> u64 {
	unsafe { _pdep_u64(value, mask) }
}

fn fill_from_left(value: u64) -> u64 {
	let index = value.leading_zeros();

	!(((1u64 << 63) - 1).unbounded_shr(index))
}

fn fill_from_right(value: u64) -> u64 {
	value ^ value.wrapping_sub(1)
}

#[derive(Clone, Copy)]
pub(crate) struct Layer {
	bits: u64,
}

impl Layer {
	pub(crate) fn empty() -> Self {
		Self { bits: 0 }
	}

	pub(crate) fn from_square(square: Square) -> Self {
		Self {
			bits: 1 << square.to_shift(),
		}
	}

	pub(crate) fn iter_squares(&self) -> impl Iterator<Item = Square> {
		let mut bits = self.bits;
		let mut i = 0;

		from_fn(move || {
			let trailing = bits.trailing_zeros();

			if trailing == 64 {
				return None;
			}

			bits = bits.unbounded_shr(trailing + 1);
			i += trailing + 1;

			Some(Square::from_shift(i - 1))
		})
	}

	pub(crate) fn as_singles(&self) -> impl Iterator<Item = Self> {
		self.iter_squares().map(Self::from_square)
	}

	pub(crate) fn is_empty(&self) -> bool {
		self.bits == 0
	}

	pub(crate) fn is_nonempty(&self) -> bool {
		self.bits != 0
	}

	pub(crate) fn num_squares(&self) -> u32 {
		self.bits.count_ones()
	}

	const LEFT_MASK_1: u64 = 0b_11111110_11111110_11111110_11111110_11111110_11111110_11111110_11111110;
	const LEFT_MASK_2: u64 = 0b_11111100_11111100_11111100_11111100_11111100_11111100_11111100_11111100;
	const LEFT_MASK_4: u64 = 0b_11110000_11110000_11110000_11110000_11110000_11110000_11110000_11110000;

	const RIGHT_MASK_1: u64 = 0b_01111111_01111111_01111111_01111111_01111111_01111111_01111111_01111111;
	const RIGHT_MASK_2: u64 = 0b_00111111_00111111_00111111_00111111_00111111_00111111_00111111_00111111;
	const RIGHT_MASK_4: u64 = 0b_00001111_00001111_00001111_00001111_00001111_00001111_00001111_00001111;

	fn slide_mask_up(self) -> Self {
		let bits = self.bits;
		let bits = bits | bits << 8;
		let bits = bits | bits << 16;
		let bits = bits | bits << 32;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_mask_down(self) -> Self {
		let bits = self.bits;
		let bits = bits | bits >> 8;
		let bits = bits | bits >> 16;
		let bits = bits | bits >> 32;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_mask_left(self) -> Self {
		let bits = self.bits;
		let bits = bits | (bits & Self::LEFT_MASK_1) >> 1;
		let bits = bits | (bits & Self::LEFT_MASK_2) >> 2;
		let bits = bits | (bits & Self::LEFT_MASK_4) >> 4;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_mask_right(self) -> Self {
		let bits = self.bits;
		let bits = bits | (bits & Self::RIGHT_MASK_1) << 1;
		let bits = bits | (bits & Self::RIGHT_MASK_2) << 2;
		let bits = bits | (bits & Self::RIGHT_MASK_4) << 4;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_mask_up_left(self) -> Self {
		let bits = self.bits;
		let bits = bits | (bits & Self::LEFT_MASK_1) << 7;
		let bits = bits | (bits & Self::LEFT_MASK_2) << 14;
		let bits = bits | (bits & Self::LEFT_MASK_4) << 28;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_mask_up_right(self) -> Self {
		let bits = self.bits;
		let bits = bits | (bits & Self::RIGHT_MASK_1) << 9;
		let bits = bits | (bits & Self::RIGHT_MASK_2) << 18;
		let bits = bits | (bits & Self::RIGHT_MASK_4) << 36;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_mask_down_left(self) -> Self {
		let bits = self.bits;
		let bits = bits | (bits & Self::LEFT_MASK_1) >> 9;
		let bits = bits | (bits & Self::LEFT_MASK_2) >> 18;
		let bits = bits | (bits & Self::LEFT_MASK_4) >> 36;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_mask_down_right(self) -> Self {
		let bits = self.bits;
		let bits = bits | (bits & Self::RIGHT_MASK_1) >> 7;
		let bits = bits | (bits & Self::RIGHT_MASK_2) >> 14;
		let bits = bits | (bits & Self::RIGHT_MASK_4) >> 28;
		let bits = bits & !self.bits;

		Self { bits }
	}

	fn slide_up(self, blockers: Self) -> Self {
		let mask = self.slide_mask_up().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_right(extracted), mask),
		}
	}

	fn slide_down(self, blockers: Self) -> Self {
		let mask = self.slide_mask_down().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_left(extracted), mask),
		}
	}

	fn slide_left(self, blockers: Self) -> Self {
		let mask = self.slide_mask_left().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_left(extracted), mask),
		}
	}

	fn slide_right(self, blockers: Self) -> Self {
		let mask = self.slide_mask_right().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_right(extracted), mask),
		}
	}

	fn slide_up_left(self, blockers: Self) -> Self {
		let mask = self.slide_mask_up_left().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_right(extracted), mask),
		}
	}

	fn slide_up_right(self, blockers: Self) -> Self {
		let mask = self.slide_mask_up_right().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_right(extracted), mask),
		}
	}

	fn slide_down_left(self, blockers: Self) -> Self {
		let mask = self.slide_mask_down_left().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_left(extracted), mask),
		}
	}

	fn slide_down_right(self, blockers: Self) -> Self {
		let mask = self.slide_mask_down_right().bits;
		let extracted = extract_bits(blockers.bits, mask);

		Self {
			bits: deposit_bits(fill_from_left(extracted), mask),
		}
	}

	pub(crate) fn pawn_forward(self, player: Player) -> Self {
		match player {
			Player::White => Self { bits: self.bits << 8 },
			Player::Black => Self { bits: self.bits >> 8 },
		}
	}

	pub(crate) fn pawn_double_forward(self, player: Player, blockers: Self) -> EnPassantState {
		let bits = self.bits;
		let blockers = blockers.bits;

		let (move_to, capture) = match player {
			Player::White => {
				let bits = bits & 0b_00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
				let bits = (bits << 8) & !blockers;
				let bits = (bits << 8) & !blockers;

				(bits >> 8, bits)
			},
			Player::Black => {
				let bits = bits & 0b_00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
				let bits = (bits >> 8) & !blockers;
				let bits = (bits >> 8) & !blockers;

				(bits << 8, bits)
			},
		};

		let move_to = Self { bits: move_to };
		let capture = Self { bits: capture };

		EnPassantState { move_to, capture }
	}

	pub(crate) fn pawn_captures(self, player: Player) -> Self {
		let bits = self.bits;

		let bits = match player {
			Player::White => (bits & Self::LEFT_MASK_1) << 7 | (bits & Self::RIGHT_MASK_1) << 9,
			Player::Black => (bits & Self::LEFT_MASK_1) >> 9 | (bits & Self::RIGHT_MASK_1) >> 7,
		};

		Self { bits }
	}

	pub(crate) fn bishop_slides(self, blockers: Self) -> Self {
		self.slide_up_left(blockers)
			| self.slide_up_right(blockers)
			| self.slide_down_left(blockers)
			| self.slide_down_right(blockers)
	}

	pub(crate) fn knight_shifts(self) -> Self {
		let bits = self.bits;
		let bits = [
			(bits & Self::RIGHT_MASK_1) << 17,
			(bits & Self::RIGHT_MASK_2) << 10,
			(bits & Self::RIGHT_MASK_2) >> 6,
			(bits & Self::RIGHT_MASK_1) >> 15,
			(bits & Self::LEFT_MASK_1) >> 17,
			(bits & Self::LEFT_MASK_2) >> 10,
			(bits & Self::LEFT_MASK_2) << 6,
			(bits & Self::LEFT_MASK_1) << 15,
		]
		.into_iter()
		.fold(0, |a, b| a | b);

		Self { bits }
	}

	pub(crate) fn rook_slides(self, blockers: Self) -> Self {
		self.slide_up(blockers) | self.slide_down(blockers) | self.slide_left(blockers) | self.slide_right(blockers)
	}

	pub(crate) fn queen_slides(self, blockers: Self) -> Self {
		self.bishop_slides(blockers) | self.rook_slides(blockers)
	}

	pub(crate) fn king_shifts(self) -> Self {
		let bits = self.bits;
		let bits = [
			bits << 8,
			(bits & Self::RIGHT_MASK_1) << 9,
			(bits & Self::RIGHT_MASK_1) << 1,
			(bits & Self::RIGHT_MASK_1) >> 7,
			bits >> 8,
			(bits & Self::LEFT_MASK_1) >> 9,
			(bits & Self::LEFT_MASK_1) >> 1,
			(bits & Self::LEFT_MASK_1) << 7,
		]
		.into_iter()
		.fold(0, |a, b| a | b);

		Self { bits }
	}

	pub(crate) const KINGSIDE_ROOK_MASK: Self = Self {
		bits: 0b_10000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000,
	};
	pub(crate) const QUEENSIDE_ROOK_MASK: Self = Self {
		bits: 0b_00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000001,
	};

	pub(crate) fn castling_data(self, side: CastlingType) -> CastlingData {
		let bits = self.bits;

		match side {
			CastlingType::Kingside => CastlingData {
				king_result: Self { bits: bits << 2 },
				delete_rook: Self { bits: bits << 3 },
				create_rook: Self { bits: bits << 1 },
				needs_empty: Self {
					bits: bits << 1 | bits << 2,
				},
				needs_safe: Self { bits: bits | bits << 1 },
			},
			CastlingType::Queenside => CastlingData {
				king_result: Self { bits: bits >> 2 },
				delete_rook: Self { bits: bits >> 4 },
				create_rook: Self { bits: bits >> 1 },
				needs_empty: Self {
					bits: bits >> 1 | bits >> 2 | bits >> 3,
				},
				needs_safe: Self { bits: bits | bits >> 1 },
			},
		}
	}

	pub(crate) fn promotions(player: Player) -> Self {
		match player {
			Player::White => Self {
				bits: 0b_11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
			},
			Player::Black => Self {
				bits: 0b_00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111,
			},
		}
	}

	pub(crate) const MIDSCREEN: Self = Self {
		bits: 0b_00000000_00000000_11111111_11111111_11111111_11111111_00000000_00000000,
	};
}

impl BitAnd for Layer {
	type Output = Self;

	fn bitand(self, other: Self) -> Self {
		Self {
			bits: self.bits & other.bits,
		}
	}
}

impl BitOr for Layer {
	type Output = Self;

	fn bitor(self, other: Self) -> Self {
		Self {
			bits: self.bits | other.bits,
		}
	}
}

impl BitXor for Layer {
	type Output = Self;

	fn bitxor(self, other: Self) -> Self::Output {
		Self {
			bits: self.bits ^ other.bits,
		}
	}
}

impl Not for Layer {
	type Output = Self;

	fn not(self) -> Self {
		Self { bits: !self.bits }
	}
}

impl BitAndAssign for Layer {
	fn bitand_assign(&mut self, other: Self) {
		*self = *self & other;
	}
}

impl BitOrAssign for Layer {
	fn bitor_assign(&mut self, other: Self) {
		*self = *self | other;
	}
}

impl BitXorAssign for Layer {
	fn bitxor_assign(&mut self, other: Self) {
		*self = *self ^ other;
	}
}

impl Debug for Layer {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f, "Layer<")?;

		let mut after_first = false;

		for square in self.iter_squares() {
			if after_first {
				write!(f, " ")?;
			}

			write!(f, "{}", square)?;

			after_first = true;
		}

		write!(f, ">")?;

		Ok(())
	}
}
