use crate::basics::State;
use crate::score::Score;
use crate::static_eval::static_eval;

pub(crate) struct SearchParameters {
	pub(crate) state: State,
	pub(crate) depth: usize,
	pub(crate) cutoff_min: Score,
	pub(crate) cutoff_max: Score,
}

pub(crate) struct SearchResult {
	pub(crate) score: Score,
	pub(crate) best_child: Option<State>,
}

pub(crate) fn search(parameters: SearchParameters) -> SearchResult {
	let SearchParameters {
		state,
		depth,
		cutoff_min,
		cutoff_max,
	} = parameters;

	if depth == 0 {
		return SearchResult {
			score: static_eval(&state),
			best_child: None,
		};
	}

	let mut children = state.children();

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

	let mut best_score = cutoff_min;
	let mut best_child = None;

	// no point sorting based on static eval if we're just gonna static eval all child nodes
	if depth > 1 {
		// this looks weird but is correct
		// ascending order of eval from other perspective -> best moves from our perspective first
		children.sort_by_key(|state| static_eval(&state));
	}

	for child in children {
		let SearchResult { score: child_score, .. } = search(SearchParameters {
			state: child.clone(),
			depth: depth - 1,
			cutoff_min: cutoff_max.flipped().sub_turn(),
			cutoff_max: best_score.flipped().sub_turn(),
		});
		let child_score = child_score.flipped().add_turn();

		if child_score >= cutoff_max {
			return SearchResult {
				score: child_score,
				best_child: None,
			};
		}

		if child_score > best_score {
			best_score = child_score;
			best_child = Some(child);
		}
	}

	SearchResult {
		score: best_score,
		best_child,
	}
}

impl SearchParameters {
	pub(crate) fn standard_search(state: State, depth: usize) -> Self {
		Self {
			state,
			depth,
			cutoff_min: Score::instant_loss(),
			cutoff_max: Score::instant_victory(),
		}
	}
}
