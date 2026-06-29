use crate::basics::State;
use crate::score::Score;
use crate::static_eval::static_eval;

pub(crate) struct SearchParameters {
	pub(crate) state: State,
	pub(crate) depth: usize,
}

pub(crate) struct SearchResult {
	pub(crate) score: Score,
	pub(crate) best_child: Option<State>,
}

pub(crate) fn search(parameters: SearchParameters) -> SearchResult {
	let SearchParameters { state, depth } = parameters;

	if depth == 0 {
		return SearchResult {
			score: static_eval(&state),
			best_child: None,
		};
	}

	let children = state.children();

	if children.len() == 0 {
		let score = if state.in_check() {
			Score::instant_loss()
		} else {
			Score::Stalemate
		};

		return SearchResult {
			score,
			best_child: None,
		};
	}

	let mut score = Score::instant_loss();
	let mut best_child = None;

	for child in children {
		let SearchResult { score: child_score, .. } = search(SearchParameters {
			state: child.clone(),
			depth: depth - 1,
		});
		let child_score = child_score.flipped().add_turn();

		if child_score > score {
			score = child_score;
			best_child = Some(child);
		}
	}

	SearchResult { score, best_child }
}

impl SearchParameters {
	pub(crate) fn standard_search(state: State, depth: usize) -> Self {
		Self { state, depth }
	}
}
