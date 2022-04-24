mod mincut_maxflow;
pub mod tournament_fetching;
pub mod tournament_prediction;

use std::boxed::Box;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub fn test() {
  mincut_maxflow::test();
  tournament_prediction::test();
  tournament_fetching::test();
}

#[repr(C)]
#[must_use]
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
#[must_use]
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
#[allow(clippy::not_unsafe_ptr_arg_deref, unused_must_use)]
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
