use std::collections::HashMap;
use std::collections::HashSet;
use std::string;
use std::sync::Arc;

use itertools::Itertools;

use super::tournament_prediction::Team;
use super::tournament_prediction::TeamId;
use super::tournament_prediction::Tournament;

enum IntegrationMode {
  #[allow(dead_code)]
  GenerateForTest,
  UseForTest,
  DownloadReal,
}

/// # Panics
#[must_use]
pub(super) fn fetch_tournament() -> Tournament {
  process_tournament(download_tournament(&IntegrationMode::DownloadReal))
}

#[allow(clippy::too_many_lines)]
fn process_tournament(
  tournament_results_pages_json_non_parsed: Vec<String>,
) -> Tournament {
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

  type MatchResult = ((Arc<TeamId>, f64), (Arc<TeamId>, f64));

  const MATCHES_PER_TEAM_PAIR: usize = 2;

  let all_results: Vec<MatchResult> = tournament_results_pages_json_non_parsed
    .into_iter()
    .flat_map(|tournament_results_page_json_non_parsed| {
      let tournament_results_single_page_json: Page =
        serde_json::from_str(&tournament_results_page_json_non_parsed).unwrap();
      tournament_results_single_page_json
        .content
        .into_iter()
        .map(|ContentItem { teams, .. }| {
          let (first_team, second_team) = &teams;

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

  let matches_won: HashMap<&Arc<TeamId>, usize> = all_results
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

  let teams: HashMap<Arc<TeamId>, Arc<Team>> = all_results
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
          matches_won: matches_won[team_id],
        }),
      )
    })
    .collect();

  let matches_played: HashMap<(&Arc<TeamId>, &Arc<TeamId>), usize> =
    all_results
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
        MATCHES_PER_TEAM_PAIR
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
    teams,
    matches_left,
  }
}

