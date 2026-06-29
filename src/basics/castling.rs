use crate::basics::layer::Layer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CastlingType {
	Kingside,
	Queenside,
}

pub(crate) struct CastlingData {
	pub(crate) king_result: Layer,
	pub(crate) delete_rook: Layer,
	pub(crate) create_rook: Layer,
	pub(crate) needs_empty: Layer,
	pub(crate) needs_safe: Layer,
}
