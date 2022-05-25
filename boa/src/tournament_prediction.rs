use std::cmp::Ord;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use itertools::Itertools;

use crate::mincut_maxflow::calculate_mincut_maxflow;
use crate::mincut_maxflow::common::Flow;
use crate::mincut_maxflow::common::FlowEdge;
use crate::mincut_maxflow::common::FlowNode;

pub type TeamId = Arc<String>;

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EliminationStatus {
  Not,
  // FIXME: Make sure always sorted properly or use `TreeSet` everywhere when
  // teams must be sorted.
  Trivially(Vec<Arc<Team>>),
  NonTrivially(Vec<Arc<Team>>),
}

#[must_use]
#[derive(Debug)]
pub struct Tournament {
  pub name: String,
  // FIXME: Change to `HashSet<Arc<Team>>`.
  pub teams: Vec<Arc<Team>>,
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
  // FIXME: Add matches-played.
  // FIXME: Make sure always validated.
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
impl Hash for Team {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}

/// # Panics
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn predict_tournament_eliminated_teams(
  tournament: &Tournament,
) -> Vec<Arc<Team>> {
  const TEAMS_COUNT_MIN: usize = 2;
  const TEAMS_COUNT_MAX: usize = 500;
  const REMAINING_POINTS_COUNT_MIN: usize = 1;
  const REMAINING_POINTS_COUNT_MAX: usize = 1000;

  assert!(
    tournament.teams.len() >= TEAMS_COUNT_MIN
      && tournament.teams.len() <= TEAMS_COUNT_MAX,
    "Invalid no. of teams ({:?}).",
    tournament.teams.len()
  );

  assert!(
    tournament.remaining_points.len() >= REMAINING_POINTS_COUNT_MIN
      && tournament.remaining_points.len() <= REMAINING_POINTS_COUNT_MAX,
    "Invalid no. of remaining-points ({:?}).",
    tournament.remaining_points.len(),
  );

  assert!(
    tournament.remaining_points.len()
      == tournament
        .remaining_points
        .keys()
        .into_iter()
        .map(|(name1, name2)| (name1.min(name2), name1.max(name2)))
        .collect::<HashSet<_>>()
        .len(),
    "Duplicate remaining-points entries ({:?}).",
    tournament.remaining_points,
  );

  let source_node = FlowNode::new(Arc::new("s".to_string()));
  let sink_node = FlowNode::new(Arc::new("t".to_string()));

  let eliminated_teams: Vec<Arc<Team>> = (&tournament.teams)
    .iter()
    .map(|team| -> Arc<Team> {
      // FIXME: Remove unnecessary borrowing.
      let possible_eliminating_teams: Vec<Arc<Team>> = (&tournament.teams)
        .iter()
        .filter(|candidate_team| candidate_team.name != team.name)
        .filter(|candidate_team| {
          let max_points = team.earned_points + team.remaining_points;
          candidate_team.earned_points > max_points
        })
        .map(Arc::clone)
        .collect();

      // NOTE: Can't remember why this special-case exists. It's probably for
      // one of the following reasons:
      // 1. The mincut-maxflow algorithm/implementation can't handle it.
      // 2. Even more special-handling has to be done otherwise.
      if !possible_eliminating_teams.is_empty() {
        return Arc::new(Team {
          elimination_status: EliminationStatus::Trivially(
            possible_eliminating_teams,
          ),
          ..Team::clone(team)
        });
      }

      let other_teams: HashMap<FlowNode, &Arc<Team>> = (&tournament.teams)
        .iter()
        .filter(|possible_other_team| possible_other_team.name != team.name)
        .map(|other_team| {
          (FlowNode::new(Arc::clone(&other_team.name)), other_team)
        })
        .collect();
      let other_teams_nodes: Vec<&FlowNode> = (&other_teams)
        .iter()
        .map(|(other_team_node, _)| other_team_node)
        .collect();

      let other_teams_nodes_combinations: Vec<(&FlowNode, &FlowNode)> =
        other_teams_nodes
          .iter()
          .combinations(2)
          .map(|nodes| (*nodes[0], *nodes[1]))
          .collect();

      let remaining_points_edges: Vec<FlowEdge> =
        (&other_teams_nodes_combinations)
          .iter()
          .map(|(node1, node2)| {
            let (id1, id2) = (&node1.id, &node2.id);

            FlowEdge::new(
              FlowNode::clone(&source_node),
              node1.join(node2),
              Flow::Regular(
                *(&tournament.remaining_points)
                  .get(&(Arc::clone(id1), Arc::clone(id2)))
                  .unwrap_or_else(|| {
                    (&tournament.remaining_points)
                      .get(&(Arc::clone(id2), Arc::clone(id1)))
                      .unwrap_or(&0)
                  }),
              ),
            )
          })
          .collect();

      let intermediate_edges = (&other_teams_nodes_combinations)
        .iter()
        .flat_map(|(node1, node2)| {
          let from = node1.join(node2);
          let capacity = Flow::Infinite;

          vec![
            FlowEdge::new(
              FlowNode::clone(&from),
              FlowNode::clone(node1),
              capacity,
            ),
            FlowEdge::new(
              FlowNode::clone(&from),
              FlowNode::clone(node2),
              capacity,
            ),
          ]
        });

      let teams_earned_points: HashMap<&FlowNode, usize> = (&other_teams)
        .iter()
        .map(|(node, t)| (node, t.earned_points))
        .collect();

      let points_to_earn_edges: Vec<FlowEdge> = (&other_teams_nodes)
        .iter()
        .map(|other_team_node| {
          let from = other_team_node;
          let to = FlowNode::clone(&sink_node);

          let other_team_earned_points =
            *teams_earned_points.get(other_team_node).unwrap();
          let own_team_max_points = team.earned_points + team.remaining_points;
          // NOTE: This case can't happen because otherwise the function would
          // have returned earlier.
          assert!(
            other_team_earned_points <= own_team_max_points,
            "Impossible case."
          );
          let capacity =
            Flow::Regular(own_team_max_points - other_team_earned_points);

          FlowEdge::new(FlowNode::clone(from), to, capacity)
        })
        .collect();

      let mut edges: Vec<FlowEdge> = Vec::new();
      edges.extend(remaining_points_edges);
      edges.extend(intermediate_edges);
      edges.extend(points_to_earn_edges);

      let mincut_maxflow =
        calculate_mincut_maxflow(&edges, &source_node, &sink_node);

      if mincut_maxflow.source_full {
        return Arc::new(Team {
          elimination_status: EliminationStatus::Not,
          ..Team::clone(team)
        });
      }

      let eliminating_teams = tournament
        .teams
        .iter()
        .filter(|team| {
          mincut_maxflow
            .mincut
            .contains(&FlowNode::new(Arc::clone(&team.name)))
        })
        .map(Arc::clone)
        .collect();

      Arc::new(Team {
        elimination_status: EliminationStatus::NonTrivially(eliminating_teams),
        ..Team::clone(team)
      })
    })
    .collect();

  eliminated_teams
}

#[must_use]
struct TestExample {
  tournament: Tournament,
  expected_prediction: Vec<Arc<Team>>,
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  #[cfg(test)]
  use pretty_assertions::assert_eq;

