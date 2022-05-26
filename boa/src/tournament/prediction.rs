use std::cmp::Ord;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use itertools::Itertools;

use crate::common::EliminationStatus;
use crate::common::Team;
use crate::common::Tournament;
use crate::mincut_maxflow::calculate_mincut_maxflow;
use crate::mincut_maxflow::common::Flow;
use crate::mincut_maxflow::common::FlowEdge;
use crate::mincut_maxflow::common::FlowNode;

/// # Panics
#[must_use]
#[allow(clippy::too_many_lines)]
pub(super) fn predict_tournament_eliminated_teams(
  tournament: &Tournament,
) -> BTreeSet<Arc<Team>> {
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

  let source_node = FlowNode::source();
  let sink_node = FlowNode::sink();

  let teams_predictions: BTreeSet<Arc<Team>> = tournament
    .teams
    .iter()
    .map(|team| -> Arc<Team> {
      let possible_eliminating_teams: BTreeSet<&Arc<Team>> = tournament
        .teams
        .iter()
        .filter(|&candidate_team| candidate_team.name != team.name)
        .filter(|&candidate_team| {
          let max_points = team.earned_points + team.remaining_points;
          candidate_team.earned_points > max_points
        })
        .collect();

      // NOTE: Can't remember why this special-case exists. It's probably for
      // one of the following reasons:
      // 1. The mincut-maxflow algorithm/implementation can't handle it.
      // 2. Even more special-handling has to be done otherwise.
      if !possible_eliminating_teams.is_empty() {
        return Arc::new(Team {
          elimination_status: EliminationStatus::Trivially(
            possible_eliminating_teams
              .into_iter()
              .map(Arc::clone)
              .collect(),
          ),
          ..Team::clone(team)
        });
      }

      let other_teams: HashMap<Arc<FlowNode>, &Arc<Team>> = tournament
        .teams
        .iter()
        .filter(|&possible_other_team| possible_other_team.name != team.name)
        .map(|other_team| {
          (Arc::new(FlowNode::new(&other_team.name)), other_team)
        })
        .collect();
      let other_teams_nodes: Vec<&Arc<FlowNode>> = other_teams
        .iter()
        .map(|(other_team_node, _)| other_team_node)
        .collect();
      let teams_earned_points: HashMap<&Arc<FlowNode>, usize> = other_teams
        .iter()
        .map(|(node, t)| (node, t.earned_points))
        .collect();

      let other_teams_nodes_combinations: Vec<(
        &Arc<FlowNode>,
        &Arc<FlowNode>,
      )> = other_teams_nodes
        .iter()
        .combinations(2)
        .map(|nodes| (*nodes[0], *nodes[1]))
        .collect();

      let remaining_points_edges: Vec<FlowEdge> =
        other_teams_nodes_combinations
          .iter()
          .map(|(node1, node2)| {
            let (id1, id2) = (&node1.id, &node2.id);

            FlowEdge::new(
              &source_node,
              &Arc::new(node1.join(node2)),
              Flow::Regular(
                *tournament
                  .remaining_points
                  .get(&(Arc::clone(id1), Arc::clone(id2)))
                  .unwrap_or_else(|| {
                    tournament
                      .remaining_points
                      .get(&(Arc::clone(id2), Arc::clone(id1)))
                      .unwrap_or(&0)
                  }),
              ),
            )
          })
          .collect();

      let intermediate_edges =
        other_teams_nodes_combinations
          .iter()
          .flat_map(|(node1, node2)| {
            let from = Arc::new(node1.join(node2));
            let capacity = Flow::Infinite;

            vec![
              FlowEdge::new(&from, node1, capacity),
              FlowEdge::new(&from, node2, capacity),
            ]
          });

      let points_to_earn_edges: Vec<FlowEdge> = other_teams_nodes
        .iter()
        .map(|&other_team_node| {
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

          let from = other_team_node;
          let to = &sink_node;
          FlowEdge::new(from, to, capacity)
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
          mincut_maxflow.mincut.contains(&FlowNode::new(&team.name))
        })
        .map(Arc::clone)
        .collect();

      Arc::new(Team {
        elimination_status: EliminationStatus::NonTrivially(eliminating_teams),
        ..Team::clone(team)
      })
    })
    .collect();

  teams_predictions
}

#[must_use]
struct TestExample {
  tournament: Tournament,
  expected_prediction: BTreeSet<Arc<Team>>,
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  #[cfg(test)]
  use pretty_assertions::assert_eq;

