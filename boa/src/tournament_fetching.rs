mod common;

use std::sync::Arc;

use crate::tournament_fetching::common::MatchResult;
use crate::tournament_fetching::common::TournamentProvider;

// FIXME: Always use `crate` imports.
use super::tournament_prediction::Team;
use super::tournament_prediction::Tournament;

struct PremierLeague {}
impl TournamentProvider for PremierLeague {
  const TEST_DATA_FILE_ID: &'static str = "2021-12-26T14:58:52";
  const TEST_DATA_PREFIX: &'static str = "premier-league";

  fn download_tournament() -> Vec<String> {
    use reqwest::blocking::Client;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    // NOTE: Used to match exactly the value used in official page.
    const PAGE_SIZE: usize = 40;
    // NOTE: Used to prevent an infinite loop in case the API response changes.
    const ITEMS_MAX: usize = 2 * 1000;

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
    headers.insert(
      "Origin",
      HeaderValue::from_static("https://www.premierleague.com"),
    );
    // SEE: https://docs.rs/reqwest/0.11.7/reqwest/struct.ClientBuilder.html
    let client = Client::builder()
  .user_agent("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:95.0) Gecko/20100101 Firefox/95.0")
  .referer(true)
  .https_only(true)
  .gzip(true)
  .deflate(true)
  .brotli(true)
  .default_headers(headers)
  .build()
  .unwrap();

    let mut page = 0;
    let mut tournament_results_pages_json_non_parsed: Vec<String> = vec![];
    loop {
      assert!(
        page * PAGE_SIZE <= ITEMS_MAX,
        "too many pages ({:?}, {:?})",
        page,
        PAGE_SIZE,
      );

      // FIXME: Primer typo.
      // NOTE: First Team.
      // NOTE: Primier League.
      let tournament_url=format!("https://footballapi.pulselive.com/football/fixtures?comps=1&compSeasons=418&teams=1,2,130,131,43,4,6,7,9,26,10,11,12,23,14,20,21,33,25,38&page={:?}&pageSize={:?}&sort=desc&statuses=C&altIds=true", page, PAGE_SIZE,);
      // NOTE: UEFA Champions League.
      // let tournament_url=format!("https://footballapi.pulselive.com/football/fixtures?comps=2&compSeasons=424&teams=47,541,48,49,51,229,52,231,4,232,83,56,58,87,1408,241,62,243,10,201,11,12,64,67,635,68,108,71,110,204,252,253,74&page={:?}&pageSize={:?}&sort=desc&statuses=C&altIds=true", page, PAGE_SIZE,);
      // NOTE: UEFA Europa League.
      // let tournament_url=format!("https://footballapi.pulselive.com/football/fixtures?comps=3&compSeasons=457&teams=541,49,50,52,505,53,494,81,84,364,58,366,989,61,93,95,96,26,369,97,98,63,202,65,66,105,470,635,106,69,1420,108,110,752,111,249,506,373,25,74&page={:?}&pageSize={:?}&sort=desc&statuses=C&altIds=true", page, PAGE_SIZE,);

      // NOTE: PL2.
      // NOTE: Primier League 2 - Division 1.
      // let tournament_url=format!("https://footballapi.pulselive.com/football/fixtures?comps=16&compSeasons=438&teams=385,332,334,336,275,337,339,279,343,344,381,387,383,358&page={:?}&pageSize={:?}&sort=desc&statuses=C&altIds=true", page, PAGE_SIZE,);
      // NOTE: Primier League 2 - Division 2.
      // let tournament_url=format!("https://footballapi.pulselive.com/football/fixtures?comps=17&compSeasons=447&teams=386,266,335,341,345,346,347,281,351,352,354,355,357,360&page={:?}&pageSize={:?}&sort=desc&statuses=C&altIds=true", page, PAGE_SIZE,);

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

    tournament_results_pages_json_non_parsed
  }

  fn process_tournament(
    tournament_results_pages_json_non_parsed: Vec<String>,
  ) -> Vec<MatchResult> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Page {
      content: Vec<ContentItem>,
    }

    #[derive(Deserialize)]
    struct ContentItem {
      teams: (ContentItemTeam, ContentItemTeam),
    }

    #[derive(Deserialize)]
    struct ContentItemTeam {
      score: f64,
      team: ContentItemTeamTeam,
    }

    #[derive(Deserialize)]
    struct ContentItemTeamTeam {
      name: String,
    }

    let matches_results: Vec<MatchResult> =
      tournament_results_pages_json_non_parsed
        .into_iter()
        .flat_map(|tournament_results_page_json_non_parsed| {
          let tournament_results_single_page_json: Page =
            serde_json::from_str(&tournament_results_page_json_non_parsed)
              .unwrap();
          tournament_results_single_page_json
            .content
            .into_iter()
            .map(|ContentItem { teams, .. }| {
              let (first_team, second_team) = &teams;

              // FIXME: Make sure scores aren't fractional and convert them to
              // `usize` to avoid landmines (some already happened but not
              // committed).

              (
                // FIXME: Make sure to handle cases where a team wins in
                // penalties. Luckily, the primary tournament we use right now
                // doesn't seem to have this case.
                (Arc::new(first_team.team.name.clone()), first_team.score),
                (Arc::new(second_team.team.name.clone()), second_team.score),
              )
            })
            .collect::<Vec<_>>()
        })
        .collect();

    matches_results
  }
}