  let examples = vec![
    TestExample {
      tournament: Tournament {
        name: "dummy-tournament".to_string(),
        teams: vec![
          Team {
            name: Arc::new("atlanta".to_string()),
            rank: 1,
            matches_left: 8,
            matches_won: 83,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 83,
            remaining_points: 8,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new("philadelphia".to_string()),
            rank: 2,
            matches_left: 3,
            matches_won: 80,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 80,
            remaining_points: 3,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new("new-york".to_string()),
            rank: 3,
            matches_left: 6,
            matches_won: 78,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 78,
            remaining_points: 6,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new("montreal".to_string()),
            rank: 4,
            matches_left: 3,
            matches_won: 77,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 77,
            remaining_points: 3,
            elimination_status: EliminationStatus::Not,
          },
        ]
        .into_iter()
        .map(Arc::new)
        .collect(),
        remaining_points: vec![
          (("atlanta", "philadelphia"), 1),
          (("atlanta", "new-york"), 6),
          (("atlanta", "montreal"), 1),
          (("philadelphia", "montreal"), 2),
        ]
        .into_iter()
        .map(|((team_name1, team_name2), remaining_points)| {
          (
            (
              Arc::new(team_name1.to_string()),
              Arc::new(team_name2.to_string()),
            ),
            remaining_points,
          )
        })
        .collect(),
      },
      expected_prediction: vec![
        Team {
          name: Arc::new("atlanta".to_string()),
          rank: 1,
          matches_left: 8,
          matches_won: 83,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 83,
          remaining_points: 8,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new("philadelphia".to_string()),
          rank: 2,
          matches_left: 3,
          matches_won: 80,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 80,
          remaining_points: 3,
          elimination_status: EliminationStatus::NonTrivially(
            vec![
              Team {
                name: Arc::new("atlanta".to_string()),
                rank: 1,
                matches_left: 8,
                matches_drawn: 0,
                matches_won: 83,
                matches_lost: 0,
                earned_points: 83,
                remaining_points: 8,
                elimination_status: EliminationStatus::Not,
              },
              Team {
                name: Arc::new("new-york".to_string()),
                rank: 3,
                matches_left: 6,
                matches_drawn: 0,
                matches_won: 78,
                matches_lost: 0,
                earned_points: 78,
                remaining_points: 6,
                elimination_status: EliminationStatus::Not,
              },
            ]
            .into_iter()
            .map(Arc::new)
            .collect(),
          ),
        },
        Team {
          name: Arc::new("new-york".to_string()),
          rank: 3,
          matches_left: 6,
          matches_won: 78,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 78,
          remaining_points: 6,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new("montreal".to_string()),
          rank: 4,
          matches_left: 3,
          matches_won: 77,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 77,
          remaining_points: 3,
          elimination_status: EliminationStatus::Trivially(
            vec![Team {
              name: Arc::new("atlanta".to_string()),
              rank: 1,
              matches_left: 8,
              matches_drawn: 0,
              matches_won: 83,
              matches_lost: 0,
              earned_points: 83,
              remaining_points: 8,
              elimination_status: EliminationStatus::Not,
            }]
            .into_iter()
            .map(Arc::new)
            .collect(),
          ),
        },
      ]
      .into_iter()
      .map(Arc::new)
      .collect(),
    },
    TestExample {
      tournament: Tournament {
        name: "dummy-tournament".to_string(),
        teams: vec![
          Team {
            name: Arc::new("new-york".to_string()),
            rank: 1,
            matches_left: 4,
            matches_won: 75,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 75,
            remaining_points: 4,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new("baltimore".to_string()),
            rank: 2,
            matches_left: 21,
            matches_won: 71,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 71,
            remaining_points: 21,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new("boston".to_string()),
            rank: 3,
            matches_left: 13,
            matches_won: 69,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 69,
            remaining_points: 13,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new("toronto".to_string()),
            rank: 4,
            matches_left: 17,
            matches_won: 63,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 63,
            remaining_points: 17,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new("detroit".to_string()),
            rank: 5,
            matches_left: 16,
            matches_won: 49,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 49,
            remaining_points: 16,
            elimination_status: EliminationStatus::Not,
          },
        ]
        .into_iter()
        .map(Arc::new)
        .collect(),
        remaining_points: vec![
          (("new-york", "baltimore"), 3),
          (("new-york", "boston"), 8),
          (("new-york", "toronto"), 7),
          (("new-york", "detroit"), 3),
          (("baltimore", "boston"), 2),
          (("baltimore", "toronto"), 7),
          (("baltimore", "detroit"), 7),
          (("boston", "detroit"), 3),
          (("toronto", "detroit"), 3),
        ]
        .into_iter()
        .map(|((team_name1, team_name2), remaining_points)| {
          (
            (
              Arc::new(team_name1.to_string()),
              Arc::new(team_name2.to_string()),
            ),
            remaining_points,
          )
        })
        .collect(),
      },
      expected_prediction: vec![
        Team {
          name: Arc::new("new-york".to_string()),
          rank: 1,
          matches_left: 4,
          matches_won: 75,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 75,
          remaining_points: 4,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new("baltimore".to_string()),
          rank: 2,
          matches_left: 21,
          matches_won: 71,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 71,
          remaining_points: 21,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new("boston".to_string()),
          rank: 3,
          matches_left: 13,
          matches_won: 69,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 69,
          remaining_points: 13,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new("toronto".to_string()),
          rank: 4,
          matches_left: 17,
          matches_won: 63,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 63,
          remaining_points: 17,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new("detroit".to_string()),
          rank: 5,
          matches_left: 16,
          matches_won: 49,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 49,
          remaining_points: 16,
          elimination_status: EliminationStatus::Trivially(
            vec![
              Team {
                name: Arc::new("new-york".to_string()),
                rank: 1,
                matches_left: 4,
                matches_drawn: 0,
                matches_won: 75,
                matches_lost: 0,
                earned_points: 75,
                remaining_points: 4,
                elimination_status: EliminationStatus::Not,
              },
              Team {
                name: Arc::new("baltimore".to_string()),
                rank: 2,
                matches_left: 21,
                matches_drawn: 0,
                matches_won: 71,
                matches_lost: 0,
                earned_points: 71,
                remaining_points: 21,
                elimination_status: EliminationStatus::Not,
              },
              Team {
                name: Arc::new("boston".to_string()),
                rank: 3,
                matches_left: 13,
                matches_drawn: 0,
                matches_won: 69,
                matches_lost: 0,
                earned_points: 69,
                remaining_points: 13,
                elimination_status: EliminationStatus::Not,
              },
            ]
            .into_iter()
            .map(Arc::new)
            .collect(),
          ),
        },
      ]
      .into_iter()
      .map(Arc::new)
      .collect(),
    },
  ];

  for TestExample {
    tournament,
    expected_prediction,
  } in examples
  {
    assert_eq!(
      predict_tournament_eliminated_teams(&tournament),
      expected_prediction
    );
  }
}

#[cfg(test)]
mod tests {
  use super::test;

  #[test]
  fn test_tournament_prediction() {
    test()
  }
}
