fn main() {
  boa::test();

  for tournament in &boa::tournament_fetching::fetch_tournaments() {
    // SEE: https://doc.rust-lang.org/std/fmt/#fillalignment
    println!("|{:-^116}|", "");
    println!("|{:^116}|", tournament.name);
    println!("|{:-^116}|", "");
    println!("| {rank:4} | {id:25} | {matches_left:12} | {matches_won:11} | {eliminated:50} |",
  rank="Rank",
  id="ID",
  matches_left="Matches Left",
  matches_won="Matches Won",
  eliminated="Eliminated",
    );
    println!("|{:-^116}|", "");

    for team in boa::tournament_prediction::predict_tournament_eliminated_teams(
      tournament,
    ) {
      println!("| {rank:4} | {id:25} | {matches_left:12} | {matches_won:11} | {elimination_status:50?} |",
rank=team.rank,
id=team.id,
matches_left=team.matches_left,
matches_won=team.matches_won,
elimination_status=team.elimination_status,
    );
    }
  }
}
