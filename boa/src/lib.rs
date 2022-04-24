mod mincut_maxflow;
mod tournament_fetching;
pub mod tournament_prediction;

use std::boxed::Box;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub fn test() {
  mincut_maxflow::test();
  tournament_prediction::test();
  tournament_fetching::test();

  for tournament in &tournament_fetching::fetch_tournaments() {
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

    for team in
      tournament_prediction::predict_tournament_eliminated_teams(tournament)
    {
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

#[repr(C)]
pub struct EliminatedTeamNative {
  id: *const c_char,
  eliminating_teams_ids_count: u64,
  eliminating_teams_ids: *const *const c_char,
}

#[no_mangle]
pub extern "C" fn test_native() {
  test();
}

/// # Panics
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn predict_tournament_eliminated_teams_native(
  eliminated_teams_count: *mut u64,
  eliminated_teams: *mut *const EliminatedTeamNative,
) -> i32 {
  // FIXME: Change API to use new format.

  let tournaments = tournament_fetching::fetch_tournaments();
  let local_eliminated_teams = tournaments
    .iter()
    .map(tournament_prediction::predict_tournament_eliminated_teams)
    .next()
    .unwrap();

  let ets = Box::into_raw(
    (&local_eliminated_teams)
      .iter()
      .map(|team| EliminatedTeamNative {
        id: CString::new(String::clone(&team.id)).unwrap().into_raw(),
        eliminating_teams_ids_count: team.eliminating_teams.len() as u64,
        eliminating_teams_ids: Box::into_raw(
          team
            .eliminating_teams
            .iter()
            .map(|s| CString::new(String::clone(s)).unwrap().into_raw())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        ) as *const *const c_char,
      })
      .collect::<Vec<_>>()
      .into_boxed_slice(),
  ) as *const EliminatedTeamNative;

  unsafe {
    *eliminated_teams_count = local_eliminated_teams.len() as u64;
    // NOTE: We have to use `NULL` when an array is empty as otherwise
    // deallocation would fail with a misaligned pointer on Android x86_64 (and
    // probably any Linux system). This is to be expected as it might be
    // considered an empty allocation (which has some subtleties).
    *eliminated_teams = if *eliminated_teams_count == 0 {
      ptr::null()
    } else {
      ets
    };
  }

  0
}

/// # Panics
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn predict_tournament_eliminated_teams_native_free(
  eliminated_teams: *mut *const EliminatedTeamNative,
) {
  unsafe {
    if (*eliminated_teams).is_null() {
      return;
    }

    Box::from_raw(*eliminated_teams as *mut EliminatedTeamNative);
    *eliminated_teams = ptr::null();
  }
}
