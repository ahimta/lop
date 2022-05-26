use std::collections::HashMap;

use crate::common::Tournament;
use crate::tournament_fetching::fetch_tournaments;
use crate::tournament_prediction::predict_tournament_eliminated_teams;

// FIXME: Rearrange so that ambassador is redundant and all tournament concerns
// are under `tournament` module.
/// # Panics
#[must_use]
pub(super) fn get_tournaments() -> Vec<Tournament> {
  fetch_tournaments()
    .into_iter()
    .map(|tournament| {
      let teams = predict_tournament_eliminated_teams(&tournament);

      Tournament {
        name: tournament.name,
        teams,
        remaining_points: HashMap::new(),
      }
    })
    .collect()
}
