#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Piece {
	Pawn,
	Bishop,
	Knight,
	Rook,
	Queen,
	King,
}

impl Piece {
	pub(crate) const ALL: [Self; 6] = [
		Self::Pawn,
		Self::Bishop,
		Self::Knight,
		Self::Rook,
		Self::Queen,
		Self::King,
	];

	pub(crate) fn representation(self) -> char {
		match self {
			Piece::Pawn => 'p',
			Piece::Bishop => 'b',
			Piece::Knight => 'n',
			Piece::Rook => 'r',
			Piece::Queen => 'q',
			Piece::King => 'k',
		}
	}
}
