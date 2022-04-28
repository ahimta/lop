fn main() {
  boa::test();

  for tournament in &boa::tournament_fetching::fetch_tournaments() {
    // SEE: https://doc.rust-lang.org/std/fmt/#fillalignment
    println!("|{:-^116}|", "");
    println!("|{:^116}|", tournament.name);
    println!("|{:-^116}|", "");
    println!("| {rank:4} | {id:25} | {eliminated:52} | {matches_left:12} | {matches_won:11} |",
  rank="Rank",
  id="ID",
  eliminated="Eliminated",
  matches_left="Matches Left",
  matches_won="Matches Won",
    );
    println!("|{:-^116}|", "");

    for team in boa::tournament_prediction::predict_tournament_eliminated_teams(
      tournament,
    ) {
      println!("| {rank:4} | {id:25} | {elimination_status:52?} | {matches_left:12} | {matches_won:11} |",
rank=team.rank,
id=team.id,
elimination_status=team.elimination_status,
matches_left=team.matches_left,
matches_won=team.matches_won,
    );
    }
  }
}