  let examples = vec![
    TestExample {
      tournament: Tournament {
        name: Arc::new(String::from("dummy-tournament")),
        teams: vec![
          Team {
            name: Arc::new(String::from("atlanta")),
            rank: 1,
            matches_played: 83,
            matches_left: 8,
            matches_won: 83,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 83,
            remaining_points: 8,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new(String::from("philadelphia")),
            rank: 2,
            matches_played: 80,
            matches_left: 3,
            matches_won: 80,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 80,
            remaining_points: 3,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new(String::from("new-york")),
            rank: 3,
            matches_played: 78,
            matches_left: 6,
            matches_won: 78,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 78,
            remaining_points: 6,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new(String::from("montreal")),
            rank: 4,
            matches_played: 77,
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
              Arc::new(String::from(team_name1)),
              Arc::new(String::from(team_name2)),
            ),
            remaining_points,
          )
        })
        .collect(),
      },
      expected_prediction: vec![
        Team {
          name: Arc::new(String::from("atlanta")),
          rank: 1,
          matches_played: 83,
          matches_left: 8,
          matches_won: 83,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 83,
          remaining_points: 8,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new(String::from("philadelphia")),
          rank: 2,
          matches_played: 80,
          matches_left: 3,
          matches_won: 80,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 80,
          remaining_points: 3,
          elimination_status: EliminationStatus::NonTrivially(
            vec![
              Team {
                name: Arc::new(String::from("atlanta")),
                rank: 1,
                matches_played: 83,
                matches_left: 8,
                matches_drawn: 0,
                matches_won: 83,
                matches_lost: 0,
                earned_points: 83,
                remaining_points: 8,
                elimination_status: EliminationStatus::Not,
              },
              Team {
                name: Arc::new(String::from("new-york")),
                rank: 3,
                matches_played: 78,
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
          name: Arc::new(String::from("new-york")),
          rank: 3,
          matches_played: 78,
          matches_left: 6,
          matches_won: 78,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 78,
          remaining_points: 6,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new(String::from("montreal")),
          rank: 4,
          matches_played: 77,
          matches_left: 3,
          matches_won: 77,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 77,
          remaining_points: 3,
          elimination_status: EliminationStatus::Trivially(
            vec![Team {
              name: Arc::new(String::from("atlanta")),
              rank: 1,
              matches_played: 83,
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
        name: Arc::new(String::from("dummy-tournament")),
        teams: vec![
          Team {
            name: Arc::new(String::from("new-york")),
            rank: 1,
            matches_played: 75,
            matches_left: 4,
            matches_won: 75,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 75,
            remaining_points: 4,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new(String::from("baltimore")),
            rank: 2,
            matches_played: 71,
            matches_left: 21,
            matches_won: 71,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 71,
            remaining_points: 21,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new(String::from("boston")),
            rank: 3,
            matches_played: 69,
            matches_left: 13,
            matches_won: 69,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 69,
            remaining_points: 13,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new(String::from("toronto")),
            rank: 4,
            matches_played: 63,
            matches_left: 17,
            matches_won: 63,
            matches_drawn: 0,
            matches_lost: 0,
            earned_points: 63,
            remaining_points: 17,
            elimination_status: EliminationStatus::Not,
          },
          Team {
            name: Arc::new(String::from("detroit")),
            rank: 5,
            matches_played: 49,
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
              Arc::new(String::from(team_name1)),
              Arc::new(String::from(team_name2)),
            ),
            remaining_points,
          )
        })
        .collect(),
      },
      expected_prediction: vec![
        Team {
          name: Arc::new(String::from("new-york")),
          rank: 1,
          matches_played: 75,
          matches_left: 4,
          matches_won: 75,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 75,
          remaining_points: 4,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new(String::from("baltimore")),
          rank: 2,
          matches_played: 71,
          matches_left: 21,
          matches_won: 71,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 71,
          remaining_points: 21,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new(String::from("boston")),
          rank: 3,
          matches_played: 69,
          matches_left: 13,
          matches_won: 69,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 69,
          remaining_points: 13,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new(String::from("toronto")),
          rank: 4,
          matches_played: 63,
          matches_left: 17,
          matches_won: 63,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 63,
          remaining_points: 17,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          name: Arc::new(String::from("detroit")),
          rank: 5,
          matches_played: 49,
          matches_left: 16,
          matches_won: 49,
          matches_drawn: 0,
          matches_lost: 0,
          earned_points: 49,
          remaining_points: 16,
          elimination_status: EliminationStatus::Trivially(
            vec![
              Team {
                name: Arc::new(String::from("new-york")),
                rank: 1,
                matches_played: 75,
                matches_left: 4,
                matches_drawn: 0,
                matches_won: 75,
                matches_lost: 0,
                earned_points: 75,
                remaining_points: 4,
                elimination_status: EliminationStatus::Not,
              },
              Team {
                name: Arc::new(String::from("baltimore")),
                rank: 2,
                matches_played: 71,
                matches_left: 21,
                matches_drawn: 0,
                matches_won: 71,
                matches_lost: 0,
                earned_points: 71,
                remaining_points: 21,
                elimination_status: EliminationStatus::Not,
              },
              Team {
                name: Arc::new(String::from("boston")),
                rank: 3,
                matches_played: 69,
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
