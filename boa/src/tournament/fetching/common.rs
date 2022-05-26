use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::string;
use std::sync::Arc;

use itertools::Itertools;

use crate::common::EliminationStatus;
use crate::common::Team;
use crate::common::TeamId;
use crate::common::Tournament;

const WIN_FACTOR: usize = 3;
const DRAW_FACTOR: usize = 1;

pub(super) type MatchResult = ((TeamId, usize), (TeamId, usize));

#[must_use]
pub(super) trait TournamentProvider {
  const TEST_TOURNAMENT_NAME: &'static str;
  const TEST_DATA_FILE_ID: &'static str;
  const TEST_DATA_PREFIX: &'static str;

  // NOTE: `download_tournaments()` and `process_tournaments()` only separated
  // for easier testing.
  // FIXME: Tournament-name passed around everywhere. Probably replace with full
  // team details/stats.
  #[must_use]
  fn download_tournaments() -> Vec<(Arc<String>, Vec<String>)>;
  #[must_use]
  fn process_tournaments(
    downloaded_tournament: Vec<(Arc<String>, Vec<String>)>,
  ) -> Vec<(Arc<String>, Vec<MatchResult>)>;

  #[must_use]
  fn fetch_tournaments() -> Vec<Tournament> {
    let all_tournaments_matches_results =
      Self::process_tournaments(Self::download_tournaments());
    Self::postprocess_tournament(all_tournaments_matches_results)
  }

