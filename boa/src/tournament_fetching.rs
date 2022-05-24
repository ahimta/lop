mod common;

use std::sync::Arc;

use reqwest::blocking::Client;

use crate::tournament_fetching::common::MatchResult;
use crate::tournament_fetching::common::TournamentProvider;
use crate::tournament_prediction::EliminationStatus;
use crate::tournament_prediction::Team;
use crate::tournament_prediction::Tournament;

#[must_use]
struct PremierLeague {}
impl TournamentProvider for PremierLeague {
  const TEST_TOURNAMENT_NAME: &'static str = "First Team - Premier League";
  const TEST_DATA_FILE_ID: &'static str = "2021-12-26T14:58:52";
  const TEST_DATA_PREFIX: &'static str = "premier-league";

  #[must_use]
  fn download_tournaments() -> Vec<(String, Vec<String>)> {
    // NOTE: Used to match exactly the value used in official page.
    const PAGE_SIZE: usize = 40;
    // NOTE: Used to prevent an infinite loop in case the API response changes.
    const ITEMS_MAX: usize = 2 * 1000;

    let client = get_client("https://www.premierleague.com");

    vec![
    ("First Team - Premier League", 1, 418, "1,2,130,131,43,4,6,7,9,26,10,11,12,23,14,20,21,33,25,38",),
    ("First Team - UEFA Champions League",2 ,424 , "47,541,48,49,51,229,52,231,4,232,83,56,58,87,1408,241,62,243,10,201,11,12,64,67,635,68,108,71,110,204,252,253,74",),
    ("First Team - UEFA Europa League", 3, 457, "541,49,50,52,505,53,494,81,84,364,58,366,989,61,93,95,96,26,369,97,98,63,202,65,66,105,470,635,106,69,1420,108,110,752,111,249,506,373,25,74",),
    ("PL2 - Primier League 2 - Division 1", 16, 438, "385,332,334,336,275,337,339,279,343,344,381,387,383,358",),
    ("PL2 - Primier League 2 - Division 2", 17, 447, "386,266,335,341,345,346,347,281,351,352,354,355,357,360",),
  ].into_iter().map(|(tournament_name, competition_id, competition_season_id, competition_teams_ids)| -> (String, Vec<String>){
    let mut page = 0;
    let mut tournament_results_pages_json_non_parsed: Vec<String> = vec![];
    loop {
      assert!(
        page * PAGE_SIZE <= ITEMS_MAX,
        "too many pages ({:?}, {:?})",
        page,
        PAGE_SIZE,
      );

      let tournament_url=format!("https://footballapi.pulselive.com/football/fixtures?comps={competition_id}&compSeasons={competition_season_id}&teams={competition_teams_ids}&page={page}&pageSize={page_size}&sort=desc&statuses=C&altIds=true",
      competition_id=competition_id,competition_season_id=competition_season_id,competition_teams_ids=competition_teams_ids, page=page, page_size=PAGE_SIZE,);
      // SEE: https://docs.rs/reqwest/0.11.7/reqwest/struct.RequestBuilder.html#method.send
      let resp = client
        // NOTE: Used to match exactly the URL used in the official page.
        // SEE: https://www.premierleague.com/results
        .get(tournament_url)
        .send()
        .unwrap()
        .text()
        .unwrap();

      let is_last_empty_page = resp.contains("\"content\":[]");

      tournament_results_pages_json_non_parsed.push(resp);
      page += 1;

      if is_last_empty_page {
        break;
      }
    }

    (tournament_name.to_string(), tournament_results_pages_json_non_parsed)
  }).collect()
  }

  #[must_use]
  fn process_tournaments(
    all_tournaments_results_pages_json_non_parsed: Vec<(String, Vec<String>)>,
  ) -> Vec<(String, Vec<MatchResult>)> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[must_use]
    struct Page {
      content: Vec<ContentItem>,
    }

    #[derive(Deserialize)]
    #[must_use]
    struct ContentItem {
      teams: (ContentItemTeam, ContentItemTeam),
    }

    #[derive(Deserialize)]
    #[must_use]
    struct ContentItemTeam {
      score: f64,
      team: ContentItemTeamTeam,
    }

    #[derive(Deserialize)]
    #[must_use]
    struct ContentItemTeamTeam {
      name: String,
    }

