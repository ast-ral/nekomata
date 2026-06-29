use std::time::{Duration, Instant};

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

pub(crate) struct TimeControl {
	counter: u64,
	start: Instant,
	cutoff: Instant,
	some_move_found: bool,
}

pub(crate) fn search(parameters: SearchParameters, time_control: &mut TimeControl) -> Option<SearchResult> {
	let SearchParameters {
		state,
		depth,
		mut cutoff_min,
		cutoff_max,
	} = parameters;

	if !time_control.can_continue() {
		return None;
	}

	if depth == 0 {
		return Some(SearchResult {
			score: static_eval(&state),
			best_child: None,
		});
	}

	let mut children = state.children();

	if children.len() == 0 {
		let score = if state.in_check() {
			Score::instant_loss()
		} else {
			Score::Stalemate
		};

		return Some(SearchResult {
			score,
			best_child: None,
		});
	}

	let mut best_score = Score::instant_loss();
	let mut best_child = None;

	// no point sorting based on static eval if we're just gonna static eval all child nodes
	if depth > 1 {
		// this looks weird but is correct
		// ascending order of eval from other perspective -> best moves from our perspective first
		children.sort_by_cached_key(|state| static_eval(&state));
	}

	for child in children {
		let SearchResult { score: child_score, .. } = search(
			SearchParameters {
				state: child.clone(),
				depth: depth - 1,
				cutoff_min: cutoff_max.flipped().sub_turn(),
				cutoff_max: cutoff_min.flipped().sub_turn(),
			},
			time_control,
		)?;
		let child_score = child_score.flipped().add_turn();

		if child_score >= cutoff_max {
			return Some(SearchResult {
				score: child_score,
				best_child: None,
			});
		}

		if child_score > best_score {
			best_score = child_score;
			best_child = Some(child);

			if child_score > cutoff_min {
				cutoff_min = child_score;
			}
		}
	}

	Some(SearchResult {
		score: best_score,
		best_child,
	})
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

impl TimeControl {
	pub(crate) fn ms_from_now(ms: u64) -> Self {
		let start = Instant::now();

		Self {
			counter: 0,
			start,
			cutoff: start + Duration::from_millis(ms),
			some_move_found: false,
		}
	}

	pub(crate) fn can_continue(&mut self) -> bool {
		self.counter += 1;

		if self.some_move_found && self.counter % 100000 == 0 {
			if Instant::now() >= self.cutoff {
				return false;
			}
		}

		true
	}

	pub(crate) fn some_move_found(&mut self) {
		self.some_move_found = true;
	}

	pub(crate) fn elapsed(&self) -> u64 {
		self.start.elapsed().as_millis().try_into().unwrap()
	}

	pub(crate) fn nodes_count(&self) -> u64 {
		self.counter
	}
}
