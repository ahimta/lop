use std::cmp::Ord;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::sync::Arc;

use itertools::Itertools;

pub type TeamId = Arc<String>;

#[must_use]
#[derive(Debug, Eq, PartialEq)]
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
  pub remaining_points: Option<HashMap<(TeamId, TeamId), usize>>,
  constructor_guard: PhantomData<()>,
  // NOTE(TOURNAMENT-FIELDS-CHANGE-DETECTOR)
}
impl PartialEq for Tournament {
  #[must_use]
  // NOTE(TOURNAMENT-FIELDS-CHANGE-DETECTOR)
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

impl Tournament {
  #[must_use]
  pub fn new(
    name: &Arc<String>,
    teams: BTreeSet<Arc<Team>>,
    remaining_points: Option<HashMap<(TeamId, TeamId), usize>>,
  ) -> Self {
    const NAME_LENGTH_MIN: usize = 1;
    const NAME_LENGTH_MAX: usize = 100;
    // NOTE: Reamining-points counts and teams counts are significant.
    // Teams count must be at least 2 because the provided teams may be
    // extracted from a tournament matches-results where a single match would
    // contain 2 teams. Creating tournaments with any fewer teams isn't
    // supported and doesn't make sense anyway.
    // Remaining-points count is just a by-product of teams-count and contains
    // all remaining-points for each pair of teams.
    const TEAMS_COUNT_MIN: usize = 2;
    const TEAMS_COUNT_MAX: usize = 500;
    const REMAINING_POINTS_COUNT_MIN: usize =
      min_pair_combinations(TEAMS_COUNT_MIN);
    const REMAINING_POINTS_COUNT_MAX: usize =
      max_pair_combinations(TEAMS_COUNT_MAX);

    assert!(
      name.len() >= NAME_LENGTH_MIN && name.len() <= NAME_LENGTH_MAX,
      "Invalid name ({:?}).",
      name
    );

    assert!(
      teams.len() >= TEAMS_COUNT_MIN && teams.len() <= TEAMS_COUNT_MAX,
      "Invalid no. of teams ({:?}).",
      teams.len()
    );

    remaining_points
      .as_ref()
      .map_or((), |remaining_points_value| {
        assert!(
          remaining_points_value.len() >= REMAINING_POINTS_COUNT_MIN
            && remaining_points_value.len() <= REMAINING_POINTS_COUNT_MAX
            && remaining_points_value.len()
              == max_pair_combinations(teams.len()),
          "Invalid no. of remaining-points ({:?}, {:?}).",
          remaining_points_value.len(),
          teams.len(),
        );

        assert!(
          remaining_points_value.len()
            == remaining_points_value
              .keys()
              .into_iter()
              .map(|(name1, name2)| (name1.min(name2), name1.max(name2)))
              .collect::<HashSet<_>>()
              .len(),
          "Duplicate remaining-points entries ({:?}).",
          remaining_points_value,
        );

        let teams_names: HashSet<&TeamId> =
          teams.iter().map(|team| &team.name).collect();
        assert!(
          remaining_points_value
            .keys()
            .into_iter()
            .all(|(name1, name2)| teams_names.contains(name1)
              && teams_names.contains(name2)),
          "Remaining-points includes non-existing teams ({:?}, {:?}).",
          remaining_points_value,
          teams,
        );

        let remaining_points_per_team: HashMap<&TeamId, usize> =
          remaining_points_value
            .iter()
            .flat_map(|((team_name1, team_name2), &remaining)| {
              vec![(team_name1, remaining), (team_name2, remaining)]
            })
            .into_grouping_map()
            .sum();
        if remaining_points_value.is_empty() || teams.is_empty() {
          // NOTE: This is important to check as otherwise parts of the
          // validation may break/not-run due to empty-collection.
          assert!(
            remaining_points_value.is_empty() && teams.is_empty(),
            "Invalid remaining-points & teams lengths ({:?}, {:?}).",
            remaining_points_value.len(),
            teams.len(),
          );
        }
        assert!(
          teams.iter().all(|team| team.remaining_points
            == *remaining_points_per_team.get(&team.name).unwrap_or(&0)),
          "Remaining-points conflicts with teams-stats ({:?}, {:?}).",
          remaining_points_value,
          teams,
        );
      });

    Self {
      name: Arc::clone(name),
      teams,
      remaining_points,
      constructor_guard: PhantomData,
    }
  }
}
#[must_use]
const fn min_pair_combinations(x: usize) -> usize {
  x - 1
}
#[must_use]
const fn max_pair_combinations(x: usize) -> usize {
  x * (x - 1) / 2
}

#[must_use]
#[derive(Debug)]
pub struct Team {
  pub name: TeamId,

  pub rank: usize,
  pub matches_played: usize,
  pub matches_left: usize,
  pub matches_drawn: usize,
  pub matches_won: usize,
  pub matches_lost: usize,
  pub earned_points: usize,
  pub remaining_points: usize,

  pub elimination_status: Option<EliminationStatus>,