    all_tournaments_results_pages_json_non_parsed
      .into_iter()
      .map(
        |(tournament_name, tournament_results_pages_json_non_parsed)| -> (String, Vec<MatchResult>) {
          let matches_results: Vec<MatchResult> =
            tournament_results_pages_json_non_parsed
              .into_iter()
              .flat_map(|tournament_results_page_json_non_parsed| {
                let tournament_results_single_page_json: Page =
                  serde_json::from_str(
                    &tournament_results_page_json_non_parsed,
                  )
                  .unwrap();
                tournament_results_single_page_json
                  .content
                  .into_iter()
                  .map(|ContentItem { teams, .. }| {
                    let (first_team, second_team) = &teams;

                    (
                      // NOTE: We don't handle the case where a team wins in
                      // penalties. And we don't have because the tournaments we
                      // support, at the moment, don't need this.
                      (
                        Arc::new(first_team.team.name.clone()),
                        f64_score_to_usize(first_team.score),
                      ),
                      (
                        Arc::new(second_team.team.name.clone()),
                        f64_score_to_usize(second_team.score),
                      ),
                    )
                  })
                  .collect::<Vec<_>>()
              })
              .collect();

          (tournament_name, matches_results)
        },
      )
      .collect()
  }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
fn f64_score_to_usize(score: f64) -> usize {
  const EPSILON: f64 = 0.00001;
  const MAX: f64 = 1000.0;

  assert!(
    score >= 0f64 && (score.round() - score).abs() < EPSILON && score <= MAX,
    "Invalid input {}",
    score,
  );

  score as usize
}

#[must_use]
struct Koora {}
impl TournamentProvider for Koora {
  const TEST_TOURNAMENT_NAME: &'static str = "Saudi Professional League";
  const TEST_DATA_FILE_ID: &'static str = "2022-02-14T22:50:10";
  const TEST_DATA_PREFIX: &'static str = "koora";

  #[must_use]
  fn download_tournaments() -> Vec<(String, Vec<String>)> {
    let client = get_client("https://www.goalzz.com");

    vec![
      ("Saudi Professional League", 22551),
      ("Saudi U-13 Premier League", 22279),
      ("Saudi U-15 Premier League", 22307),
      ("Saudi Arabia U-19 League Division 1", 22277),
      ("Saudi Arabia U-17 League Division 1", 22278),
      ("Saudi Arabia Youth League - U19", 22274),
      ("Saudi U-17 Premier League", 22276),
    ]
    .into_iter()
    .map(
      |(tournament_name, competition_id)| -> (String, Vec<String>) {
        let months = vec![
          "202101", "202102", "202103", "202104", "202105", "202106", "202107",
          "202108", "202109", "202110", "202111", "202112", "202201", "202202",
          "202203", "202204", "202205", "202206", "202207", "202208", "202209",
          "202210", "202211", "202212",
        ];

        (
          tournament_name.to_string(),
          months
            .into_iter()
            .map(|current_month| -> String {
              let tournament_url = format!(
                "https://www.goalzz.com/main.aspx?c={competition_id}&stage=1&smonth={current_month}&ajax=true",
                competition_id=competition_id, current_month=current_month,
              );

              // SEE: https://docs.rs/reqwest/0.11.7/reqwest/struct.RequestBuilder.html#method.send
              let resp = client
                // NOTE: Used to match exactly the URL used in the official page.
                // SEE: https://www.goalzz.com/main.aspx?c=22551&stage=1&smonth=202108
                // SEE: https://www.kooora.com/?c=22551&stage=1&smonth=202108
                .get(tournament_url)
                .send()
                .unwrap()
                .text()
                .unwrap();

              resp.replace('\n', "")
            })
            .collect(),
        )
      },
    )
    .collect()
  }

  #[must_use]
  fn process_tournaments(
    all_tournaments_responses: Vec<(String, Vec<String>)>,
  ) -> Vec<(String, Vec<MatchResult>)> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[must_use]
    struct Table {
      matches_list: Vec<serde_json::Value>,
    }

    all_tournaments_responses
      .into_iter()
      .map(
        |(tournament_name, responses)| -> (String, Vec<MatchResult>) {
          let matches_results: Vec<MatchResult> = responses
            .into_iter()
            .flat_map(|current_response| {
              let table: Table =
                serde_json::from_str(&current_response).unwrap();
              let table_cells = table.matches_list;

              let results: Vec<MatchResult> = (0..table_cells.len())
                .filter_map(|i| -> Option<MatchResult> {
                  let cell = &table_cells[i];

                  match cell {
                    serde_json::Value::String(_) => (),
                    _ => return None,
                  }

                  let cell_value = cell.as_str().unwrap();

                  let possible_scores: Vec<&str> =
                    cell_value.split('|').collect();
                  if possible_scores.len() != 2
                    || possible_scores[0]
                      .matches(char::is_numeric)
                      .next()
                      .is_none()
                    || possible_scores[1]
                      .matches(char::is_numeric)
                      .next()
                      .is_none()
                  {
                    return None;
                  }

                  let first_team_score: usize =
                    possible_scores[0].parse().unwrap();
                  let second_team_score: usize =
                    possible_scores[1].parse().unwrap();

                  let first_team_name = table_cells[i - 2].as_str().unwrap();
                  let second_team_name = table_cells[i + 3].as_str().unwrap();

                  Some((
                    (Arc::new(first_team_name.to_string()), first_team_score),
                    (Arc::new(second_team_name.to_string()), second_team_score),
                  ))
                })
                .collect();

              results
            })
            .collect();

          (tournament_name, matches_results)
        },
      )
      .collect()
  }
}