struct Koora {}
impl TournamentProvider for Koora {
  const TEST_DATA_FILE_ID: &'static str = "2022-02-14T22:50:10";
  const TEST_DATA_PREFIX: &'static str = "koora";

  fn download_tournament() -> Vec<String> {
    use reqwest::blocking::Client;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

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
    headers
      .insert("Origin", HeaderValue::from_static("https://www.kooora.com"));
    // FIXME: Add helper for getting client (or even better, doing a request).
    // SEE: https://docs.rs/reqwest/0.11.7/reqwest/struct.ClientBuilder.html
    let client = Client::builder()
  .user_agent("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:95.0) Gecko/20100101 Firefox/95.0")
  .referer(true)
  .https_only(true)
  .gzip(true)
  .deflate(true)
  .brotli(true)
  .default_headers(headers)
  .build()
  .unwrap();

    // NOTE: Saudi Professional League.
    let competition = 22551;
    // NOTE: Saudi U-13 Premier League.
    // NOTE: Al-Ahli (rank 6) and Al-Nassr (rank 7) eliminated.
    // let competition = 22279;
    // NOTE: Saudi U-15 Premier League.
    // NOTE: Al-Baten (rank 9) and Hajer (rank 10) eliminated.
    // let competition = 22307;
    // NOTE: Saudi Arabia U-19 League Division 1.
    // let competition = 22277;
    // NOTE: Saudi Arabia U-17 League Division 1.
    // let competition = 22278;
    // NOTE: Saudi Arabia Youth League - U19.
    // let competition = 22274;
    // NOTE: Saudi U-17 Premier League.
    // let competition = 22276;

    let months = vec![
      "202101", "202102", "202103", "202104", "202105", "202106", "202107",
      "202108", "202109", "202110", "202111", "202112", "202201", "202202",
      "202203", "202204", "202205", "202206", "202207", "202208", "202209",
      "202210", "202211", "202212",
    ];
    months
      .into_iter()
      .map(|current_month| -> String {
        let tournament_url = format!(
          "https://www.goalzz.com/main.aspx?c={}&stage=1&smonth={}&ajax=true",
          competition, current_month,
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

        resp.replace("\n", "")
      })
      .collect()
  }

  fn process_tournament(responses: Vec<String>) -> Vec<MatchResult> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Table {
      matches_list: Vec<serde_json::Value>,
    }

    let matches_results: Vec<MatchResult> = responses
      .into_iter()
      .flat_map(|current_response| {
        let table: Table = serde_json::from_str(&current_response).unwrap();
        let table_cells = table.matches_list;

        let results: Vec<MatchResult> = (0..table_cells.len())
          .filter_map(|i| -> Option<MatchResult> {
            let cell = &table_cells[i];

            match cell {
              serde_json::Value::String(_) => (),
              _ => return None,
            }

            let cell_value = cell.as_str().unwrap();

            let possible_scores: Vec<&str> = cell_value.split('|').collect();
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

            let first_team_score: f64 = possible_scores[0].parse().unwrap();
            let second_team_score: f64 = possible_scores[1].parse().unwrap();

            let first_team_name = table_cells[i - 2].as_str().unwrap();
            let second_team_name = table_cells[i + 3].as_str().unwrap();

            // FIXME: Make sure scores aren't fractional and convert them to
            // `usize` to avoid landmines (some already happened but not
            // committed).

            Some((
              // FIXME: Make sure to handle cases where a team wins in
              // penalties. Luckily, the primary tournament we use right now
              // doesn't seem to have this case.
              (Arc::new(first_team_name.to_string()), first_team_score),
              (Arc::new(second_team_name.to_string()), second_team_score),
            ))
          })
          .collect();

        results
      })
      .collect();

    matches_results
  }
}

