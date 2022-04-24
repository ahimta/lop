use boa::tournament_prediction::EliminationStatus;

fn main() {
  boa::test();

  for tournament in &boa::tournament_fetching::fetch_tournaments() {
    // SEE: https://doc.rust-lang.org/std/fmt/#fillalignment
    println!("|{:-^116}|", "");
    println!("|{:^116}|", tournament.name);
    println!("|{:-^116}|", "");
    println!("| {rank:4} | {id:25} | {eliminated:11} | {matches_left:12} | {matches_won:11} | {eliminating_teams:36} |",
  rank="Rank",
  id="ID",
  eliminated="Eliminated",
  matches_left="Matches Left",
  matches_won="Matches Won",
  eliminating_teams="Eliminating Teams",
    );
    println!("|{:-^116}|", "");

    for team in boa::tournament_prediction::predict_tournament_eliminated_teams(
      tournament,
    ) {
      let eliminating_teams =
        match EliminationStatus::clone(&team.elimination_status) {
          EliminationStatus::Not => vec![].into_iter().collect(),
          EliminationStatus::Trivially(eliminating_teams)
          | EliminationStatus::NonTrivially(eliminating_teams) => {
            eliminating_teams
          }
        };
      println!("| {rank:4} | {id:25} | {elimination_status:11?} | {matches_left:12} | {matches_won:11} | {eliminating_teams:36} |",
rank=team.rank,
id=team.id,
elimination_status=team.elimination_status,
matches_left=team.matches_left,
matches_won=team.matches_won,
eliminating_teams=eliminating_teams.iter().map(|team_id|String::clone(team_id)).collect::<Vec<_>>().join(", "),
    );
    }
  }
}
