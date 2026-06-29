use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::basics::player::Player;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Square {
	index: u8,
}

#[derive(Debug)]
pub(crate) struct SquareParseError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Rank {
	N1,
	N2,
	N3,
	N4,
	N5,
	N6,
	N7,
	N8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum File {
	A,
	B,
	C,
	D,
	E,
	F,
	G,
	H,
}

impl Square {
	pub(crate) fn from_rank_and_file(file: File, rank: Rank) -> Self {
		Self {
			index: rank.to_u8() * 8 + file.to_u8(),
		}
	}

	pub(crate) fn rank(&self) -> Rank {
		Rank::from_u8(self.index / 8)
	}

	pub(crate) fn file(&self) -> File {
		File::from_u8(self.index % 8)
	}

	pub(in crate::basics) fn from_shift(shift: u32) -> Self {
		Self { index: shift as u8 }
	}

	pub(in crate::basics) fn to_shift(&self) -> u32 {
		u32::from(self.index)
	}

	fn up(&self) -> Option<Self> {
		(self.rank() != Rank::N8).then(|| Self { index: self.index + 8 })
	}

	fn down(&self) -> Option<Self> {
		(self.rank() != Rank::N1).then(|| Self { index: self.index - 8 })
	}

	fn left(&self) -> Option<Self> {
		(self.file() != File::A).then(|| Self { index: self.index - 1 })
	}

	fn right(&self) -> Option<Self> {
		(self.file() != File::H).then(|| Self { index: self.index + 1 })
	}

	pub(crate) fn pawn_direction(&self, player: Player) -> Option<Self> {
		match player {
			Player::White => self.up(),
			Player::Black => self.down(),
		}
	}
}

impl Rank {
	pub(crate) const ALL: [Self; 8] = [
		Self::N1,
		Self::N2,
		Self::N3,
		Self::N4,
		Self::N5,
		Self::N6,
		Self::N7,
		Self::N8,
	];

	fn from_usize(index: usize) -> Self {
		match index {
			0 => Self::N1,
			1 => Self::N2,
			2 => Self::N3,
			3 => Self::N4,
			4 => Self::N5,
			5 => Self::N6,
			6 => Self::N7,
			7 => Self::N8,
			_ => panic!("index can't be converted to a file"),
		}
	}

	fn from_u8(index: u8) -> Self {
		match index {
			0 => Self::N1,
			1 => Self::N2,
			2 => Self::N3,
			3 => Self::N4,
			4 => Self::N5,
			5 => Self::N6,
			6 => Self::N7,
			7 => Self::N8,
			_ => panic!("index can't be converted to a file"),
		}
	}

	fn to_usize(self) -> usize {
		match self {
			Self::N1 => 0,
			Self::N2 => 1,
			Self::N3 => 2,
			Self::N4 => 3,
			Self::N5 => 4,
			Self::N6 => 5,
			Self::N7 => 6,
			Self::N8 => 7,
		}
	}

	fn to_u8(self) -> u8 {
		match self {
			Self::N1 => 0,
			Self::N2 => 1,
			Self::N3 => 2,
			Self::N4 => 3,
			Self::N5 => 4,
			Self::N6 => 5,
			Self::N7 => 6,
			Self::N8 => 7,
		}
	}

	fn representation(self) -> char {
		['1', '2', '3', '4', '5', '6', '7', '8'][self.to_usize()]
	}
}

impl File {
	pub(crate) const ALL: [Self; 8] = [Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H];

	fn from_usize(index: usize) -> Self {
		match index {
			0 => Self::A,
			1 => Self::B,
			2 => Self::C,
			3 => Self::D,
			4 => Self::E,
			5 => Self::F,
			6 => Self::G,
			7 => Self::H,
			_ => panic!("index can't be converted to a rank"),
		}
	}

	fn from_u8(index: u8) -> Self {
		match index {
			0 => Self::A,
			1 => Self::B,
			2 => Self::C,
			3 => Self::D,
			4 => Self::E,
			5 => Self::F,
			6 => Self::G,
			7 => Self::H,
			_ => panic!("index can't be converted to a rank"),
		}
	}

	fn to_usize(self) -> usize {
		match self {
			Self::A => 0,
			Self::B => 1,
			Self::C => 2,
			Self::D => 3,
			Self::E => 4,
			Self::F => 5,
			Self::G => 6,
			Self::H => 7,
		}
	}

	fn to_u8(self) -> u8 {
		match self {
			Self::A => 0,
			Self::B => 1,
			Self::C => 2,
			Self::D => 3,
			Self::E => 4,
			Self::F => 5,
			Self::G => 6,
			Self::H => 7,
		}
	}

	fn representation(self) -> char {
		['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'][self.to_usize()]
	}
}

impl FromStr for Square {
	type Err = SquareParseError;

	fn from_str(name: &str) -> Result<Self, SquareParseError> {
		let mut name = name.bytes();

		let file = name.next().ok_or(SquareParseError)?;
		matches!(file, b'a' ..= b'h').then_some(()).ok_or(SquareParseError)?;

		let rank = name.next().ok_or(SquareParseError)?;
		matches!(rank, b'1' ..= b'8').then_some(()).ok_or(SquareParseError)?;

		name.next().is_none().then_some(()).ok_or(SquareParseError)?;

		Ok(Self::from_rank_and_file(
			File::from_u8(file - b'a'),
			Rank::from_u8(rank - b'1'),
		))
	}
}

impl Display for Square {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{}{}", self.file().representation(), self.rank().representation())?;

		Ok(())
	}
}
