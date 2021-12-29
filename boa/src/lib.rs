mod mincut_maxflow;
pub mod tournament_prediction;

use std::boxed::Box;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::rc::Rc;

use tournament_prediction::Team;
use tournament_prediction::Tournament;

pub fn test() {
  mincut_maxflow::test();
  tournament_prediction::test();
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
  eprintln!("hello world!");

  let tournament = Tournament {
    teams: vec![
      (
        "atlanta",
        Team {
          matches_won: 83,
          matches_lost: 71,
        },
      ),
      (
        "philadelphia",
        Team {
          matches_won: 80,
          matches_lost: 79,
        },
      ),
      (
        "new-york",
        Team {
          matches_won: 78,
          matches_lost: 78,
        },
      ),
      (
        "montreal",
        Team {
          matches_won: 77,
          matches_lost: 82,
        },
      ),
    ]
    .into_iter()
    .map(|(team_id, team)| (Rc::new(team_id.to_string()), Rc::new(team)))
    .collect(),
    matches_left: vec![
      (("atlanta", "philadelphia"), 1),
      (("atlanta", "new-york"), 6),
      (("atlanta", "montreal"), 1),
      (("philadelphia", "montreal"), 2),
    ]
    .into_iter()
    .map(|((team_id1, team_id2), matches_left)| {
      (
        (Rc::new(team_id1.to_string()), Rc::new(team_id2.to_string())),
        matches_left,
      )
    })
    .collect(),
  };

  let local_eliminated_teams =
    tournament_prediction::predict_tournament_eliminated_teams(&tournament);

  let ets = Box::into_raw(
    (&local_eliminated_teams)
      .iter()
      .map(|(id, eliminating_teams_ids)| EliminatedTeamNative {
        id: CString::new(String::clone(id)).unwrap().into_raw(),
        eliminating_teams_ids_count: eliminating_teams_ids.len() as u64,
        eliminating_teams_ids: Box::into_raw(
          eliminating_teams_ids
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