#[must_use]
fn get_client(origin: &'static str) -> Client {
  use reqwest::header::HeaderMap;
  use reqwest::header::HeaderValue;

  // SEE: https://docs.rs/reqwest/0.11.7/reqwest/struct.ClientBuilder.html
  let client = Client::builder()
  .user_agent("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:95.0) Gecko/20100101 Firefox/95.0")
  .referer(true)
  .https_only(true)
  .gzip(true)
  .deflate(true)
  .brotli(true);

  // SEE: https://docs.rs/reqwest/0.11.7/reqwest/struct.ClientBuilder.html#method.default_headers
  let mut headers = HeaderMap::new();
  headers.insert(
    "Content-Type",
    HeaderValue::from_static(
      "application/x-www-form-urlencoded; charset=UTF-8",
    ),
  );
  headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
  headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
  headers.insert("Sec-Fetch-Site", HeaderValue::from_static("cross-site"));
  headers.insert(
    "Accept-Language",
    HeaderValue::from_static("en-US,en;q=0.5"),
  );
  headers.insert("Origin", HeaderValue::from_static(origin));

  client.default_headers(headers).build().unwrap()
}

/// # Panics
#[must_use]
pub fn fetch_tournaments() -> Vec<Tournament> {
  Koora::fetch_tournaments()
    .into_iter()
    .chain(PremierLeague::fetch_tournaments())
    .collect()
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  #[cfg(test)]
  use pretty_assertions::assert_eq;

  assert_eq!(
    Koora::test_fetch_tournaments().first().unwrap(),
    &Tournament {
      name: "Saudi Professional League".to_string(),
      teams: vec![
        Team {
          id: Arc::new("Al Ittihad".to_string()),
          rank: 1,
          matches_left: 11,
          matches_won: 15,
          matches_drawn: 2,
          matches_lost: 2,
          earned_points: 47,
          remaining_points: 33,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Shabab".to_string()),
          rank: 2,
          matches_left: 10,
          matches_won: 11,
          matches_drawn: 7,
          matches_lost: 2,
          earned_points: 40,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Nassr".to_string()),
          rank: 3,
          matches_left: 10,
          matches_won: 12,
          matches_drawn: 2,
          matches_lost: 6,
          earned_points: 38,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Hilal".to_string()),
          rank: 4,
          matches_left: 13,
          matches_won: 8,
          matches_drawn: 7,
          matches_lost: 2,
          earned_points: 31,
          remaining_points: 39,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Damac".to_string()),
          rank: 5,
          matches_left: 10,
          matches_won: 8,
          matches_drawn: 6,
          matches_lost: 6,
          earned_points: 30,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Abha".to_string()),
          rank: 6,
          matches_left: 10,
          matches_won: 8,
          matches_drawn: 4,
          matches_lost: 8,
          earned_points: 28,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Fayha".to_string()),
          rank: 7,
          matches_left: 11,
          matches_won: 6,
          matches_drawn: 8,
          matches_lost: 5,
          earned_points: 26,
          remaining_points: 33,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Raed".to_string()),
          rank: 8,
          matches_left: 10,
          matches_won: 7,
          matches_drawn: 4,
          matches_lost: 9,
          earned_points: 25,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Ahli".to_string()),
          rank: 9,
          matches_left: 10,
          matches_won: 5,
          matches_drawn: 9,
          matches_lost: 6,
          earned_points: 24,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al-Tai".to_string()),
          rank: 10,
          matches_left: 10,
          matches_won: 7,
          matches_drawn: 1,
          matches_lost: 12,
          earned_points: 22,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Taawoun".to_string()),
          rank: 11,
          matches_left: 10,
          matches_won: 5,
          matches_drawn: 6,
          matches_lost: 9,
          earned_points: 21,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Faisaly".to_string()),
          rank: 12,
          matches_left: 11,
          matches_won: 4,
          matches_drawn: 8,
          matches_lost: 7,
          earned_points: 20,
          remaining_points: 33,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Ettifaq".to_string()),
          rank: 13,
          matches_left: 11,
          matches_won: 4,
          matches_drawn: 8,
          matches_lost: 7,
          earned_points: 20,
          remaining_points: 33,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al-Batin".to_string()),
          rank: 14,
          matches_left: 10,
          matches_won: 4,
          matches_drawn: 7,
          matches_lost: 9,
          earned_points: 19,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Fateh".to_string()),
          rank: 15,
          matches_left: 11,
          matches_won: 4,
          matches_drawn: 6,
          matches_lost: 9,
          earned_points: 18,
          remaining_points: 33,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Al Hazem".to_string()),
          rank: 16,
          matches_left: 10,
          matches_won: 3,
          matches_drawn: 5,
          matches_lost: 12,
          earned_points: 14,
          remaining_points: 30,
          elimination_status: EliminationStatus::Not,
        },
      ]
      .into_iter()
      .map(Arc::new)
      .collect(),
      remaining_points: vec![
        (("Al Shabab", "Al-Tai"), 3),
        (("Al Ittihad", "Al Shabab"), 3),
        (("Al Ettifaq", "Al-Batin"), 0),
        (("Al Ittihad", "Al Taawoun"), 3),
        (("Al Ettifaq", "Damac"), 3),
        (("Al Hazem", "Al-Tai"), 3),
        (("Al Shabab", "Al-Batin"), 3),
        (("Al-Tai", "Damac"), 0),
        (("Al Ittihad", "Al Raed"), 0),
        (("Al Ettifaq", "Al Shabab"), 0),
        (("Al Taawoun", "Al-Tai"), 3),
        (("Al Hilal", "Al Taawoun"), 0),
        (("Al Hazem", "Al Shabab"), 0),
        (("Al Hazem", "Damac"), 3),
        (("Al Taawoun", "Al-Batin"), 3),
        (("Al Raed", "Al-Batin"), 3),
        (("Abha", "Damac"), 3),
        (("Al Fayha", "Damac"), 3),
        (("Abha", "Al Fateh"), 3),
        (("Abha", "Al Ittihad"), 0),
        (("Al Ahli", "Al Ittihad"), 3),
        (("Al Faisaly", "Al-Tai"), 3),
        (("Al Fateh", "Al Hilal"), 3),
        (("Al Fayha", "Al Hilal"), 3),
        (("Al Raed", "Al Shabab"), 3),
        (("Abha", "Al-Batin"), 0),
        (("Al Faisaly", "Al-Batin"), 3),
        (("Al Ettifaq", "Al Fateh"), 3),
        (("Al Fateh", "Al Hazem"), 3),
        (("Al Fateh", "Al Raed"), 0),
        (("Al Fateh", "Al Shabab"), 0),
        (("Al Ettifaq", "Al Ittihad"), 3),
        (("Al-Batin", "Al-Tai"), 0),
        (("Al Faisaly", "Al Fateh"), 6),
        (("Abha", "Al Shabab"), 0),
        (("Al Hilal", "Al Nassr"), 3),
        (("Al Nassr", "Damac"), 0),
        (("Al Fateh", "Al Ittihad"), 3),
        (("Al Faisaly", "Al Fayha"), 0),
        (("Al Fateh", "Damac"), 3),
        (("Al Ettifaq", "Al Raed"), 0),
        (("Al Fayha", "Al Raed"), 3),
        (("Al Nassr", "Al-Tai"), 0),
        (("Al Fateh", "Al-Batin"), 0),
        (("Al Hilal", "Al-Tai"), 0),
        (("Abha", "Al-Tai"), 3),
        (("Al Hazem", "Al Ittihad"), 3),
        (("Al Ettifaq", "Al Fayha"), 3),
        (("Al Faisaly", "Al Hazem"), 3),
        (("Abha", "Al Fayha"), 3),
        (("Al Faisaly", "Al Raed"), 3),
        (("Al Ahli", "Damac"), 0),
        (("Al Hazem", "Al Raed"), 0),
        (("Al Shabab", "Damac"), 0),
        (("Al Fayha", "Al-Batin"), 3),
        (("Al Ahli", "Al Shabab"), 3),
        (("Abha", "Al Hilal"), 3),
        (("Al Faisaly", "Damac"), 0),
        (("Al-Batin", "Damac"), 3),
        (("Al Faisaly", "Al Nassr"), 0),
        (("Al Raed", "Al-Tai"), 3),
        (("Abha", "Al Ahli"), 3),
        (("Al Ahli", "Al Fateh"), 0),
        (("Al Shabab", "Al Taawoun"), 3),
        (("Al Nassr", "Al Taawoun"), 0),
        (("Al Ahli", "Al Ettifaq"), 3),
        (("Al Fayha", "Al Ittihad"), 0),
        (("Al Ahli", "Al-Batin"), 3),
        (("Al Hilal", "Al Raed"), 3),
        (("Al Faisaly", "Al Hilal"), 3),
        (("Abha", "Al Ettifaq"), 0),
        (("Al Ahli", "Al-Tai"), 3),
        (("Al Fateh", "Al Nassr"), 3),
        (("Al Nassr", "Al-Batin"), 3),
        (("Al Fayha", "Al Nassr"), 3),
        (("Al Raed", "Damac"), 3),
        (("Al Hilal", "Damac"), 3),
        (("Al Nassr", "Al Raed"), 3),
        (("Al Hilal", "Al Ittihad"), 6),
        (("Al Hilal", "Al Shabab"), 3),
        (("Abha", "Al Taawoun"), 3),
        (("Al Fayha", "Al Hazem"), 3),
        (("Al Hazem", "Al Taawoun"), 0),
        (("Al Taawoun", "Damac"), 3),
        (("Abha", "Al Hazem"), 0),
        (("Al Nassr", "Al Shabab"), 3),
        (("Al Ahli", "Al Raed"), 3),
        (("Al Fayha", "Al Shabab"), 3),
        (("Al Faisaly", "Al Ittihad"), 0),
        (("Al Ahli", "Al Taawoun"), 0),
        (("Al Fateh", "Al Taawoun"), 3),
        (("Abha", "Al Nassr"), 3),
        (("Al Hazem", "Al Nassr"), 3),
        (("Al Ittihad", "Al-Tai"), 3),
        (("Al Fateh", "Al Fayha"), 0),
        (("Al Ettifaq", "Al Hilal"), 3),
        (("Al Ahli", "Al Fayha"), 3),
        (("Al Hilal", "Al-Batin"), 0),
        (("Al Ahli", "Al Hilal"), 3),
        (("Al Ittihad", "Al Nassr"), 0),
        (("Al Ahli", "Al Faisaly"), 0),
        (("Al Raed", "Al Taawoun"), 0),
        (("Al Ettifaq", "Al Taawoun"), 3),
        (("Al Fayha", "Al Taawoun"), 3),
        (("Al Hazem", "Al-Batin"), 3),
        (("Al Ittihad", "Al-Batin"), 3),
        (("Al Ahli", "Al Nassr"), 3),
        (("Al Fateh", "Al-Tai"), 3),
        (("Abha", "Al Raed"), 3),
        (("Al Fayha", "Al-Tai"), 0),
        (("Al Ahli", "Al Hazem"), 0),
        (("Abha", "Al Faisaly"), 3),
        (("Al Ettifaq", "Al-Tai"), 3),
        (("Al Ettifaq", "Al Nassr"), 3),
        (("Al Ettifaq", "Al Faisaly"), 3),
        (("Al Faisaly", "Al Shabab"), 3),
        (("Al Faisaly", "Al Taawoun"), 3),
        (("Al Hazem", "Al Hilal"), 3),
        (("Al Ettifaq", "Al Hazem"), 3),
        (("Al Ittihad", "Damac"), 3),
      ]
      .into_iter()
      .map(|((first_team, second_team), remaining_points)| (
        (
          Arc::new(first_team.to_string()),
          Arc::new(second_team.to_string())
        ),
        remaining_points,
      ))
      .collect(),
    }
  );

  assert_eq!(
    PremierLeague::test_fetch_tournaments().first().unwrap(),
    &Tournament {
      name: "First Team - Premier League".to_string(),
      teams: vec![
        Team {
          id: Arc::new("Manchester City".to_string()),
          rank: 1,
          matches_left: 20,
          matches_won: 14,
          matches_drawn: 2,
          matches_lost: 2,
          earned_points: 44,
          remaining_points: 60,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Liverpool".to_string()),
          rank: 2,
          matches_left: 20,
          matches_won: 12,
          matches_drawn: 5,
          matches_lost: 1,
          earned_points: 41,
          remaining_points: 60,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Chelsea".to_string()),
          rank: 3,
          matches_left: 20,
          matches_won: 11,
          matches_drawn: 5,
          matches_lost: 2,
          earned_points: 38,
          remaining_points: 60,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Arsenal".to_string()),
          rank: 4,
          matches_left: 20,
          matches_won: 10,
          matches_drawn: 2,
          matches_lost: 6,
          earned_points: 32,
          remaining_points: 60,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("West Ham United".to_string()),
          rank: 5,
          matches_left: 21,
          matches_won: 8,
          matches_drawn: 4,
          matches_lost: 5,
          earned_points: 28,
          remaining_points: 63,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Manchester United".to_string()),
          rank: 6,
          matches_left: 22,
          matches_won: 8,
          matches_drawn: 3,
          matches_lost: 5,
          earned_points: 27,
          remaining_points: 66,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Tottenham Hotspur".to_string()),
          rank: 7,
          matches_left: 23,
          matches_won: 8,
          matches_drawn: 2,
          matches_lost: 5,
          earned_points: 26,
          remaining_points: 69,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Wolverhampton Wanderers".to_string()),
          rank: 8,
          matches_left: 20,
          matches_won: 7,
          matches_drawn: 4,
          matches_lost: 7,
          earned_points: 25,
          remaining_points: 60,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Leicester City".to_string()),
          rank: 9,
          matches_left: 22,
          matches_won: 6,
          matches_drawn: 4,
          matches_lost: 6,
          earned_points: 22,
          remaining_points: 66,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Aston Villa".to_string()),
          rank: 10,
          matches_left: 21,
          matches_won: 7,
          matches_drawn: 1,
          matches_lost: 9,
          earned_points: 22,
          remaining_points: 63,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Crystal Palace".to_string()),
          rank: 11,
          matches_left: 21,
          matches_won: 4,
          matches_drawn: 8,
          matches_lost: 5,
          earned_points: 20,
          remaining_points: 63,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Brighton and Hove Albion".to_string()),
          rank: 12,
          matches_left: 22,
          matches_won: 4,
          matches_drawn: 8,
          matches_lost: 4,
          earned_points: 20,
          remaining_points: 66,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Brentford".to_string()),
          rank: 13,
          matches_left: 22,
          matches_won: 5,
          matches_drawn: 5,
          matches_lost: 6,
          earned_points: 20,
          remaining_points: 66,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Everton".to_string()),
          rank: 14,
          matches_left: 21,
          matches_won: 5,
          matches_drawn: 4,
          matches_lost: 8,
          earned_points: 19,
          remaining_points: 63,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Southampton".to_string()),
          rank: 15,
          matches_left: 21,
          matches_won: 3,
          matches_drawn: 8,
          matches_lost: 6,
          earned_points: 17,
          remaining_points: 63,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Leeds United".to_string()),
          rank: 16,
          matches_left: 20,
          matches_won: 3,
          matches_drawn: 7,
          matches_lost: 8,
          earned_points: 16,
          remaining_points: 60,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Watford".to_string()),
          rank: 17,
          matches_left: 22,
          matches_won: 4,
          matches_drawn: 1,
          matches_lost: 11,
          earned_points: 13,
          remaining_points: 66,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Burnley".to_string()),
          rank: 18,
          matches_left: 23,
          matches_won: 1,
          matches_drawn: 8,
          matches_lost: 6,
          earned_points: 11,
          remaining_points: 69,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Norwich City".to_string()),
          rank: 19,
          matches_left: 21,
          matches_won: 2,
          matches_drawn: 4,
          matches_lost: 11,
          earned_points: 10,
          remaining_points: 63,
          elimination_status: EliminationStatus::Not,
        },
        Team {
          id: Arc::new("Newcastle United".to_string()),
          rank: 20,
          matches_left: 20,
          matches_won: 1,
          matches_drawn: 7,
          matches_lost: 10,
          earned_points: 10,
          remaining_points: 60,
          elimination_status: EliminationStatus::Not,
        },
      ]
      .into_iter()
      .map(Arc::new)
      .collect(),
      remaining_points: vec![
        (("Arsenal", "Leeds United"), 3),
        (("Brentford", "Burnley"), 3),
        (("Arsenal", "Manchester United"), 3),
        (("Liverpool", "Newcastle United"), 3),
        (("Manchester City", "Tottenham Hotspur"), 3),
        (("Manchester United", "Newcastle United"), 3),
        (("Newcastle United", "Watford"), 3),
        (("Brentford", "Everton"), 3),
        (("Leicester City", "Southampton"), 3),
        (("Arsenal", "Crystal Palace"), 3),
        (("Leeds United", "Wolverhampton Wanderers"), 3),
        (("Chelsea", "Tottenham Hotspur"), 3),
        (("Newcastle United", "Norwich City"), 3),
        (("Liverpool", "Wolverhampton Wanderers"), 3),
        (("Brighton and Hove Albion", "Leicester City"), 3),
        (("Tottenham Hotspur", "West Ham United"), 3),
        (("Crystal Palace", "Liverpool"), 3),
        (("Newcastle United", "Tottenham Hotspur"), 3),
        (("Liverpool", "Manchester City"), 3),
        (("Liverpool", "Watford"), 3),
        (("Aston Villa", "Leicester City"), 3),
        (("Brighton and Hove Albion", "Burnley"), 3),
        (("Newcastle United", "Wolverhampton Wanderers"), 3),
        (("Chelsea", "Liverpool"), 3),
        (("Leicester City", "Manchester United"), 3),
        (("Tottenham Hotspur", "Watford"), 3),
        (("Manchester United", "Tottenham Hotspur"), 3),
        (("Everton", "Norwich City"), 3),
        (("Chelsea", "Wolverhampton Wanderers"), 3),
        (("Chelsea", "West Ham United"), 3),
        (("Brentford", "Leicester City"), 3),
        (("Brentford", "Crystal Palace"), 3),
        (("Aston Villa", "Brentford"), 3),
        (("Brentford", "Leeds United"), 3),
        (("Arsenal", "Southampton"), 3),
        (("Aston Villa", "Watford"), 3),
        (("Burnley", "Liverpool"), 3),
        (("Brighton and Hove Albion", "Liverpool"), 3),
        (("Brentford", "Tottenham Hotspur"), 3),
        (("Aston Villa", "Wolverhampton Wanderers"), 3),
        (("Brentford", "Watford"), 3),
        (("Norwich City", "Watford"), 3),
        (("Crystal Palace", "Manchester City"), 3),
        (("Brighton and Hove Albion", "West Ham United"), 3),
        (("Everton", "Leeds United"), 3),
        (("Leeds United", "Watford"), 3),
        (("Everton", "Manchester City"), 3),
        (("Aston Villa", "Newcastle United"), 3),
        (("Chelsea", "Manchester United"), 3),
        (("Burnley", "Manchester City"), 3),
        (("Leeds United", "West Ham United"), 3),
        (("Crystal Palace", "Newcastle United"), 3),
        (("Manchester United", "Southampton"), 3),
        (("Crystal Palace", "Everton"), 3),
        (("Brighton and Hove Albion", "Norwich City"), 3),
        (("Manchester United", "Wolverhampton Wanderers"), 3),
        (("Aston Villa", "Chelsea"), 3),
        (("Manchester City", "Southampton"), 3),
        (("Chelsea", "Leeds United"), 3),
        (("Leicester City", "Wolverhampton Wanderers"), 3),
        (("Brentford", "Chelsea"), 3),
        (("Aston Villa", "Manchester United"), 3),
        (("Southampton", "Watford"), 3),
        (("Burnley", "Leeds United"), 3),
        (("Leeds United", "Southampton"), 3),
        (("Crystal Palace", "West Ham United"), 3),
        (("Aston Villa", "Crystal Palace"), 3),
        (("Aston Villa", "West Ham United"), 3),
        (("Brighton and Hove Albion", "Everton"), 3),
        (("Arsenal", "Norwich City"), 3),
        (("Brentford", "Liverpool"), 3),
        (("Everton", "Liverpool"), 3),
        (("Arsenal", "Brentford"), 3),
        (("Aston Villa", "Liverpool"), 3),
        (("Aston Villa", "Brighton and Hove Albion"), 3),
        (("Chelsea", "Norwich City"), 3),
        (("Brighton and Hove Albion", "Southampton"), 3),
        (("Aston Villa", "Norwich City"), 3),
        (("Tottenham Hotspur", "Wolverhampton Wanderers"), 3),
        (("Burnley", "Wolverhampton Wanderers"), 3),
        (("Brentford", "Wolverhampton Wanderers"), 3),
        (("Aston Villa", "Everton"), 3),
        (("Everton", "Manchester United"), 3),
        (("Brentford", "Newcastle United"), 3),
        (("Liverpool", "Manchester United"), 3),
        (("Brighton and Hove Albion", "Newcastle United"), 3),
        (("Burnley", "Southampton"), 3),
        (("Leeds United", "Tottenham Hotspur"), 3),
        (("Everton", "Southampton"), 3),
        (("Aston Villa", "Manchester City"), 3),
        (("Chelsea", "Everton"), 3),
        (("Arsenal", "Aston Villa"), 3),
        (("Burnley", "Newcastle United"), 3),
        (("Arsenal", "Liverpool"), 3),
        (("Burnley", "West Ham United"), 3),
        (("Leeds United", "Norwich City"), 3),
        (("Leicester City", "West Ham United"), 3),
        (("Aston Villa", "Tottenham Hotspur"), 3),
        (("Chelsea", "Crystal Palace"), 3),
        (("Chelsea", "Leicester City"), 3),
        (("Burnley", "Norwich City"), 3),
        (("Leeds United", "Liverpool"), 3),
        (("Chelsea", "Newcastle United"), 3),
        (("Leeds United", "Leicester City"), 3),
        (("Arsenal", "Burnley"), 3),
        (("Manchester United", "Norwich City"), 3),
        (("Arsenal", "West Ham United"), 3),
        (("Crystal Palace", "Tottenham Hotspur"), 3),
        (("Burnley", "Chelsea"), 3),
        (("Norwich City", "Wolverhampton Wanderers"), 3),
        (("Brighton and Hove Albion", "Leeds United"), 3),
        (("Arsenal", "Tottenham Hotspur"), 3),
        (("Brentford", "West Ham United"), 3),
        (("Manchester City", "Manchester United"), 3),
        (("West Ham United", "Wolverhampton Wanderers"), 3),
        (("Manchester City", "Norwich City"), 3),
        (("Everton", "Watford"), 3),
        (("Norwich City", "Tottenham Hotspur"), 3),
        (("Everton", "Wolverhampton Wanderers"), 3),
        (("Arsenal", "Newcastle United"), 3),
        (("Arsenal", "Watford"), 3),
        (("Leeds United", "Newcastle United"), 3),
        (("Crystal Palace", "Southampton"), 3),
        (("Crystal Palace", "Manchester United"), 3),
        (("Brighton and Hove Albion", "Manchester City"), 3),
        (("Crystal Palace", "Leeds United"), 3),
        (("Liverpool", "Southampton"), 3),
        (("Brighton and Hove Albion", "Wolverhampton Wanderers"), 3),
        (("Manchester City", "West Ham United"), 3),
        (("Brentford", "Norwich City"), 3),
        (("Leeds United", "Manchester United"), 3),
        (("Southampton", "Wolverhampton Wanderers"), 3),
        (("Brighton and Hove Albion", "Crystal Palace"), 3),
        (("Manchester City", "Watford"), 3),
        (("Leicester City", "Newcastle United"), 3),
        (("Newcastle United", "Southampton"), 3),
        (("Manchester City", "Newcastle United"), 3),
        (("Norwich City", "Southampton"), 3),
        (("Burnley", "Everton"), 3),
        (("Crystal Palace", "Wolverhampton Wanderers"), 3),
        (("Chelsea", "Southampton"), 3),
        (("Arsenal", "Brighton and Hove Albion"), 3),
        (("Chelsea", "Manchester City"), 3),
        (("Brentford", "Brighton and Hove Albion"), 3),
        (("Watford", "Wolverhampton Wanderers"), 3),
        (("Leeds United", "Manchester City"), 3),
        (("Arsenal", "Leicester City"), 3),
        (("Arsenal", "Everton"), 3),
        (("Burnley", "Crystal Palace"), 3),
        (("Southampton", "West Ham United"), 3),
        (("Liverpool", "Tottenham Hotspur"), 3),
        (("Newcastle United", "West Ham United"), 3),
        (("Arsenal", "Manchester City"), 3),
        (("Arsenal", "Chelsea"), 3),
        (("Manchester United", "Watford"), 3),
        (("Manchester United", "West Ham United"), 3),
        (("Crystal Palace", "Leicester City"), 3),
        (("Leicester City", "Watford"), 3),
        (("Leicester City", "Norwich City"), 3),
        (("Everton", "Tottenham Hotspur"), 3),
        (("Manchester City", "Wolverhampton Wanderers"), 3),
        (("Liverpool", "West Ham United"), 3),
        (("Chelsea", "Watford"), 3),
        (("Everton", "West Ham United"), 3),
        (("Aston Villa", "Southampton"), 3),
        (("Brighton and Hove Albion", "Watford"), 3),
        (("Leicester City", "Manchester City"), 3),
        (("Liverpool", "Norwich City"), 3),
        (("Burnley", "Leicester City"), 3),
        (("Brighton and Hove Albion", "Chelsea"), 6),
        (("Aston Villa", "Burnley"), 6),
        (("Crystal Palace", "Norwich City"), 6),
        (("Brentford", "Manchester City"), 6),
        (("Burnley", "Manchester United"), 6),
        (("Burnley", "Watford"), 6),
        (("Watford", "West Ham United"), 6),
        (("Brentford", "Manchester United"), 6),
        (("Leicester City", "Liverpool"), 6),
        (("Burnley", "Tottenham Hotspur"), 6),
        (("Leicester City", "Tottenham Hotspur"), 6),
        (("Crystal Palace", "Watford"), 6),
        (("Aston Villa", "Leeds United"), 6),
        (("Brentford", "Southampton"), 6),
        (("Brighton and Hove Albion", "Tottenham Hotspur"), 6),
        (("Everton", "Newcastle United"), 6),
        (("Brighton and Hove Albion", "Manchester United"), 6),
        (("Arsenal", "Wolverhampton Wanderers"), 6),
        (("Everton", "Leicester City"), 6),
        (("Norwich City", "West Ham United"), 6),
        (("Southampton", "Tottenham Hotspur"), 6),
      ]
      .into_iter()
      .map(|((first_team, second_team), remaining_points)| (
        (
          Arc::new(first_team.to_string()),
          Arc::new(second_team.to_string())
        ),
        remaining_points,
      ))
      .collect(),
    }
  );
}

#[cfg(test)]
mod tests {
  use super::test;

  #[test]
  fn test_tournament_fetching() {
    test()
  }
}
