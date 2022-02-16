use std::collections::HashMap;
use std::collections::HashSet;
use std::string;
use std::sync::Arc;

use itertools::Itertools;

use crate::tournament_prediction::Team;
use crate::tournament_prediction::TeamId;
use crate::tournament_prediction::Tournament;

const WIN_FACTOR: usize = 3;
const DRAW_FACTOR: usize = 1;

pub(super) type MatchResult = ((Arc<TeamId>, f64), (Arc<TeamId>, f64));

pub(super) trait TournamentProvider {
  const TEST_TOURNAMENT_NAME: &'static str;
  const TEST_DATA_FILE_ID: &'static str;
  const TEST_DATA_PREFIX: &'static str;

  // NOTE: `download_tournaments()` and `process_tournaments()` only separated
  // for easier testing.
  // FIXME: Tournament-name passed around everywhere. Probably replace with full
  // team details/stats.
  fn download_tournaments() -> Vec<(String, Vec<String>)>;
  fn process_tournaments(
    downloaded_tournament: Vec<(String, Vec<String>)>,
  ) -> Vec<(String, Vec<MatchResult>)>;

  fn fetch_tournaments() -> Vec<Tournament> {
    let all_tournaments_matches_results =
      Self::process_tournaments(Self::download_tournaments());
    Self::postprocess_tournament(all_tournaments_matches_results)
  }

  #[allow(clippy::too_many_lines)]
  fn postprocess_tournament(
    all_tournaments_matches_results: Vec<(String, Vec<MatchResult>)>,
  ) -> Vec<Tournament> {
    const MATCHES_PER_TEAM_PAIR: usize = 2;

    all_tournaments_matches_results
      .into_iter()
      .map(|(tournament_name, matches_results)| -> Tournament {
        let matches_won: HashMap<&Arc<TeamId>, usize> = matches_results
          .iter()
          .map(
            |(
              (first_team_id, first_team_score),
              (second_team_id, second_team_score),
            )| {
              if first_team_score > second_team_score {
                (first_team_id, 1)
              } else if second_team_score > first_team_score {
                (second_team_id, 1)
              } else {
                (second_team_id, 0)
              }
            },
          )
          .into_group_map_by(|(team_id, _)| *team_id)
          .into_iter()
          .map(|(team_id, values)| {
            (team_id, values.into_iter().fold(0, |acc, (_, v)| acc + v))
          })
          .collect();

        let matches_drawn: HashMap<&Arc<TeamId>, usize> = matches_results
          .iter()
          .map(
            |(
              (first_team_id, first_team_score),
              (second_team_id, second_team_score),
            )| {
              if (first_team_score - second_team_score).abs() < 0.01 {
                (first_team_id, 1)
              } else {
                (second_team_id, 0)
              }
            },
          )
          .into_group_map_by(|(team_id, _)| *team_id)
          .into_iter()
          .map(|(team_id, values)| {
            (team_id, values.into_iter().fold(0, |acc, (_, v)| acc + v))
          })
          .collect();

        let teams: HashMap<Arc<TeamId>, Arc<Team>> = matches_results
          .iter()
          .flat_map(|((first_team_id, _), (second_team_id, _))| {
            vec![first_team_id, second_team_id]
          })
          .collect::<HashSet<&Arc<TeamId>>>()
          .into_iter()
          .map(|team_id| {
            (
              Arc::clone(team_id),
              Arc::new(Team {
                // FIXME: Make sure there's a test to cover this (e.g: using all
                // tournament states).
                matches_won: WIN_FACTOR
                  * *matches_won.get(team_id).unwrap_or(&0)
                  + DRAW_FACTOR * *matches_drawn.get(team_id).unwrap_or(&0),
              }),
            )
          })
          .collect();

        let matches_played: HashMap<(&Arc<TeamId>, &Arc<TeamId>), usize> =
          matches_results
            .iter()
            .map(|((first_team_id, _), (second_team_id, _))| {
              (
                (
                  first_team_id.min(second_team_id),
                  first_team_id.max(second_team_id),
                ),
                1,
              )
            })
            .counts_by(|(team_pair_id, _)| team_pair_id)
            .iter()
            .map(|((first_team_id, second_team_id), &played)| {
              ((*first_team_id, *second_team_id), played)
            })
            .collect();

        let matches_left: HashMap<(Arc<TeamId>, Arc<TeamId>), usize> = teams
          .iter()
          .map(|(team_id, _)| team_id)
          .combinations(2)
          .map(|team_pair| (team_pair[0], team_pair[1]))
          .map(|(first_team_id, second_team_id)| {
            (
              (
                Arc::clone(first_team_id.min(second_team_id)),
                Arc::clone(first_team_id.max(second_team_id)),
              ),
              WIN_FACTOR
                * MATCHES_PER_TEAM_PAIR
                  .checked_sub(
                    *matches_played
                      .get(&(
                        first_team_id.min(second_team_id),
                        first_team_id.max(second_team_id),
                      ))
                      .unwrap_or(&0),
                  )
                  .unwrap(),
            )
          })
          .collect();

        Tournament {
          name: tournament_name,
          teams,
          matches_left,
        }
      })
      .collect()
  }

  fn test_fetch_tournaments() -> Vec<Tournament> {
    let matches_results =
      Self::process_tournaments(Self::test_helper_download_tournaments());
    Self::postprocess_tournament(matches_results)
  }

  fn test_helper_generate_downloaded_tournaments() {
    use std::fs::File;
    use std::io::Write;

    use chrono::prelude::Utc;

    let all_tournaments_responses = Self::download_tournaments();
    for (_, responses) in all_tournaments_responses {
      let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

      let mut f =
        File::create(format!("data/{}-{}", Self::TEST_DATA_PREFIX, timestamp))
          .expect("Unable to create file");
      for response in &responses {
        f.write_all(response.as_bytes()).expect("write failed");
        f.write_all(b"\n").expect("newline write failed");
      }
    }
  }

  fn test_helper_download_tournaments() -> Vec<(String, Vec<String>)> {
    use std::fs;

    return vec![(
      Self::TEST_TOURNAMENT_NAME.to_string(),
      fs::read_to_string(format!(
        "data/{}-{}",
        Self::TEST_DATA_PREFIX,
        Self::TEST_DATA_FILE_ID,
      ))
      .expect("reading test data failed")
      .lines()
      .map(string::ToString::to_string)
      .collect(),
    )];
  }
}
