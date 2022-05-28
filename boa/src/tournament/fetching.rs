mod common;

use std::sync::Arc;

use reqwest::blocking::Client;

use crate::common::Team;
use crate::common::Tournament;
use crate::tournament::fetching::common::MatchResult;
use crate::tournament::fetching::common::TournamentProvider;

#[must_use]
struct PremierLeague {}
#[must_use]
impl TournamentProvider for PremierLeague {
  const TEST_TOURNAMENT_NAME: &'static str = "First Team - Premier League";
  const TEST_DATA_FILE_ID: &'static str = "2021-12-26T14:58:52";
  const TEST_DATA_PREFIX: &'static str = "premier-league";

  #[must_use]
  fn download_tournaments() -> Vec<(Arc<String>, Vec<String>)> {
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
  ].into_iter().map(|(tournament_name, competition_id, competition_season_id, competition_teams_ids)| -> (Arc<String>, Vec<String>){
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

    (Arc::new(String::from(tournament_name)), tournament_results_pages_json_non_parsed)
  }).collect()
  }

  #[must_use]
  fn process_tournaments(
    all_tournaments_results_pages_json_non_parsed: Vec<(
      Arc<String>,
      Vec<String>,
    )>,
  ) -> Vec<(Arc<String>, Vec<MatchResult>)> {
    use serde::Deserialize;

    #[must_use]
    #[derive(Deserialize)]
    struct Page {
      content: Vec<ContentItem>,
    }

    #[must_use]
    #[derive(Deserialize)]
    struct ContentItem {
      teams: (ContentItemTeam, ContentItemTeam),
    }

    #[must_use]
    #[derive(Deserialize)]
    struct ContentItemTeam {
      score: f64,
      team: ContentItemTeamTeam,
    }

    #[must_use]
    #[derive(Deserialize)]
    struct ContentItemTeamTeam {
      name: String,
    }

    all_tournaments_results_pages_json_non_parsed
      .into_iter()
      .map(
        |(tournament_name, tournament_results_pages_json_non_parsed)| -> (Arc<String>, Vec<MatchResult>) {
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
                    let (first_team, second_team) = teams;

                    (
                      // NOTE: We don't handle the case where a team wins in
                      // penalties. And we don't have to because the tournaments
                      // we support, at the moment, don't need this.
                      (
                        Arc::new(first_team.team.name),
                        f64_score_to_usize(first_team.score),
                      ),
                      (
                        Arc::new(second_team.team.name),
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

#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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
#[must_use]
impl TournamentProvider for Koora {
  const TEST_TOURNAMENT_NAME: &'static str = "Saudi Professional League";
  const TEST_DATA_FILE_ID: &'static str = "2022-02-14T22:50:10";
  const TEST_DATA_PREFIX: &'static str = "koora";

  #[must_use]
  fn download_tournaments() -> Vec<(Arc<String>, Vec<String>)> {
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
      |(tournament_name, competition_id)| -> (Arc<String>, Vec<String>) {
        let months = vec![
          "202101", "202102", "202103", "202104", "202105", "202106", "202107",
          "202108", "202109", "202110", "202111", "202112", "202201", "202202",
          "202203", "202204", "202205", "202206", "202207", "202208", "202209",
          "202210", "202211", "202212",
        ];

        (
          Arc::new(String::from(tournament_name)),
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
    all_tournaments_responses: Vec<(Arc<String>, Vec<String>)>,
  ) -> Vec<(Arc<String>, Vec<MatchResult>)> {
    use serde::Deserialize;

    #[must_use]
    #[derive(Deserialize)]
    struct Table {
      matches_list: Vec<serde_json::Value>,
    }

    all_tournaments_responses
      .into_iter()
      .map(
        |(tournament_name, responses)| -> (Arc<String>, Vec<MatchResult>) {
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
                    (Arc::new(String::from(first_team_name)), first_team_score),
                    (
                      Arc::new(String::from(second_team_name)),
                      second_team_score,
                    ),
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
pub(super) fn fetch_tournaments() -> Vec<Tournament> {
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
    &Tournament::new(
      &Arc::new(String::from("Saudi Professional League")),
      vec![
        Team::new(
          &Arc::new(String::from("Al Ittihad")),
          1,
          19,
          11,
          15,
          2,
          2,
          47,
          33,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Shabab")),
          2,
          20,
          10,
          11,
          7,
          2,
          40,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Nassr")),
          3,
          20,
          10,
          12,
          2,
          6,
          38,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Hilal")),
          4,
          17,
          13,
          8,
          7,
          2,
          31,
          39,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Damac")),
          5,
          20,
          10,
          8,
          6,
          6,
          30,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Abha")),
          6,
          20,
          10,
          8,
          4,
          8,
          28,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Fayha")),
          7,
          19,
          11,
          6,
          8,
          5,
          26,
          33,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Raed")),
          8,
          20,
          10,
          7,
          4,
          9,
          25,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Ahli")),
          9,
          20,
          10,
          5,
          9,
          6,
          24,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al-Tai")),
          10,
          20,
          10,
          7,
          1,
          12,
          22,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Taawoun")),
          11,
          20,
          10,
          5,
          6,
          9,
          21,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Ettifaq")),
          12,
          19,
          11,
          4,
          8,
          7,
          20,
          33,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Faisaly")),
          13,
          19,
          11,
          4,
          8,
          7,
          20,
          33,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al-Batin")),
          14,
          20,
          10,
          4,
          7,
          9,
          19,
          30,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Fateh")),
          15,
          19,
          11,
          4,
          6,
          9,
          18,
          33,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Al Hazem")),
          16,
          20,
          10,
          3,
          5,
          12,
          14,
          30,
          None,
        ),
      ]
      .into_iter()
      .map(Arc::new)
      .collect(),
      Some(
        vec![
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
            Arc::new(String::from(first_team)),
            Arc::new(String::from(second_team))
          ),
          remaining_points,
        ))
        .collect()
      ),
    )
  );

  assert_eq!(
    PremierLeague::test_fetch_tournaments().first().unwrap(),
    &Tournament::new(
      &Arc::new(String::from("First Team - Premier League")),
      vec![
        Team::new(
          &Arc::new(String::from("Manchester City")),
          1,
          18,
          20,
          14,
          2,
          2,
          44,
          60,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Liverpool")),
          2,
          18,
          20,
          12,
          5,
          1,
          41,
          60,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Chelsea")),
          3,
          18,
          20,
          11,
          5,
          2,
          38,
          60,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Arsenal")),
          4,
          18,
          20,
          10,
          2,
          6,
          32,
          60,
          None,
        ),
        Team::new(
          &Arc::new(String::from("West Ham United")),
          5,
          17,
          21,
          8,
          4,
          5,
          28,
          63,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Manchester United")),
          6,
          16,
          22,
          8,
          3,
          5,
          27,
          66,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Tottenham Hotspur")),
          7,
          15,
          23,
          8,
          2,
          5,
          26,
          69,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Wolverhampton Wanderers")),
          8,
          18,
          20,
          7,
          4,
          7,
          25,
          60,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Aston Villa")),
          9,
          17,
          21,
          7,
          1,
          9,
          22,
          63,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Leicester City")),
          10,
          16,
          22,
          6,
          4,
          6,
          22,
          66,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Brentford")),
          11,
          16,
          22,
          5,
          5,
          6,
          20,
          66,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Brighton and Hove Albion")),
          12,
          16,
          22,
          4,
          8,
          4,
          20,
          66,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Crystal Palace")),
          13,
          17,
          21,
          4,
          8,
          5,
          20,
          63,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Everton")),
          14,
          17,
          21,
          5,
          4,
          8,
          19,
          63,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Southampton")),
          15,
          17,
          21,
          3,
          8,
          6,
          17,
          63,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Leeds United")),
          16,
          18,
          20,
          3,
          7,
          8,
          16,
          60,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Watford")),
          17,
          16,
          22,
          4,
          1,
          11,
          13,
          66,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Burnley")),
          18,
          15,
          23,
          1,
          8,
          6,
          11,
          69,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Newcastle United")),
          19,
          18,
          20,
          1,
          7,
          10,
          10,
          60,
          None,
        ),
        Team::new(
          &Arc::new(String::from("Norwich City")),
          20,
          17,
          21,
          2,
          4,
          11,
          10,
          63,
          None,
        ),
      ]
      .into_iter()
      .map(Arc::new)
      .collect(),
      Some(
        vec![
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
            Arc::new(String::from(first_team)),
            Arc::new(String::from(second_team))
          ),
          remaining_points,
        ))
        .collect()
      ),
    )
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