  #[must_use]
  #[allow(clippy::too_many_lines)]
  fn postprocess_tournament(
    all_tournaments_matches_results: Vec<(Arc<String>, Vec<MatchResult>)>,
  ) -> Vec<Tournament> {
    const MATCHES_PER_TEAM_PAIR: usize = 2;

    all_tournaments_matches_results
      .into_iter()
      .map(|(tournament_name, matches_results)| -> Tournament {
        let matches_won: HashMap<&TeamId, usize> = matches_results
          .iter()
          .map(
            |(
              (first_team_name, first_team_score),
              (second_team_name, second_team_score),
            )| {
              match first_team_score.cmp(second_team_score) {
                Ordering::Greater => (first_team_name, 1),
                Ordering::Less => (second_team_name, 1),
                Ordering::Equal => (second_team_name, 0),
              }
            },
          )
          .into_group_map_by(|(team_name, _)| *team_name)
          .into_iter()
          .map(|(team_name, values)| {
            (team_name, values.into_iter().fold(0, |acc, (_, v)| acc + v))
          })
          .collect();

        let matches_drawn: HashMap<&TeamId, usize> = matches_results
          .iter()
          .flat_map(
            |(
              (first_team_name, first_team_score),
              (second_team_name, second_team_score),
            )| {
              if first_team_score != second_team_score {
                return vec![];
              }

              vec![(first_team_name, 1), (second_team_name, 1)]
            },
          )
          .into_group_map_by(|(team_name, _)| *team_name)
          .into_iter()
          .map(|(team_name, values)| {
            (team_name, values.into_iter().fold(0, |acc, (_, v)| acc + v))
          })
          .collect();

        let matches_lost: HashMap<&TeamId, usize> = matches_results
          .iter()
          .filter_map(
            |(
              (first_team_name, first_team_score),
              (second_team_name, second_team_score),
            )| match first_team_score.cmp(second_team_score) {
              Ordering::Less => Some((first_team_name, 1)),
              Ordering::Greater => Some((second_team_name, 1)),
              Ordering::Equal => None,
            },
          )
          .into_group_map_by(|(team_name, _)| *team_name)
          .into_iter()
          .map(|(team_name, values)| {
            (team_name, values.into_iter().fold(0, |acc, (_, v)| acc + v))
          })
          .collect();

        // NOTE: Only teams that have played so far are included and we're OK
        // with this tradeoff as it doesn't affect the tournament-elimination
        // functionality and gives an almost optimal approach that only needs
        // the matches' results.
        let teams_names: HashSet<&TeamId> = matches_results
          .iter()
          .flat_map(|((first_team_name, _), (second_team_name, _))| {
            vec![first_team_name, second_team_name]
          })
          .collect();

        let matches_played: HashMap<(&TeamId, &TeamId), usize> =
          matches_results
            .iter()
            .map(|((first_team_name, _), (second_team_name, _))| {
              (
                (
                  first_team_name.min(second_team_name),
                  first_team_name.max(second_team_name),
                ),
                1,
              )
            })
            .counts_by(|(team_pair_name, _)| team_pair_name)
            .iter()
            .map(|((first_team_name, second_team_name), &played)| {
              ((*first_team_name, *second_team_name), played)
            })
            .collect();
        // FIXME: Extract collection helpers (e.g.: pair-to-per).
        let matches_played_per_team: HashMap<&TeamId, usize> = matches_played
          .iter()
          .flat_map(
            |(
              (first_team_name, second_team_name),
              matches_played_between_pair,
            )| {
              vec![
                (first_team_name, matches_played_between_pair),
                (second_team_name, matches_played_between_pair),
              ]
            },
          )
          .into_group_map_by(|(team_name, _)| *team_name)
          .into_iter()
          .map(|(team_name, values)| {
            (
              *team_name,
              values.into_iter().fold(0, |acc, (_, v)| acc + v),
            )
          })
          .collect();

        let matches_left: HashMap<(&TeamId, &TeamId), usize> = teams_names
          .iter()
          .combinations(2)
          .map(|team_pair| (team_pair[0], team_pair[1]))
          .map(|(first_team_name, second_team_name)| {
            (
              (
                *first_team_name.min(second_team_name),
                *first_team_name.max(second_team_name),
              ),
              // NOTE: From a logical perspective, we should fail here as this
              // indicates incorrect data.
              // But, in reality, it was observed that some providers can
              // respond with this invalid data.
              // Falling back to zero (using saturating-sub) fixes this and
              // produces correct results, I think. For example, assuming
              // finals (quarter/half/actual) are counted separately.
              // And investingating this behavior would be useful and we can
              // easily do it (and test the system better) by replaying
              // history and testing that no failures occur.
              MATCHES_PER_TEAM_PAIR.saturating_sub(
                *matches_played
                  .get(&(
                    first_team_name.min(second_team_name),
                    first_team_name.max(second_team_name),
                  ))
                  .unwrap_or(&0),
              ),
            )
          })
          .collect();
        let tournament_remaining_points: HashMap<(TeamId, TeamId), usize> =
          matches_left
            .iter()
            .map(|((first_team_name, second_team_name), matches_left)| {
              (
                (Arc::clone(first_team_name), Arc::clone(second_team_name)),
                matches_left.checked_mul(WIN_FACTOR).unwrap(),
              )
            })
            .collect();
        let matches_left_per_team: HashMap<&TeamId, usize> = matches_left
          .iter()
          .flat_map(
            |(
              (first_team_name, second_team_name),
              matches_left_between_pair,
            )| {
              vec![
                (first_team_name, matches_left_between_pair),
                (second_team_name, matches_left_between_pair),
              ]
            },
          )
          .into_group_map_by(|(team_name, _)| *team_name)
          .into_iter()
          .map(|(team_name, values)| {
            (
              *team_name,
              values.into_iter().fold(0, |acc, (_, v)| acc + v),
            )
          })
          .collect();
        let remaining_points_per_team: HashMap<&TeamId, usize> =
          matches_left_per_team
            .iter()
            .map(|(team_name, matches_left)| {
              (*team_name, matches_left.checked_mul(WIN_FACTOR).unwrap())
            })
            .collect();

        let teams: BTreeSet<Arc<Team>> = teams_names
          .into_iter()
          .map(|team_name| {
            let team_matches_drawn =
              *matches_drawn.get(team_name).unwrap_or(&0);

            Arc::new(Team {
              name: Arc::clone(team_name),

              rank: 0,
              matches_played: *matches_played_per_team
                .get(team_name)
                .unwrap_or(&0),
              matches_left: *matches_left_per_team.get(team_name).unwrap_or(&0),
              // FIXME: Make sure there's a test to cover this (e.g: using all
              // tournament states).
              matches_won: *matches_won.get(team_name).unwrap_or(&0),
              matches_drawn: team_matches_drawn,
              matches_lost: *matches_lost.get(team_name).unwrap_or(&0),

              earned_points: WIN_FACTOR
                * *matches_won.get(team_name).unwrap_or(&0)
                + DRAW_FACTOR * team_matches_drawn,
              remaining_points: *remaining_points_per_team
                .get(team_name)
                .unwrap_or(&0),

              elimination_status: EliminationStatus::Not,
            })
          })
          .collect::<BTreeSet<Arc<Team>>>()
          .into_iter()
          .enumerate()
          .map(|(i, team)| {
            Arc::new(Team {
              rank: i + 1,
              ..Team::clone(&team)
            })
          })
          .collect();

        Tournament {
          name: tournament_name,
          teams,
          remaining_points: tournament_remaining_points,
        }
      })
      .collect()
  }

  #[must_use]
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

  #[must_use]
  fn test_helper_download_tournaments() -> Vec<(Arc<String>, Vec<String>)> {
    use std::fs;

    return vec![(
      Arc::new(String::from(Self::TEST_TOURNAMENT_NAME)),
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
