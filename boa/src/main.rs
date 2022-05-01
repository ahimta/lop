fn main() {
  boa::test();

  for tournament in &boa::tournament_fetching::fetch_tournaments() {
    // SEE: https://doc.rust-lang.org/std/fmt/#fillalignment
    println!("|{:-^116}|", "");
    println!("|{:^116}|", tournament.name);
    println!("|{:-^116}|", "");
    println!("| {rank:4} | {id:25} | {earned_points:13} | {matches_left:12} | {matches_won:11} | {matches_drawn:13} | {eliminated:18} |",
  rank="Rank",
  id="ID",
  matches_left="Matches Left",
  matches_won="Matches Won",
  matches_drawn="Matches Drawn",
  earned_points="Earned Points",
  eliminated="Eliminated",
    );
    println!("|{:-^116}|", "");

    for team in boa::tournament_prediction::predict_tournament_eliminated_teams(
      tournament,
    ) {
      println!("| {rank:4} | {id:25} | {earned_points:13} | {matches_left:12} | {matches_won:11} | {matches_drawn:13} | {elimination_status:18?} |",
rank=team.rank,
id=team.id,
matches_left=team.matches_left,
matches_won=team.matches_won,
matches_drawn=team.matches_drawn,
earned_points=team.earned_points,
elimination_status=team.elimination_status,
    );
    }
  }
}
