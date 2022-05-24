use std::marker::PhantomData;
use std::sync::Arc;

use crate::tournament_fetching::fetch_tournaments;
use crate::tournament_prediction::predict_tournament_eliminated_teams;
use crate::tournament_prediction::Team;

#[must_use]
pub struct DisplayableTournament {
  pub name: String,
  pub teams: Vec<Arc<Team>>,
  constructor_guard: PhantomData<()>,
}

/// # Panics
#[must_use]
pub fn get_tournaments() -> Vec<DisplayableTournament> {
  fetch_tournaments()
    .into_iter()
    .map(|tournament| DisplayableTournament {
      name: String::clone(&tournament.name),
      teams: predict_tournament_eliminated_teams(&tournament),
      constructor_guard: PhantomData,
    })
    .collect()
}
