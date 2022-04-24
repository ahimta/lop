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
      let eliminated =
        format!("{}-{}", team.eliminated, team.eliminated_trivially);
      println!("| {rank:4} | {id:25} | {eliminated:11} | {matches_left:12} | {matches_won:11} | {eliminating_teams:36} |",
rank=team.rank,
id=team.id,
eliminated=eliminated,
matches_left=team.matches_left,
matches_won=team.matches_won,
eliminating_teams=team.eliminating_teams.iter().map(|team_id|String::clone(team_id)).collect::<Vec<_>>().join(", "),
    );
    }
  }
}
