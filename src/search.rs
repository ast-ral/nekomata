use std::time::{Duration, Instant};

use crate::basics::State;
use crate::score::Score;
use crate::static_eval::static_eval;

pub(crate) struct SearchParameters<'a> {
	state: State,
	current_depth: usize,
	remaining_depth: usize,
	cutoff_min: Score,
	cutoff_max: Score,
	pv_table: &'a mut PvTable,
}

pub(crate) struct SearchResult {
	pub(crate) score: Score,
	pub(crate) pv_found: bool,
}

pub(crate) struct TimeControl {
	counter: u64,
	start: Instant,
	cutoff: Instant,
	some_move_found: bool,
}

pub(crate) struct PvTable {
	layers: Vec<Vec<State>>,
}

pub(crate) fn search(parameters: SearchParameters<'_>, time_control: &mut TimeControl) -> Option<SearchResult> {
	let SearchParameters {
		state,
		current_depth,
		remaining_depth,
		mut cutoff_min,
		cutoff_max,
		pv_table,
	} = parameters;

	if !time_control.can_continue() {
		return None;
	}

	let mut pv_found = false;

	if remaining_depth == 0 {
		return Some(SearchResult {
			score: static_eval(&state),
			pv_found,
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
			pv_found,
		});
	}

	let mut best_score = Score::instant_loss();

	// no point sorting based on static eval if we're just gonna static eval all child nodes
	if remaining_depth > 1 {
		// this looks weird but is correct
		// ascending order of eval from other perspective -> best moves from our perspective first
		children.sort_by_cached_key(|state| static_eval(&state));
	}

	for child in children {
		let SearchResult { score: child_score, pv_found: child_pv_found } = search(
			SearchParameters {
				state: child.clone(),
				current_depth: current_depth + 1,
				remaining_depth: remaining_depth - 1,
				cutoff_min: cutoff_max.flipped().sub_turn(),
				cutoff_max: cutoff_min.flipped().sub_turn(),
				pv_table,
			},
			time_control,
		)?;
		let child_score = child_score.flipped().add_turn();

		if child_score >= cutoff_max {
			return Some(SearchResult {
				score: child_score,
				pv_found,
			});
		}

		if child_score > best_score {
			best_score = child_score;

			if child_score > cutoff_min {
				if pv_table.layers.len() <= current_depth {
					pv_table.layers.resize(current_depth + 1, Vec::new());
				}

				let (split_lower, split_upper) = pv_table.layers.split_at_mut(current_depth + 1);

				let this_layer = &mut split_lower[current_depth];

				this_layer.clear();
				this_layer.push(child);

				if child_pv_found {
					this_layer.extend_from_slice(&split_upper[0]);
				}

				pv_found = true;

				cutoff_min = child_score;
			}
		}
	}

	Some(SearchResult {
		score: best_score,
		pv_found,
	})
}

impl<'a> SearchParameters<'a> {
	pub(crate) fn standard_search(state: State, depth: usize, pv_table: &'a mut PvTable) -> Self {
		Self {
			state,
			current_depth: 0,
			remaining_depth: depth,
			cutoff_min: Score::instant_loss(),
			cutoff_max: Score::instant_victory(),
			pv_table,
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

impl PvTable {
	pub(crate) fn new() -> Self {
		Self {
			layers: Vec::new(),
		}
	}

	pub(crate) fn extract_pv(&self) -> &[State] {
		&self.layers[0]
	}
}