/// # Panics
#[must_use]
fn download_tournament(integration_mode: &IntegrationMode) -> Vec<String> {
  use reqwest::blocking::Client;
  use reqwest::header::HeaderMap;
  use reqwest::header::HeaderValue;

  // NOTE: Used to match exactly the value used in official page.
  const PAGE_SIZE: usize = 40;
  // NOTE: Used to prevent an infinite loop in case the API response changes.
  const ITEMS_MAX: usize = 2 * 1000;

  match integration_mode {
    IntegrationMode::UseForTest => {
      use std::fs;

      const TEST_DATA_FILE_ID: &str = "2021-12-26T14:58:52";
      return fs::read_to_string(format!(
        "data/premier-league-{}",
        TEST_DATA_FILE_ID,
      ))
      .expect("reading test data failed")
      .lines()
      .map(string::ToString::to_string)
      .collect();
    }
    IntegrationMode::GenerateForTest | IntegrationMode::DownloadReal => (),
  }

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

    // SEE: https://docs.rs/reqwest/0.11.7/reqwest/struct.RequestBuilder.html#method.send
    let resp = client
    // NOTE: Used to match exactly the URL used in the official page.
    // SEE: https://www.premierleague.com/results
    .get(format!("https://footballapi.pulselive.com/football/fixtures?comps=1&compSeasons=418&teams=1,2,130,131,43,4,6,7,9,26,10,11,12,23,14,20,21,33,25,38&page={:?}&pageSize={:?}&sort=desc&statuses=C&altIds=true", page, PAGE_SIZE))
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

  match integration_mode {
    IntegrationMode::UseForTest => panic!("unexpected/impossible case"),
    IntegrationMode::GenerateForTest => {
      use std::fs::File;
      use std::io::Write;

      use chrono::prelude::Utc;

      let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

      let mut f = File::create(format!("data/premier-league-{}", timestamp))
        .expect("Unable to create file");
      for page in &tournament_results_pages_json_non_parsed {
        f.write_all(page.as_bytes()).expect("write failed");
        f.write_all(b"\n").expect("newline write failed");
      }
    }
    IntegrationMode::DownloadReal => (),
  }

  tournament_results_pages_json_non_parsed
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  assert_eq!(
    process_tournament(download_tournament(&IntegrationMode::UseForTest)),
    Tournament {
      teams: vec![
        ("Watford", Team { matches_won: 4 }),
        ("Brentford", Team { matches_won: 5 }),
        ("Chelsea", Team { matches_won: 11 }),
        ("Norwich City", Team { matches_won: 2 }),
        ("Tottenham Hotspur", Team { matches_won: 8 }),
        ("Liverpool", Team { matches_won: 12 }),
        ("Everton", Team { matches_won: 5 }),
        ("Leicester City", Team { matches_won: 6 }),
        ("Aston Villa", Team { matches_won: 7 }),
        ("Brighton and Hove Albion", Team { matches_won: 4 }),
        ("West Ham United", Team { matches_won: 8 }),
        ("Manchester United", Team { matches_won: 8 }),
        ("Burnley", Team { matches_won: 1 }),
        ("Newcastle United", Team { matches_won: 1 }),
        ("Wolverhampton Wanderers", Team { matches_won: 7 }),
        ("Leeds United", Team { matches_won: 3 }),
        ("Manchester City", Team { matches_won: 14 }),
        ("Southampton", Team { matches_won: 3 }),
        ("Arsenal", Team { matches_won: 10 }),
        ("Crystal Palace", Team { matches_won: 4 })
      ]
      .into_iter()
      .map(|(team_id, team)| (Arc::new(team_id.to_string()), Arc::new(team)))
      .collect(),
      matches_left: vec![
        (("Arsenal", "Leeds United"), 1),
        (("Brentford", "Burnley"), 1),
        (("Arsenal", "Manchester United"), 1),
        (("Liverpool", "Newcastle United"), 1),
        (("Manchester City", "Tottenham Hotspur"), 1),
        (("Manchester United", "Newcastle United"), 1),
        (("Newcastle United", "Watford"), 1),
        (("Brentford", "Everton"), 1),
        (("Leicester City", "Southampton"), 1),
        (("Arsenal", "Crystal Palace"), 1),
        (("Leeds United", "Wolverhampton Wanderers"), 1),
        (("Chelsea", "Tottenham Hotspur"), 1),
        (("Newcastle United", "Norwich City"), 1),
        (("Liverpool", "Wolverhampton Wanderers"), 1),
        (("Brighton and Hove Albion", "Leicester City"), 1),
        (("Tottenham Hotspur", "West Ham United"), 1),
        (("Crystal Palace", "Liverpool"), 1),
        (("Newcastle United", "Tottenham Hotspur"), 1),
        (("Liverpool", "Manchester City"), 1),
        (("Liverpool", "Watford"), 1),
        (("Aston Villa", "Leicester City"), 1),
        (("Brighton and Hove Albion", "Burnley"), 1),
        (("Newcastle United", "Wolverhampton Wanderers"), 1),
        (("Chelsea", "Liverpool"), 1),
        (("Leicester City", "Manchester United"), 1),
        (("Tottenham Hotspur", "Watford"), 1),
        (("Manchester United", "Tottenham Hotspur"), 1),
        (("Everton", "Norwich City"), 1),
        (("Chelsea", "Wolverhampton Wanderers"), 1),
        (("Chelsea", "West Ham United"), 1),
        (("Brentford", "Leicester City"), 1),
        (("Brentford", "Crystal Palace"), 1),
        (("Aston Villa", "Brentford"), 1),
        (("Brentford", "Leeds United"), 1),
        (("Arsenal", "Southampton"), 1),
        (("Aston Villa", "Watford"), 1),
        (("Burnley", "Liverpool"), 1),
        (("Brighton and Hove Albion", "Liverpool"), 1),
        (("Brentford", "Tottenham Hotspur"), 1),
        (("Aston Villa", "Wolverhampton Wanderers"), 1),
        (("Brentford", "Watford"), 1),
        (("Norwich City", "Watford"), 1),
        (("Crystal Palace", "Manchester City"), 1),
        (("Brighton and Hove Albion", "West Ham United"), 1),
        (("Everton", "Leeds United"), 1),
        (("Leeds United", "Watford"), 1),
        (("Everton", "Manchester City"), 1),
        (("Aston Villa", "Newcastle United"), 1),
        (("Chelsea", "Manchester United"), 1),
        (("Burnley", "Manchester City"), 1),
        (("Leeds United", "West Ham United"), 1),
        (("Crystal Palace", "Newcastle United"), 1),
        (("Manchester United", "Southampton"), 1),
        (("Crystal Palace", "Everton"), 1),
        (("Brighton and Hove Albion", "Norwich City"), 1),
        (("Manchester United", "Wolverhampton Wanderers"), 1),
        (("Aston Villa", "Chelsea"), 1),
        (("Manchester City", "Southampton"), 1),
        (("Chelsea", "Leeds United"), 1),
        (("Leicester City", "Wolverhampton Wanderers"), 1),
        (("Brentford", "Chelsea"), 1),
        (("Aston Villa", "Manchester United"), 1),
        (("Southampton", "Watford"), 1),
        (("Burnley", "Leeds United"), 1),
        (("Leeds United", "Southampton"), 1),
        (("Crystal Palace", "West Ham United"), 1),
        (("Aston Villa", "Crystal Palace"), 1),
        (("Aston Villa", "West Ham United"), 1),
        (("Brighton and Hove Albion", "Everton"), 1),
        (("Arsenal", "Norwich City"), 1),
        (("Brentford", "Liverpool"), 1),
        (("Everton", "Liverpool"), 1),
        (("Arsenal", "Brentford"), 1),
        (("Aston Villa", "Liverpool"), 1),
        (("Aston Villa", "Brighton and Hove Albion"), 1),
        (("Chelsea", "Norwich City"), 1),
        (("Brighton and Hove Albion", "Southampton"), 1),
        (("Aston Villa", "Norwich City"), 1),
        (("Tottenham Hotspur", "Wolverhampton Wanderers"), 1),
        (("Burnley", "Wolverhampton Wanderers"), 1),
        (("Brentford", "Wolverhampton Wanderers"), 1),
        (("Aston Villa", "Everton"), 1),
        (("Everton", "Manchester United"), 1),
        (("Brentford", "Newcastle United"), 1),
        (("Liverpool", "Manchester United"), 1),
        (("Brighton and Hove Albion", "Newcastle United"), 1),
        (("Burnley", "Southampton"), 1),
        (("Leeds United", "Tottenham Hotspur"), 1),
        (("Everton", "Southampton"), 1),
        (("Aston Villa", "Manchester City"), 1),
        (("Chelsea", "Everton"), 1),
        (("Arsenal", "Aston Villa"), 1),
        (("Burnley", "Newcastle United"), 1),
        (("Arsenal", "Liverpool"), 1),
        (("Burnley", "West Ham United"), 1),
        (("Leeds United", "Norwich City"), 1),
        (("Leicester City", "West Ham United"), 1),
        (("Aston Villa", "Tottenham Hotspur"), 1),
        (("Chelsea", "Crystal Palace"), 1),
        (("Chelsea", "Leicester City"), 1),
        (("Burnley", "Norwich City"), 1),
        (("Leeds United", "Liverpool"), 1),
        (("Chelsea", "Newcastle United"), 1),
        (("Leeds United", "Leicester City"), 1),
        (("Arsenal", "Burnley"), 1),
        (("Manchester United", "Norwich City"), 1),
        (("Arsenal", "West Ham United"), 1),
        (("Crystal Palace", "Tottenham Hotspur"), 1),
        (("Burnley", "Chelsea"), 1),
        (("Norwich City", "Wolverhampton Wanderers"), 1),
        (("Brighton and Hove Albion", "Leeds United"), 1),
        (("Arsenal", "Tottenham Hotspur"), 1),
        (("Brentford", "West Ham United"), 1),
        (("Manchester City", "Manchester United"), 1),
        (("West Ham United", "Wolverhampton Wanderers"), 1),
        (("Manchester City", "Norwich City"), 1),
        (("Everton", "Watford"), 1),
        (("Norwich City", "Tottenham Hotspur"), 1),
        (("Everton", "Wolverhampton Wanderers"), 1),
        (("Arsenal", "Newcastle United"), 1),
        (("Arsenal", "Watford"), 1),
        (("Leeds United", "Newcastle United"), 1),
        (("Crystal Palace", "Southampton"), 1),
        (("Crystal Palace", "Manchester United"), 1),
        (("Brighton and Hove Albion", "Manchester City"), 1),
        (("Crystal Palace", "Leeds United"), 1),
        (("Liverpool", "Southampton"), 1),
        (("Brighton and Hove Albion", "Wolverhampton Wanderers"), 1),
        (("Manchester City", "West Ham United"), 1),
        (("Brentford", "Norwich City"), 1),
        (("Leeds United", "Manchester United"), 1),
        (("Southampton", "Wolverhampton Wanderers"), 1),
        (("Brighton and Hove Albion", "Crystal Palace"), 1),
        (("Manchester City", "Watford"), 1),
        (("Leicester City", "Newcastle United"), 1),
        (("Newcastle United", "Southampton"), 1),
        (("Manchester City", "Newcastle United"), 1),
        (("Norwich City", "Southampton"), 1),
        (("Burnley", "Everton"), 1),
        (("Crystal Palace", "Wolverhampton Wanderers"), 1),
        (("Chelsea", "Southampton"), 1),
        (("Arsenal", "Brighton and Hove Albion"), 1),
        (("Chelsea", "Manchester City"), 1),
        (("Brentford", "Brighton and Hove Albion"), 1),
        (("Watford", "Wolverhampton Wanderers"), 1),
        (("Leeds United", "Manchester City"), 1),
        (("Arsenal", "Leicester City"), 1),
        (("Arsenal", "Everton"), 1),
        (("Burnley", "Crystal Palace"), 1),
        (("Southampton", "West Ham United"), 1),
        (("Liverpool", "Tottenham Hotspur"), 1),
        (("Newcastle United", "West Ham United"), 1),
        (("Arsenal", "Manchester City"), 1),
        (("Arsenal", "Chelsea"), 1),
        (("Manchester United", "Watford"), 1),
        (("Manchester United", "West Ham United"), 1),
        (("Crystal Palace", "Leicester City"), 1),
        (("Leicester City", "Watford"), 1),
        (("Leicester City", "Norwich City"), 1),
        (("Everton", "Tottenham Hotspur"), 1),
        (("Manchester City", "Wolverhampton Wanderers"), 1),
        (("Liverpool", "West Ham United"), 1),
        (("Chelsea", "Watford"), 1),
        (("Everton", "West Ham United"), 1),
        (("Aston Villa", "Southampton"), 1),
        (("Brighton and Hove Albion", "Watford"), 1),
        (("Leicester City", "Manchester City"), 1),
        (("Liverpool", "Norwich City"), 1),
        (("Burnley", "Leicester City"), 1),
        (("Brighton and Hove Albion", "Chelsea"), 2),
        (("Aston Villa", "Burnley"), 2),
        (("Crystal Palace", "Norwich City"), 2),
        (("Brentford", "Manchester City"), 2),
        (("Burnley", "Manchester United"), 2),
        (("Burnley", "Watford"), 2),
        (("Watford", "West Ham United"), 2),
        (("Brentford", "Manchester United"), 2),
        (("Leicester City", "Liverpool"), 2),
        (("Burnley", "Tottenham Hotspur"), 2),
        (("Leicester City", "Tottenham Hotspur"), 2),
        (("Crystal Palace", "Watford"), 2),
        (("Aston Villa", "Leeds United"), 2),
        (("Brentford", "Southampton"), 2),
        (("Brighton and Hove Albion", "Tottenham Hotspur"), 2),
        (("Everton", "Newcastle United"), 2),
        (("Brighton and Hove Albion", "Manchester United"), 2),
        (("Arsenal", "Wolverhampton Wanderers"), 2),
        (("Everton", "Leicester City"), 2),
        (("Norwich City", "West Ham United"), 2),
        (("Southampton", "Tottenham Hotspur"), 2),
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