/// # Panics
#[must_use]
pub(super) fn fetch_tournament() -> Tournament {
  Koora::fetch_tournament()
  // PremierLeague::fetch_tournament()
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  assert_eq!(
    Koora::test_fetch_tournament(),
    Tournament {
      teams: vec![
        ("Al Hazem", Team { matches_won: 13 }),
        ("Al Taawoun", Team { matches_won: 17 }),
        ("Al Ahli", Team { matches_won: 20 }),
        ("Al Shabab", Team { matches_won: 37 }),
        ("Al-Batin", Team { matches_won: 16 }),
        ("Al Nassr", Team { matches_won: 37 }),
        ("Al Hilal", Team { matches_won: 26 }),
        ("Al Ittihad", Team { matches_won: 46 }),
        ("Al Fayha", Team { matches_won: 22 }),
        ("Damac", Team { matches_won: 28 }),
        ("Abha", Team { matches_won: 26 }),
        ("Al Fateh", Team { matches_won: 14 }),
        ("Al-Tai", Team { matches_won: 21 }),
        ("Al Ettifaq", Team { matches_won: 16 }),
        ("Al Raed", Team { matches_won: 23 }),
        ("Al Faisaly", Team { matches_won: 16 }),
      ]
      .into_iter()
      .map(|(team_id, team)| (Arc::new(team_id.to_string()), Arc::new(team)))
      .collect(),
      matches_left: vec![
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
      .map(|((first_team, second_team), matches_left)| (
        (
          Arc::new(first_team.to_string()),
          Arc::new(second_team.to_string())
        ),
        matches_left,
      ))
      .collect(),
    }
  );

  assert_eq!(
    PremierLeague::test_fetch_tournament(),
    Tournament {
      teams: vec![
        ("Watford", Team { matches_won: 13 }),
        ("Brentford", Team { matches_won: 16 }),
        ("Chelsea", Team { matches_won: 36 }),
        ("Norwich City", Team { matches_won: 8 }),
        ("Tottenham Hotspur", Team { matches_won: 25 }),
        ("Liverpool", Team { matches_won: 39 }),
        ("Everton", Team { matches_won: 16 }),
        ("Leicester City", Team { matches_won: 19 }),
        ("Aston Villa", Team { matches_won: 22 }),
        ("Brighton and Hove Albion", Team { matches_won: 15 }),
        ("West Ham United", Team { matches_won: 26 }),
        ("Manchester United", Team { matches_won: 25 }),
        ("Burnley", Team { matches_won: 7 }),
        ("Newcastle United", Team { matches_won: 7 }),
        ("Wolverhampton Wanderers", Team { matches_won: 23 }),
        ("Leeds United", Team { matches_won: 13 }),
        ("Manchester City", Team { matches_won: 43 }),
        ("Southampton", Team { matches_won: 14 }),
        ("Arsenal", Team { matches_won: 31 }),
        ("Crystal Palace", Team { matches_won: 17 })
      ]
      .into_iter()
      .map(|(team_id, team)| (Arc::new(team_id.to_string()), Arc::new(team)))
      .collect(),
      matches_left: vec![
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
      .map(|((first_team, second_team), matches_left)| (
        (
          Arc::new(first_team.to_string()),
          Arc::new(second_team.to_string())
        ),
        matches_left,
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
