use std::collections::BTreeSet;
use std::collections::HashMap;
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
  let source_node = FlowNode::source();
  let sink_node = FlowNode::sink();

  let teams_predictions: BTreeSet<Arc<Team>> = tournament
    .teams
    .iter()
    .map(|team| -> Arc<Team> {
      match team.elimination_status {
        None => {}
        Some(_) => panic!("Team elimination-status already predicted"),
      };

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
        return Arc::new(Team::with_elimination_status(
          team,
          &EliminationStatus::Trivially(
            possible_eliminating_teams
              .into_iter()
              .map(Arc::clone)
              .collect(),
          ),
        ));
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

      let remaining_points = match &tournament.remaining_points {
        None => panic!("Missing remaining-points"),
        Some(value) => value,
      };

      let remaining_points_edges: Vec<FlowEdge> =
        other_teams_nodes_combinations
          .iter()
          .map(|(node1, node2)| {
            let (id1, id2) = (&node1.id, &node2.id);

            FlowEdge::new(
              &source_node,
              &Arc::new(node1.join(node2)),
              Flow::Regular(
                *remaining_points
                  .get(&(Arc::clone(id1), Arc::clone(id2)))
                  .unwrap_or_else(|| {
                    remaining_points
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
        return Arc::new(Team::with_elimination_status(
          team,
          &EliminationStatus::Not,
        ));
      }

      let eliminating_teams = tournament
        .teams
        .iter()
        .filter(|team| {
          mincut_maxflow.mincut.contains(&FlowNode::new(&team.name))
        })
        .map(Arc::clone)
        .collect();

      Arc::new(Team::with_elimination_status(
        team,
        &EliminationStatus::NonTrivially(eliminating_teams),
      ))
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
      tournament: Tournament::new(
        &Arc::new(String::from("dummy-tournament")),
        vec![
          Team::new(
            &Arc::new(String::from("atlanta")),
            1,
            83,
            8,
            83,
            0,
            0,
            83,
            8,
            None,
          ),
          Team::new(
            &Arc::new(String::from("philadelphia")),
            2,
            80,
            3,
            80,
            0,
            0,
            80,
            3,
            None,
          ),
          Team::new(
            &Arc::new(String::from("new-york")),
            3,
            78,
            6,
            78,
            0,
            0,
            78,
            6,
            None,
          ),
          Team::new(
            &Arc::new(String::from("montreal")),
            4,
            77,
            3,
            77,
            0,
            0,
            77,
            3,
            None,
          ),
        ]
        .into_iter()
        .map(Arc::new)
        .collect(),
        Some(
          vec![
            (("atlanta", "philadelphia"), 1),
            (("atlanta", "new-york"), 6),
            (("atlanta", "montreal"), 1),
            (("philadelphia", "montreal"), 2),
            (("philadelphia", "new-york"), 0),
            (("new-york", "montreal"), 0),
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
        ),
      ),
      expected_prediction: vec![
        Team::new(
          &Arc::new(String::from("atlanta")),
          1,
          83,
          8,
          83,
          0,
          0,
          83,
          8,
          Some(EliminationStatus::Not),
        ),
        Team::new(
          &Arc::new(String::from("philadelphia")),
          2,
          80,
          3,
          80,
          0,
          0,
          80,
          3,
          Some(EliminationStatus::NonTrivially(
            vec![
              Team::new(
                &Arc::new(String::from("atlanta")),
                1,
                83,
                8,
                83,
                0,
                0,
                83,
                8,
                None,
              ),
              Team::new(
                &Arc::new(String::from("new-york")),
                3,
                78,
                6,
                78,
                0,
                0,
                78,
                6,
                None,
              ),
            ]
            .into_iter()
            .map(Arc::new)
            .collect(),
          )),
        ),
        Team::new(
          &Arc::new(String::from("new-york")),
          3,
          78,
          6,
          78,
          0,
          0,
          78,
          6,
          Some(EliminationStatus::Not),
        ),
        Team::new(
          &Arc::new(String::from("montreal")),
          4,
          77,
          3,
          77,
          0,
          0,
          77,
          3,
          Some(EliminationStatus::Trivially(
            vec![Team::new(
              &Arc::new(String::from("atlanta")),
              1,
              83,
              8,
              83,
              0,
              0,
              83,
              8,
              None,
            )]
            .into_iter()
            .map(Arc::new)
            .collect(),
          )),
        ),
      ]
      .into_iter()
      .map(Arc::new)
      .collect(),
    },
    TestExample {
      tournament: Tournament::new(
        &Arc::new(String::from("dummy-tournament")),
        vec![
          Team::new(
            &Arc::new(String::from("new-york")),
            1,
            75,
            21,
            75,
            0,
            0,
            75,
            21,
            None,
          ),
          Team::new(
            &Arc::new(String::from("baltimore")),
            2,
            71,
            19,
            71,
            0,
            0,
            71,
            19,
            None,
          ),
          Team::new(
            &Arc::new(String::from("boston")),
            3,
            69,
            13,
            69,
            0,
            0,
            69,
            13,
            None,
          ),
          Team::new(
            &Arc::new(String::from("toronto")),
            4,
            63,
            17,
            63,
            0,
            0,
            63,
            17,
            None,
          ),
          Team::new(
            &Arc::new(String::from("detroit")),
            5,
            49,
            16,
            49,
            0,
            0,
            49,
            16,
            None,
          ),
        ]
        .into_iter()
        .map(Arc::new)
        .collect(),
        Some(
          vec![
            (("new-york", "baltimore"), 3),
            (("new-york", "boston"), 8),
            (("new-york", "toronto"), 7),
            (("new-york", "detroit"), 3),
            (("baltimore", "boston"), 2),
            (("baltimore", "toronto"), 7),
            (("baltimore", "detroit"), 7),
            (("boston", "detroit"), 3),
            (("boston", "toronto"), 0),
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
        ),
      ),
      expected_prediction: vec![
        Team::new(
          &Arc::new(String::from("new-york")),
          1,
          75,
          21,
          75,
          0,
          0,
          75,
          21,
          Some(EliminationStatus::Not),
        ),
        Team::new(
          &Arc::new(String::from("baltimore")),
          2,
          71,
          19,
          71,
          0,
          0,
          71,
          19,
          Some(EliminationStatus::Not),
        ),
        Team::new(
          &Arc::new(String::from("boston")),
          3,
          69,
          13,
          69,
          0,
          0,
          69,
          13,
          Some(EliminationStatus::Not),
        ),
        Team::new(
          &Arc::new(String::from("toronto")),
          4,
          63,
          17,
          63,
          0,
          0,
          63,
          17,
          Some(EliminationStatus::Not),
        ),
        Team::new(
          &Arc::new(String::from("detroit")),
          5,
          49,
          16,
          49,
          0,
          0,
          49,
          16,
          Some(EliminationStatus::Trivially(
            vec![
              Team::new(
                &Arc::new(String::from("new-york")),
                1,
                75,
                21,
                75,
                0,
                0,
                75,
                21,
                None,
              ),
              Team::new(
                &Arc::new(String::from("baltimore")),
                2,
                71,
                19,
                71,
                0,
                0,
                71,
                19,
                None,
              ),
              Team::new(
                &Arc::new(String::from("boston")),
                3,
                69,
                13,
                69,
                0,
                0,
                69,
                13,
                None,
              ),
            ]
            .into_iter()
            .map(Arc::new)
            .collect(),
          )),
        ),
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
