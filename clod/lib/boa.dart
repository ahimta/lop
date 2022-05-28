import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

// SEE: https://dart.dev/guides/libraries/c-interop
// SEE: https://docs.flutter.dev/development/platform-integration/c-interop
// SEE: https://github.com/dart-lang/samples/tree/master/ffi
// SEE: https://api.flutter.dev/flutter/dart-ffi/dart-ffi-library.html
final DynamicLibrary _boa = Platform.isIOS
    ? DynamicLibrary.process()
    : DynamicLibrary.open('libboa.so');

class _TournamentNative extends Struct {
  external Pointer<Utf8> name;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int teams_count;
  // ignore: non_constant_identifier_names
  external Pointer<_TeamNative> teams;
}

class Tournament {
  Tournament(this.name, this.teams);

  String name;
  List<Team> teams;
}

class _TeamNative extends Struct {
  external Pointer<Utf8> name;

  @Uint64()
  // ignore: non_constant_identifier_names
  external int rank;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int matches_played;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int matches_left;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int matches_drawn;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int matches_won;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int matches_lost;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int earned_points;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int remaining_points;

  @Uint64()
  // ignore: non_constant_identifier_names
  external int elimination_status;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int eliminating_teams_count;
  // ignore: non_constant_identifier_names
  external Pointer<_TeamNative> eliminating_teams;
}

class Team {
  Team(
    this.name,
    this.rank,
    this.matchesPlayed,
    this.matchesLeft,
    this.matchesDrawn,
    this.matchesWon,
    this.matchesLost,
    this.earnedPoints,
    this.remainingPoints,
    this.eliminationStatus,
    this.eliminatingTeams,
  );

  String name;

  int rank;
  int matchesPlayed;
  int matchesLeft;
  int matchesDrawn;
  int matchesWon;
  int matchesLost;
  int earnedPoints;
  int remainingPoints;

  int eliminationStatus;
  List<Team> eliminatingTeams;
}

// ignore: avoid_private_typedef_functions
typedef _BoaGetTournamentsNative = Int32 Function(
  Pointer<Uint64>,
  Pointer<Pointer<_TournamentNative>>,
);
// ignore: avoid_private_typedef_functions
typedef _BoaGetTournaments = int Function(
  Pointer<Uint64>,
  Pointer<Pointer<_TournamentNative>>,
);

final _BoaGetTournaments _boaGetTournaments = _boa
    .lookup<NativeFunction<_BoaGetTournamentsNative>>(
      'boa_get_tournaments',
    )
    .asFunction();

// ignore: avoid_private_typedef_functions
typedef _BoaFreeTournamentsNative = Void Function(
  Pointer<Pointer<_TournamentNative>>,
);
// ignore: avoid_private_typedef_functions
typedef _BoaFreeTournaments = void Function(
  Pointer<Pointer<_TournamentNative>>,
);

final _BoaFreeTournaments _boaFreeTournaments = _boa
    .lookup<NativeFunction<_BoaFreeTournamentsNative>>(
      'boa_free_tournaments',
    )
    .asFunction();

List<Tournament> getTournaments() {
  final tournamentsCountNative = calloc.allocate<Uint64>(sizeOf<Uint64>());
  final tournamentsNative = calloc.allocate<Pointer<_TournamentNative>>(
    sizeOf<Pointer<_TournamentNative>>(),
  );

  final statusCode = _boaGetTournaments(
    tournamentsCountNative,
    tournamentsNative,
  );
  // FIXME: Better end-to-end error-handling (i.e: from boa to UI).
  final count = statusCode == 0 ? tournamentsCountNative.value : 0;

  final tournaments = <Tournament>[];
  for (var i = 0; i < count; i++) {
    final tournamentNative = tournamentsNative.value[i];

    final teams = <Team>[];
    for (var j = 0; j < tournamentNative.teams_count; j++) {
      teams.add(_doTeam(tournamentNative.teams[j]));
    }

    tournaments.add(Tournament(tournamentNative.name.toDartString(), teams));
  }

  _boaFreeTournaments(tournamentsNative);

  calloc
    ..free(tournamentsCountNative)
    ..free(tournamentsNative);

  return tournaments;
}

Team _doTeam(final _TeamNative teamNative) {
  final eliminatingTeams = <Team>[];
  for (var i = 0; i < teamNative.eliminating_teams_count; i++) {
    eliminatingTeams.add(_doTeam(teamNative.eliminating_teams[i]));
  }

  return Team(
    teamNative.name.toDartString(),
    teamNative.rank,
    teamNative.matches_played,
    teamNative.matches_left,
    teamNative.matches_drawn,
    teamNative.matches_won,
    teamNative.matches_lost,
    teamNative.earned_points,
    teamNative.remaining_points,
    teamNative.elimination_status,
    eliminatingTeams,
  );
}
