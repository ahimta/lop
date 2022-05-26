mod fetching;
mod prediction;

use std::collections::HashMap;

use crate::common::Tournament;
use crate::tournament::fetching::fetch_tournaments;
use crate::tournament::prediction::predict_tournament_eliminated_teams;

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

pub(super) fn test() {
  fetching::test();
  prediction::test();
}