  constructor_guard: PhantomData<()>,
  // NOTE(TEAM-FIELDS-CHANGE-DETECTOR)
}
impl PartialEq for Team {
  #[must_use]
  // NOTE(TEAM-FIELDS-CHANGE-DETECTOR)
  fn eq(&self, other: &Self) -> bool {
    if !cfg!(test) {
      return self.name == other.name;
    }

    // NOTE(EXHAUSTIVE-EQUALITY-ONLY-FOR-TESTS)
    self.name == other.name
      && self.rank == other.rank
      && self.matches_played == other.matches_played
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
    let ordering = other.earned_points.cmp(&self.earned_points);
    if ordering != Ordering::Equal {
      return ordering;
    }

    let ordering = self.rank.cmp(&other.rank);
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

impl Team {
  #[must_use]
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    name: &TeamId,
    rank: usize,
    matches_played: usize,
    matches_left: usize,
    matches_drawn: usize,
    matches_won: usize,
    matches_lost: usize,
    earned_points: usize,
    remaining_points: usize,
    elimination_status: Option<EliminationStatus>,
  ) -> Self {
    const NAME_LENGTH_MIN: usize = 1;
    const NAME_LENGTH_MAX: usize = 100;
    const ELIMINATING_TEAMS_COUNT_MIN: usize = 1;
    const ELIMINATING_TEAMS_COUNT_MAX: usize = 100;

    assert!(
      name.len() >= NAME_LENGTH_MIN && name.len() <= NAME_LENGTH_MAX,
      "Invalid name ({:?}).",
      name,
    );

    assert!(
      (1..=ELIMINATING_TEAMS_COUNT_MAX).contains(&rank),
      "Invalid rank ({:?}).",
      rank
    );

    assert!(
      matches_played == matches_drawn + matches_won + matches_lost,
      "Invalid matches-stats ({:?}, {:?}, {:?}, {:?}).",
      matches_played,
      matches_drawn,
      matches_won,
      matches_lost,
    );

    assert!(
      earned_points >= matches_drawn + matches_won,
      "Invalid earned-points ({:?}, {:?}, {:?}).",
      earned_points,
      matches_drawn,
      matches_won,
    );

    assert!(
      remaining_points >= matches_left,
      "Invalid earned-points ({:?}, {:?}).",
      remaining_points,
      matches_left,
    );

    match &elimination_status {
      None | Some(EliminationStatus::Not) => {},
      Some(
        EliminationStatus::Trivially(eliminating_teams)
        | EliminationStatus::NonTrivially(eliminating_teams),
      ) => {
        assert!(
          eliminating_teams.len() >= ELIMINATING_TEAMS_COUNT_MIN
            && eliminating_teams.len() <= ELIMINATING_TEAMS_COUNT_MAX,
          "Invalid eliminating-teams count ({:?}).",
          eliminating_teams.len(),
        );

        for eliminating_team in eliminating_teams {
          assert!(
            eliminating_team.elimination_status.is_none(),
            "Invalid elimination-team ({:?}).",
            eliminating_team,
          );
        }
      },
    }

    Self {
      name: Arc::clone(name),
      rank,
      matches_played,
      matches_left,
      matches_drawn,
      matches_won,
      matches_lost,
      earned_points,
      remaining_points,
      elimination_status,
      constructor_guard: PhantomData,
    }
  }

  #[must_use]
  pub fn with_rank(team: &Self, rank: usize) -> Self {
    assert!(
      team.elimination_status.is_none(),
      "Unexpected elimination-status ({:?})",
      team.elimination_status,
    );

    Self::new(
      &team.name,
      rank,
      team.matches_played,
      team.matches_left,
      team.matches_drawn,
      team.matches_won,
      team.matches_lost,
      team.earned_points,
      team.remaining_points,
      None,
    )
  }

  #[must_use]
  pub fn with_elimination_status(
    team: &Self,
    elimination_status: &EliminationStatus,
  ) -> Self {
    let sanitized_eliminating_teams: BTreeSet<Arc<Self>> =
      match &elimination_status {
        EliminationStatus::Not => BTreeSet::new(),
        EliminationStatus::Trivially(eliminating_teams)
        | EliminationStatus::NonTrivially(eliminating_teams) => {
          eliminating_teams
            .iter()
            .map(|eliminating_team| {
              Arc::new(Self::new(
                &eliminating_team.name,
                eliminating_team.rank,
                eliminating_team.matches_played,
                eliminating_team.matches_left,
                eliminating_team.matches_drawn,
                eliminating_team.matches_won,
                eliminating_team.matches_lost,
                eliminating_team.earned_points,
                eliminating_team.remaining_points,
                None,
              ))
            })
            .collect()
        },
      };
    let sanitized_elimination_status = match &elimination_status {
      EliminationStatus::Not => EliminationStatus::Not,
      EliminationStatus::Trivially(_) => {
        EliminationStatus::Trivially(sanitized_eliminating_teams)
      },
      EliminationStatus::NonTrivially(_) => {
        EliminationStatus::NonTrivially(sanitized_eliminating_teams)
      },
    };

    Self::new(
      &team.name,
      team.rank,
      team.matches_played,
      team.matches_left,
      team.matches_drawn,
      team.matches_won,
      team.matches_lost,
      team.earned_points,
      team.remaining_points,
      Some(sanitized_elimination_status),
    )
  }
}
