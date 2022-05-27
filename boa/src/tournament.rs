mod fetching;
mod prediction;

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
      Tournament::new(&tournament.name, teams, None)
    })
    .collect()
}

pub(super) fn test() {
  fetching::test();
  prediction::test();
}
