use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::string;
use std::sync::Arc;

use itertools::Itertools;

use crate::tournament_prediction::EliminationStatus;
use crate::tournament_prediction::Team;
use crate::tournament_prediction::TeamId;
use crate::tournament_prediction::Tournament;

const WIN_FACTOR: usize = 3;
const DRAW_FACTOR: usize = 1;

pub(super) type MatchResult = ((Arc<TeamId>, usize), (Arc<TeamId>, usize));

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
  fn download_tournaments() -> Vec<(String, Vec<String>)>;
  #[must_use]
  fn process_tournaments(
    downloaded_tournament: Vec<(String, Vec<String>)>,
  ) -> Vec<(String, Vec<MatchResult>)>;

  #[must_use]
  fn fetch_tournaments() -> Vec<Tournament> {
    let all_tournaments_matches_results =
      Self::process_tournaments(Self::download_tournaments());
    Self::postprocess_tournament(all_tournaments_matches_results)
  }

  #[allow(clippy::too_many_lines)]
  #[must_use]
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
              match first_team_score.cmp(second_team_score) {
                Ordering::Greater => (first_team_id, 1),
                Ordering::Less => (second_team_id, 1),
                Ordering::Equal => (second_team_id, 0),
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
          .flat_map(
            |(
              (first_team_id, first_team_score),
              (second_team_id, second_team_score),
            )| {
              if first_team_score != second_team_score {
                return vec![];
              }

              vec![(first_team_id, 1), (second_team_id, 1)]
            },
          )
          .into_group_map_by(|(team_id, _)| *team_id)
          .into_iter()
          .map(|(team_id, values)| {
            (team_id, values.into_iter().fold(0, |acc, (_, v)| acc + v))
          })
          .collect();

        let teams_ids: HashSet<&Arc<TeamId>> = matches_results
          .iter()
          .flat_map(|((first_team_id, _), (second_team_id, _))| {
            vec![first_team_id, second_team_id]
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

        let matches_left: HashMap<(Arc<TeamId>, Arc<TeamId>), usize> =
          teams_ids
            .iter()
            .combinations(2)
            .map(|team_pair| (team_pair[0], team_pair[1]))
            .map(|(first_team_id, second_team_id)| {
              (
                (
                  Arc::clone(first_team_id.min(second_team_id)),
                  Arc::clone(first_team_id.max(second_team_id)),
                ),
                // NOTE: From a logical perspective, we should fail here as this
                // indicates incorrect data.
                // But, in reality, it was observed that some providers can
                // respond with this invalid data.
                // Falling back to zero fixes (using saturating-sub) this and
                // produces correct results, I think. For example, assuming
                // finals (quarter/half/actual) are counted separately.
                // And investingating this behavior would be useful and we can
                // easily do it (and test the system better) by replaying
                // history and testing that no failures occur.
                MATCHES_PER_TEAM_PAIR.saturating_sub(
                  *matches_played
                    .get(&(
                      first_team_id.min(second_team_id),
                      first_team_id.max(second_team_id),
                    ))
                    .unwrap_or(&0),
                ),
              )
            })
            .collect();
        let tournament_remaining_points: HashMap<
          (Arc<TeamId>, Arc<TeamId>),
          usize,
        > = matches_left
          .iter()
          .map(|((first_team_id, second_team_id), matches_left)| {
            (
              (Arc::clone(first_team_id), Arc::clone(second_team_id)),
              matches_left.checked_mul(WIN_FACTOR).unwrap(),
            )
          })
          .collect();
        let matches_left_per_team: HashMap<&Arc<TeamId>, usize> = matches_left
          .iter()
          .flat_map(
            |((first_team_id, second_team_id), matches_left_between_pair)| {
              vec![
                (first_team_id, matches_left_between_pair),
                (second_team_id, matches_left_between_pair),
              ]
            },
          )
          .into_group_map_by(|(team_id, _)| *team_id)
          .into_iter()
          .map(|(team_id, values)| {
            (team_id, values.into_iter().fold(0, |acc, (_, v)| acc + v))
          })
          .collect();
        let remaining_points_per_team: HashMap<&Arc<TeamId>, usize> =
          matches_left_per_team
            .iter()
            .map(|(team_id, matches_left)| {
              (*team_id, matches_left.checked_mul(WIN_FACTOR).unwrap())
            })
            .collect();

        let mut teams: Vec<Arc<Team>> = teams_ids
          .iter()
          .map(|team_id| {
            let team_matches_drawn = *matches_drawn.get(team_id).unwrap_or(&0);

            Arc::new(Team {
              id: Arc::clone(team_id),

              rank: 0,
              matches_left: *matches_left_per_team.get(team_id).unwrap_or(&0),
              // FIXME: Make sure there's a test to cover this (e.g: using all
              // tournament states).
              matches_won: *matches_won.get(team_id).unwrap_or(&0),
              matches_drawn: team_matches_drawn,

              earned_points: WIN_FACTOR
                * *matches_won.get(team_id).unwrap_or(&0)
                + DRAW_FACTOR * team_matches_drawn,
              remaining_points: *remaining_points_per_team
                .get(team_id)
                .unwrap_or(&0),

              elimination_status: EliminationStatus::Not,
            })
          })
          .collect();

        // FIXME: Better ranking to match actual tournaments.
        teams.sort_unstable_by_key(|team| {
          (team.earned_points, Arc::clone(&team.id))
        });
        teams.reverse();
        let ranked_teams: Vec<Arc<Team>> = teams
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
          teams: ranked_teams,
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
