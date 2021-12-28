use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::rc::Rc;

use super::mincut_maxflow::calculate_mincut_maxflow;
use super::mincut_maxflow::common::Flow;
use super::mincut_maxflow::common::FlowEdge;
use super::mincut_maxflow::common::FlowNode;
use super::mincut_maxflow::MincutMaxflow;

pub type TeamId = String;
pub type EliminatedTeams = HashMap<Rc<TeamId>, HashSet<Rc<TeamId>>>;

pub struct Tournament {
  pub teams: HashMap<Rc<TeamId>, Rc<Team>>,
  pub matches_left: HashMap<(Rc<TeamId>, Rc<TeamId>), usize>,
}

#[derive(Debug)]
pub struct Team {
  pub matches_won: usize,
  pub matches_lost: usize,
}

/// # Panics
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn predict_tournament_eliminated_teams(
  tournament: &Tournament,
) -> EliminatedTeams {
  const TEAMS_COUNT_MIN: usize = 2;
  const TEAMS_COUNT_MAX: usize = 500;
  const MATCHES_LEFT_COUNT_MIN: usize = 1;
  const MATCHES_LEFT_COUNT_MAX: usize = 1000;

  if tournament.teams.len() < TEAMS_COUNT_MIN
    || tournament.teams.len() > TEAMS_COUNT_MAX
  {
    panic!("Invalid no. of teams ({:?}).", tournament.teams.len());
  }

  if tournament.matches_left.len() < MATCHES_LEFT_COUNT_MIN
    || tournament.matches_left.len() > MATCHES_LEFT_COUNT_MAX
  {
    panic!(
      "Invalid no. of matches-left ({:?}).",
      tournament.matches_left.len()
    );
  }

  if tournament.matches_left.len()
    != tournament
      .matches_left
      .keys()
      .into_iter()
      .map(|(id1, id2)| (id1.min(id2), id1.max(id2)))
      .collect::<HashSet<_>>()
      .len()
  {
    panic!(
      "Duplicate matches-left entries ({:?}).",
      tournament.matches_left
    );
  }

  let matches_left_by_team: HashMap<Rc<TeamId>, usize> = (&tournament
    .matches_left)
    .iter()
    .flat_map(|((node1, node2), matches_left)| {
      vec![(node1, matches_left), (node2, matches_left)]
    })
    .fold(HashMap::new(), |mut map, (node, matches_left)| {
      let existing_matches_left = match map.get(node) {
        None => 0,
        Some(&existing_value) => existing_value,
      };

      map.insert(Rc::clone(node), existing_matches_left + matches_left);

      map
    });

  let source_node = FlowNode::new(Rc::new("s".to_string()));
  let sink_node = FlowNode::new(Rc::new("t".to_string()));

  let mincut_maxflow_results: HashMap<FlowNode, MincutMaxflow> = (&tournament
    .teams)
    .iter()
    .map(|(team_id, team)| -> (FlowNode, MincutMaxflow) {
      let possible_eliminating_teams_nodes: VecDeque<FlowNode> = (&tournament
        .teams)
        .iter()
        .filter(|(candidate_team_id, _)| *candidate_team_id != team_id)
        .filter(|(_, candidate_team)| {
          let max_wins =
            team.matches_won + matches_left_by_team.get(team_id).unwrap_or(&0);
          candidate_team.matches_won > max_wins
        })
        .map(|(candidate_team_id, _)| {
          FlowNode::new(Rc::clone(candidate_team_id))
        })
        .collect();

      // NOTE: Can't remember why this special-case exists. It's probably for
      // one of the following reasons:
      // 1. The mincut-maxflow algorithm/implementation can't handle it.
      // 2. Even more special-handling has to be done otherwise.
      if !possible_eliminating_teams_nodes.is_empty() {
        let mut eliminating_teams_nodes = possible_eliminating_teams_nodes;
        eliminating_teams_nodes.push_front(FlowNode::clone(&source_node));

        let mincut = eliminating_teams_nodes.into_iter().collect();
        return (
          FlowNode::new(Rc::clone(team_id)),
          MincutMaxflow::fake(mincut),
        );
      }

      let other_teams: HashMap<FlowNode, &Rc<Team>> = (&tournament.teams)
        .iter()
        .filter(|(possible_other_team_id, _)| {
          *possible_other_team_id != team_id
        })
        .map(|(other_team_id, other_team)| {
          (FlowNode::new(Rc::clone(other_team_id)), other_team)
        })
        .collect();
      let other_teams_nodes: Vec<&FlowNode> = (&other_teams)
        .iter()
        .map(|(other_team_node, _)| other_team_node)
        .collect();

      let mut other_teams_nodes_combinations: Vec<(&FlowNode, &FlowNode)> =
        Vec::new();
      for i in 0..(&other_teams_nodes).len() {
        for j in (i + 1)..(&other_teams_nodes).len() {
          other_teams_nodes_combinations
            .push(((&other_teams_nodes)[i], (&other_teams_nodes)[j]));
        }
      }

      let games_left_edges: Vec<FlowEdge> = (&other_teams_nodes_combinations)
        .iter()
        .map(|(node1, node2)| {
          let (id1, id2) = (&node1.id, &node2.id);

          FlowEdge::new(
            FlowNode::clone(&source_node),
            node1.join(node2),
            Flow::Regular(
              *(&tournament.matches_left)
                .get(&(Rc::clone(id1), Rc::clone(id2)))
                .unwrap_or_else(|| {
                  (&tournament.matches_left)
                    .get(&(Rc::clone(id2), Rc::clone(id1)))
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
            team.matches_won + matches_left_by_team.get(team_id).unwrap_or(&0);
          if other_team_wins > own_team_max_wins {
            // NOTE: This case can't happen because otherwise the function would
            // have returned earlier.
            panic!("Impossible case.");
          }
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

      (FlowNode::new(Rc::clone(team_id)), mincut_maxflow)
    })
    .collect();

  let all_teams_nodes: HashSet<FlowNode> = (&tournament.teams)
    .iter()
    .map(|(id, _)| FlowNode::new(Rc::clone(id)))
    .collect();

  let eliminated_teams: EliminatedTeams = mincut_maxflow_results
    .into_iter()
    .filter(|(_, MincutMaxflow { source_full, .. })| !source_full)
    .map(|(team_node, MincutMaxflow { mincut, .. })| {
      let eliminating_teams = mincut
        .into_iter()
        .filter(|node| all_teams_nodes.contains(node))
        .map(|node| node.id)
        .collect();

      (team_node.id, eliminating_teams)
    })
    .collect();

  eliminated_teams
}

struct TestExample {
  tournament: Tournament,
  expected_eliminated_teams: EliminatedTeams,
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  let examples = vec![
    TestExample {
      tournament: Tournament {
        teams: vec![
          (
            "atlanta",
            Team {
              matches_won: 83,
              matches_lost: 71,
            },
          ),
          (
            "philadelphia",
            Team {
              matches_won: 80,
              matches_lost: 79,
            },
          ),
          (
            "new-york",
            Team {
              matches_won: 78,
              matches_lost: 78,
            },
          ),
          (
            "montreal",
            Team {
              matches_won: 77,
              matches_lost: 82,
            },
          ),
        ]
        .into_iter()
        .map(|(team_id, team)| (Rc::new(team_id.to_string()), Rc::new(team)))
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
            (Rc::new(team_id1.to_string()), Rc::new(team_id2.to_string())),
            matches_left,
          )
        })
        .collect(),
      },
      expected_eliminated_teams: vec![
        (
          Rc::new("montreal".to_string()),
          vec!["atlanta".to_string()]
            .into_iter()
            .map(Rc::new)
            .collect(),
        ),
        (
          Rc::new("philadelphia".to_string()),
          vec!["atlanta", "new-york"]
            .into_iter()
            .map(|team_id| Rc::new(team_id.to_string()))
            .collect(),
        ),
      ]
      .into_iter()
      .collect(),
    },
    TestExample {
      tournament: Tournament {
        teams: vec![
          (
            "new-york",
            Team {
              matches_won: 75,
              matches_lost: 59,
            },
          ),
          (
            "baltimore",
            Team {
              matches_won: 71,
              matches_lost: 63,
            },
          ),
          (
            "boston",
            Team {
              matches_won: 69,
              matches_lost: 66,
            },
          ),
          (
            "toronto",
            Team {
              matches_won: 63,
              matches_lost: 72,
            },
          ),
          (
            "detroit",
            Team {
              matches_won: 49,
              matches_lost: 86,
            },
          ),
        ]
        .into_iter()
        .map(|(team_id, team)| (Rc::new(team_id.to_string()), Rc::new(team)))
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
            (Rc::new(team_id1.to_string()), Rc::new(team_id2.to_string())),
            matches_left,
          )
        })
        .collect(),
      },
      expected_eliminated_teams: vec![(
        Rc::new("detroit".to_string()),
        vec![
          "baltimore".to_string(),
          "boston".to_string(),
          "new-york".to_string(),
        ]
        .into_iter()
        .map(Rc::new)
        .collect(),
      )]
      .into_iter()
      .collect(),
    },
  ];

  for TestExample {
    tournament,
    expected_eliminated_teams,
  } in examples
  {
    assert_eq!(
      predict_tournament_eliminated_teams(&tournament),
      expected_eliminated_teams
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
