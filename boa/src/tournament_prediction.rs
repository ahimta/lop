use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use itertools::Itertools;

use super::mincut_maxflow::calculate_mincut_maxflow;
use super::mincut_maxflow::common::Flow;
use super::mincut_maxflow::common::FlowEdge;
use super::mincut_maxflow::common::FlowNode;

pub type TeamId = String;

#[derive(Eq, PartialEq, Debug)]
pub struct Tournament {
  pub name: String,
  pub teams: Vec<Arc<Team>>,
  // FIXME: Maybe replace `Arc<TeamId>` with `Arc<Team>`.
  pub matches_left: HashMap<(Arc<TeamId>, Arc<TeamId>), usize>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Team {
  pub id: Arc<TeamId>,

  // FIXME: Consider using `Option` with nullable fields.
  // FIXME: Consider adding `Team` constructors for common instantiations.

  // NOTE: `rank` and `matches_left` only used to propagate values to
  // `Prediction`.
  pub rank: usize,
  // FIXME: Make sure always validated.
  pub matches_left: usize,

  // NOTE: `matches_won` used only for prediction and can be points and not
  // necessarily actual matches won.
  pub matches_won: usize,

  // FIXME: Possible add enum with: not-eliminated|trivially-eliminated|non-trivially-eliminated|.
  pub eliminated: bool,
  pub eliminated_trivially: bool,
  // FIXME: Maybe replace `Arc<TeamId>` with `Arc<Team>`.
  // FIXME: Maybe then sort eliminating teams by rank.
  pub eliminating_teams: HashSet<Arc<TeamId>>,
}

/// # Panics
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn predict_tournament_eliminated_teams(
  tournament: &Tournament,
) -> Vec<Arc<Team>> {
  const TEAMS_COUNT_MIN: usize = 2;
  const TEAMS_COUNT_MAX: usize = 500;
  const MATCHES_LEFT_COUNT_MIN: usize = 1;
  const MATCHES_LEFT_COUNT_MAX: usize = 1000;

  assert!(
    tournament.teams.len() >= TEAMS_COUNT_MIN
      && tournament.teams.len() <= TEAMS_COUNT_MAX,
    "Invalid no. of teams ({:?}).",
    tournament.teams.len()
  );

  assert!(
    tournament.matches_left.len() >= MATCHES_LEFT_COUNT_MIN
      && tournament.matches_left.len() <= MATCHES_LEFT_COUNT_MAX,
    "Invalid no. of matches-left ({:?}).",
    tournament.matches_left.len(),
  );

  assert!(
    tournament.matches_left.len()
      == tournament
        .matches_left
        .keys()
        .into_iter()
        .map(|(id1, id2)| (id1.min(id2), id1.max(id2)))
        .collect::<HashSet<_>>()
        .len(),
    "Duplicate matches-left entries ({:?}).",
    tournament.matches_left,
  );

  let matches_left_by_team: HashMap<&Arc<TeamId>, usize> = (&tournament
    .matches_left)
    .iter()
    .flat_map(|((team_id1, team_id2), matches_left)| {
      vec![(team_id1, matches_left), (team_id2, matches_left)]
    })
    .into_group_map_by(|(team_id, _)| *team_id)
    .into_iter()
    .map(|(team_id, values)| {
      (team_id, values.into_iter().fold(0, |acc, (_, v)| acc + v))
    })
    .collect();

  let source_node = FlowNode::new(Arc::new("s".to_string()));
  let sink_node = FlowNode::new(Arc::new("t".to_string()));

  let all_teams_nodes: HashSet<FlowNode> = (&tournament.teams)
    .iter()
    .map(|team| FlowNode::new(Arc::clone(&team.id)))
    .collect();

  let eliminated_teams: Vec<Arc<Team>> = (&tournament.teams)
    .iter()
    .map(|team| -> Arc<Team> {
      let possible_eliminating_teams_nodes: HashSet<Arc<TeamId>> =
        (&tournament.teams)
          .iter()
          .filter(|candidate_team| candidate_team.id != team.id)
          .filter(|candidate_team| {
            let max_wins = team.matches_won
              + matches_left_by_team.get(&team.id).unwrap_or(&0);
            candidate_team.matches_won > max_wins
          })
          .map(|candidate_team| Arc::clone(&candidate_team.id))
          .collect();

      // NOTE: Can't remember why this special-case exists. It's probably for
      // one of the following reasons:
      // 1. The mincut-maxflow algorithm/implementation can't handle it.
      // 2. Even more special-handling has to be done otherwise.
      if !possible_eliminating_teams_nodes.is_empty() {
        return Arc::new(Team {
          eliminated: true,
          eliminated_trivially: true,
          eliminating_teams: possible_eliminating_teams_nodes,
          ..Team::clone(team)
        });
      }

      let other_teams: HashMap<FlowNode, &Arc<Team>> = (&tournament.teams)
        .iter()
        .filter(|possible_other_team| possible_other_team.id != team.id)
        .map(|other_team| {
          (FlowNode::new(Arc::clone(&other_team.id)), other_team)
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

      let games_left_edges: Vec<FlowEdge> = (&other_teams_nodes_combinations)
        .iter()
        .map(|(node1, node2)| {
          let (id1, id2) = (&node1.id, &node2.id);

          FlowEdge::new(
            FlowNode::clone(&source_node),
            node1.join(node2),
            Flow::Regular(
              *(&tournament.matches_left)
                .get(&(Arc::clone(id1), Arc::clone(id2)))
                .unwrap_or_else(|| {
                  (&tournament.matches_left)
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

      let teams_wins: HashMap<&FlowNode, usize> = (&other_teams)
        .iter()
        .map(|(node, t)| (node, t.matches_won))
        .collect();

      let game_to_win_edges: Vec<FlowEdge> = (&other_teams_nodes)
        .iter()
        .map(|other_team_node| {
          let from = other_team_node;
          let to = FlowNode::clone(&sink_node);

          let other_team_wins = *teams_wins.get(other_team_node).unwrap();
          let own_team_max_wins =
            team.matches_won + matches_left_by_team.get(&team.id).unwrap_or(&0);
          // NOTE: This case can't happen because otherwise the function would
          // have returned earlier.
          assert!(other_team_wins <= own_team_max_wins, "Impossible case.");
          let capacity = Flow::Regular(own_team_max_wins - other_team_wins);

          FlowEdge::new(FlowNode::clone(from), to, capacity)
        })
        .collect();

      let mut edges: Vec<FlowEdge> = Vec::new();
      edges.extend(games_left_edges);
      edges.extend(intermediate_edges);
      edges.extend(game_to_win_edges);

      let mincut_maxflow =
        calculate_mincut_maxflow(&edges, &source_node, &sink_node);

      if mincut_maxflow.source_full {
        return Arc::new(Team {
          eliminated: false,
          eliminated_trivially: false,
          eliminating_teams: vec![].into_iter().collect(),
          ..Team::clone(team)
        });
      }

      let eliminating_teams = mincut_maxflow
        .mincut
        .into_iter()
        .filter(|node| all_teams_nodes.contains(node))
        .map(|node| node.id)
        .collect();

      Arc::new(Team {
        eliminated: true,
        eliminated_trivially: false,
        eliminating_teams,
        ..Team::clone(team)
      })
    })
    .collect();

  eliminated_teams
}

struct TestExample {
  tournament: Tournament,
  expected_prediction: Vec<Arc<Team>>,
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  let examples = vec![
    TestExample {
      tournament: Tournament {
        name: "dummy-tournament".to_string(),
        teams: vec![
          Team {
            id: Arc::new("atlanta".to_string()),
            rank: 1,
            matches_left: 8,
            matches_won: 83,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
          Team {
            id: Arc::new("philadelphia".to_string()),
            rank: 2,
            matches_left: 3,
            matches_won: 80,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
          Team {
            id: Arc::new("new-york".to_string()),
            rank: 3,
            matches_left: 6,
            matches_won: 78,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
          Team {
            id: Arc::new("montreal".to_string()),
            rank: 4,
            matches_left: 3,
            matches_won: 77,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
        ]
        .into_iter()
        .map(Arc::new)
        .collect(),
        matches_left: vec![
          (("atlanta", "philadelphia"), 1),
          (("atlanta", "new-york"), 6),
          (("atlanta", "montreal"), 1),
          (("philadelphia", "montreal"), 2),
        ]
        .into_iter()
        .map(|((team_id1, team_id2), matches_left)| {
          (
            (
              Arc::new(team_id1.to_string()),
              Arc::new(team_id2.to_string()),
            ),
            matches_left,
          )
        })
        .collect(),
      },
      expected_prediction: vec![
        Team {
          id: Arc::new("atlanta".to_string()),
          rank: 1,
          matches_left: 8,
          matches_won: 83,
          eliminated: false,
          eliminated_trivially: false,
          eliminating_teams: vec![].into_iter().collect(),
        },
        Team {
          id: Arc::new("philadelphia".to_string()),
          rank: 2,
          matches_left: 3,
          matches_won: 80,
          eliminated: true,
          eliminated_trivially: false,
          eliminating_teams: vec!["atlanta", "new-york"]
            .into_iter()
            .map(|team_id| Arc::new(team_id.to_string()))
            .collect(),
        },
        Team {
          id: Arc::new("new-york".to_string()),
          rank: 3,
          matches_left: 6,
          matches_won: 78,
          eliminated: false,
          eliminated_trivially: false,
          eliminating_teams: vec![].into_iter().collect(),
        },
        Team {
          id: Arc::new("montreal".to_string()),
          rank: 4,
          matches_left: 3,
          // FIXME: `matches_won` would break due to points hack.
          matches_won: 77,
          eliminated: true,
          eliminated_trivially: true,
          eliminating_teams: vec!["atlanta"]
            .into_iter()
            .map(|team_id| Arc::new(team_id.to_string()))
            .collect(),
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
            id: Arc::new("new-york".to_string()),
            rank: 1,
            matches_left: 4,
            matches_won: 75,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
          Team {
            id: Arc::new("baltimore".to_string()),
            rank: 2,
            matches_left: 21,
            matches_won: 71,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
          Team {
            id: Arc::new("boston".to_string()),
            rank: 3,
            matches_left: 13,
            matches_won: 69,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
          Team {
            id: Arc::new("toronto".to_string()),
            rank: 4,
            matches_left: 17,
            matches_won: 63,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
          Team {
            id: Arc::new("detroit".to_string()),
            rank: 5,
            matches_left: 16,
            matches_won: 49,
            eliminated: false,
            eliminated_trivially: false,
            eliminating_teams: vec![].into_iter().collect(),
          },
        ]
        .into_iter()
        .map(Arc::new)
        .collect(),
        matches_left: vec![
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
        .map(|((team_id1, team_id2), matches_left)| {
          (
            (
              Arc::new(team_id1.to_string()),
              Arc::new(team_id2.to_string()),
            ),
            matches_left,
          )
        })
        .collect(),
      },
      expected_prediction: vec![
        Team {
          id: Arc::new("new-york".to_string()),
          rank: 1,
          matches_left: 4,
          matches_won: 75,
          eliminated: false,
          eliminated_trivially: false,
          eliminating_teams: vec![].into_iter().collect(),
        },
        Team {
          id: Arc::new("baltimore".to_string()),
          rank: 2,
          matches_left: 21,
          matches_won: 71,
          eliminated: false,
          eliminated_trivially: false,
          eliminating_teams: vec![].into_iter().collect(),
        },
        Team {
          id: Arc::new("boston".to_string()),
          rank: 3,
          matches_left: 13,
          matches_won: 69,
          eliminated: false,
          eliminated_trivially: false,
          eliminating_teams: vec![].into_iter().collect(),
        },
        Team {
          id: Arc::new("toronto".to_string()),
          rank: 4,
          matches_left: 17,
          matches_won: 63,
          eliminated: false,
          eliminated_trivially: false,
          eliminating_teams: vec![].into_iter().collect(),
        },
        Team {
          id: Arc::new("detroit".to_string()),
          rank: 5,
          matches_left: 16,
          matches_won: 49,
          eliminated: true,
          eliminated_trivially: true,
          eliminating_teams: vec!["baltimore", "boston", "new-york"]
            .into_iter()
            .map(|team_id| Arc::new(team_id.to_string()))
            .collect(),
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
