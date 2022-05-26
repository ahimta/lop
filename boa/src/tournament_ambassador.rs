use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::tournament_fetching::fetch_tournaments;
use crate::tournament_prediction::predict_tournament_eliminated_teams;
use crate::tournament_prediction::Team;

// FIXME: Make almost everything except ambassadors private after separating
// domain types.
#[must_use]
pub struct DisplayableTournament {
  pub name: Arc<String>,
  pub teams: BTreeSet<Arc<Team>>,
  constructor_guard: PhantomData<()>,
}

/// # Panics
#[must_use]
pub fn get_tournaments() -> Vec<DisplayableTournament> {
  fetch_tournaments()
    .into_iter()
    .map(|tournament| {
      let teams = predict_tournament_eliminated_teams(&tournament);

      DisplayableTournament {
        name: tournament.name,
        teams,
        constructor_guard: PhantomData,
      }
    })
    .collect()
}
