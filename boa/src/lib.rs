mod common;
mod mincut_maxflow;
mod tournament;

use std::boxed::Box;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::common::EliminationStatus;
use crate::common::Team;
use crate::common::Tournament;

pub fn test() {
  mincut_maxflow::test();
  tournament::test();
}

#[no_mangle]
pub extern "C" fn test_native() {
  test();
}

#[must_use]
#[repr(C)]
pub struct TournamentNative {
  name: *const c_char,
  teams_count: u64,
  teams: *const TeamNative,
}

#[must_use]
#[repr(C)]
pub struct TeamNative {
  name: *const c_char,
  rank: u64,
  matches_played: u64,
  matches_left: u64,
  matches_drawn: u64,
  matches_won: u64,
  matches_lost: u64,
  earned_points: u64,
  remaining_points: u64,

  elimination_status: u64,
  eliminating_teams_count: u64,
  eliminating_teams: *const TeamNative,
}

#[must_use]
pub fn get_tournaments() -> Vec<Tournament> {
  tournament::get_tournaments()
}

/// # Panics
#[must_use]
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn boa_get_tournaments(
  tournaments_count: *mut u64,
  tournaments: *mut *const TournamentNative,
) -> i32 {
  let local_tournaments = tournament::get_tournaments();

  unsafe {
    *tournaments_count = local_tournaments.len() as u64;
  }

  let tournaments_native: *const TournamentNative = Box::into_raw(
    local_tournaments
      .into_iter()
      .map(|tournament| TournamentNative {
        name: CString::new(&**tournament.name).unwrap().into_raw(),
        teams_count: tournament.teams.len() as u64,
        teams: Box::into_raw(
          tournament
            .teams
            .into_iter()
            .map(|team| do_team(&team))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        ) as *const TeamNative,
      })
      .collect::<Vec<_>>()
      .into_boxed_slice(),
  )
    as *const TournamentNative;

  unsafe {
    // NOTE: We have to use `NULL` when an array is empty as otherwise
    // deallocation would fail with a misaligned pointer on Android x86_64 (and
    // probably any Linux system). This is to be expected as it might be
    // considered an empty allocation (which has some subtleties).
    *tournaments = if *tournaments_count == 0 {
      ptr::null()
    } else {
      tournaments_native
    };
  }

  0
}

#[must_use]
fn do_team(team: &Team) -> TeamNative {
  let empty_eliminating_teams = vec![].into_iter().collect();
  let eliminating_teams = match &team.elimination_status {
    EliminationStatus::Not => &empty_eliminating_teams,
    EliminationStatus::Trivially(eliminating_teams)
    | EliminationStatus::NonTrivially(eliminating_teams) => eliminating_teams,
  };

  TeamNative {
    name: CString::new(&**team.name).unwrap().into_raw(),
    rank: team.rank as u64,
    matches_played: team.matches_played as u64,
    matches_left: team.matches_left as u64,
    matches_drawn: team.matches_drawn as u64,
    matches_won: team.matches_won as u64,
    matches_lost: team.matches_lost as u64,
    earned_points: team.earned_points as u64,
    remaining_points: team.remaining_points as u64,

    elimination_status: match team.elimination_status {
      EliminationStatus::Not => 1u64,
      EliminationStatus::Trivially(_) => 2u64,
      EliminationStatus::NonTrivially(_) => 3u64,
    },
    eliminating_teams_count: eliminating_teams.len() as u64,
    eliminating_teams: Box::into_raw(
      eliminating_teams
        .iter()
        .map(|eliminating_team| do_team(eliminating_team))
        .collect::<Vec<_>>()
        .into_boxed_slice(),
    ) as *const TeamNative,
  }
}

/// # Panics
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref, unused_must_use)]
pub extern "C" fn boa_free_tournaments(
  tournaments: *mut *const TournamentNative,
) {
  unsafe {
    if (*tournaments).is_null() {
      return;
    }

    Box::from_raw(*tournaments as *mut TournamentNative);
    *tournaments = ptr::null();
  }
}
