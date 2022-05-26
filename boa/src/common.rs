use std::cmp::Ord;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

pub type TeamId = Arc<String>;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EliminationStatus {
  Not,
  Trivially(BTreeSet<Arc<Team>>),
  NonTrivially(BTreeSet<Arc<Team>>),
}

#[must_use]
#[derive(Debug)]
pub struct Tournament {
  pub name: Arc<String>,
  pub teams: BTreeSet<Arc<Team>>,
  // FIXME: Make sure always validated.
  pub remaining_points: HashMap<(TeamId, TeamId), usize>,
}
impl PartialEq for Tournament {
  #[must_use]
  // NOTE(MUST-CHANGE-WHENEVER-STRUCT-FIELDS-CHANGE)
  fn eq(&self, other: &Self) -> bool {
    if !cfg!(test) {
      return self.name == other.name;
    }

    // NOTE(EXHAUSTIVE-EQUALITY-ONLY-FOR-TESTS)
    self.name == other.name
      && self.teams == other.teams
      && self.remaining_points == other.remaining_points
  }
}
impl Eq for Tournament {}

#[must_use]
#[derive(Clone, Debug)]
pub struct Team {
  pub name: TeamId,

  // FIXME: Make sure always validated.
  pub rank: usize,
  // FIXME: Make sure always validated.
  pub matches_played: usize,
  pub matches_left: usize,
  pub matches_drawn: usize,
  pub matches_won: usize,
  pub matches_lost: usize,
  pub earned_points: usize,
  pub remaining_points: usize,

  pub elimination_status: EliminationStatus,
}
impl PartialEq for Team {
  #[must_use]
  // NOTE(MUST-CHANGE-WHENEVER-STRUCT-FIELDS-CHANGE)
  fn eq(&self, other: &Self) -> bool {
    if !cfg!(test) {
      return self.name == other.name;
    }

    // NOTE(EXHAUSTIVE-EQUALITY-ONLY-FOR-TESTS)
    self.name == other.name
      && self.rank == other.rank
      && self.matches_left == other.matches_left
      && self.matches_drawn == other.matches_drawn
      && self.matches_won == other.matches_won
      && self.matches_lost == other.matches_lost
      && self.earned_points == other.earned_points
      && self.remaining_points == other.remaining_points
      && self.elimination_status == other.elimination_status
  }
}
impl Eq for Team {}
impl PartialOrd for Team {
  #[must_use]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for Team {
  #[must_use]
  fn cmp(&self, other: &Self) -> Ordering {
    let mut ordering = other.earned_points.cmp(&self.earned_points);
    if ordering != Ordering::Equal {
      return ordering;
    }

    ordering = self.rank.cmp(&other.rank);
    if ordering != Ordering::Equal {
      return ordering;
    }

    self.name.cmp(&other.name)
  }
}
impl Hash for Team {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}
