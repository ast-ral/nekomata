#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Player {
	White,
	Black,
}

impl Player {
	pub(crate) fn flipped(self) -> Self {
		match self {
			Self::White => Self::Black,
			Self::Black => Self::White,
		}
	}
}
