import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

// SEE: https://dart.dev/guides/libraries/c-interop
// SEE: https://docs.flutter.dev/development/platform-integration/c-interop
// SEE: https://github.com/dart-lang/samples/tree/master/ffi
// SEE: https://api.flutter.dev/flutter/dart-ffi/dart-ffi-library.html
final DynamicLibrary _lop = Platform.isAndroid
    ? DynamicLibrary.open('liblop.so')
    : DynamicLibrary.process();

class _EliminatedTeamNative extends Struct {
  external Pointer<Utf8> id;
  @Uint64()
  // ignore: non_constant_identifier_names
  external int eliminating_teams_ids_count;
  // ignore: non_constant_identifier_names
  external Pointer<Pointer<Utf8>> eliminating_teams_ids;
}

/// A team that's predicted to be eliminated from the tournament.
/// The prediction may be only guaranteed under certain assumptions or may not
/// entirely apply due to the tournament's deviation from the general
/// algorithmic lines used.
/// In general, a team is only eliminated if it has no chance of winning the
/// tournament.
class EliminatedTeam {
  /// [id]
  /// [eliminatingTeamsIds]
  EliminatedTeam(this.id, this.eliminatingTeamsIds);

  /// Only used to distinguish teams from each other.
  /// It can be assumed to be descriptive but may not be entirely appropriate
  /// for display.
  String id;

  /// These teams are a contributing factor to this team being eliminated.
  /// For example, they may already have significat advantage to eliminate this
  /// or may be able to catch up.
  List<String> eliminatingTeamsIds;
}

// ignore: avoid_private_typedef_functions
typedef _PredictSomeTournamentNative = Int32 Function(
  Pointer<Uint64>,
  Pointer<Pointer<_EliminatedTeamNative>>,
);
// ignore: avoid_private_typedef_functions
typedef _PredictSomeTournament = int Function(
  Pointer<Uint64>,
  Pointer<Pointer<_EliminatedTeamNative>>,
);

final _PredictSomeTournament _predictSomeTournamentNative = _lop
    .lookup<NativeFunction<_PredictSomeTournamentNative>>(
      'predict_some_tournaments',
    )
    .asFunction();

// ignore: avoid_private_typedef_functions
typedef _PredictSomeTournamentFreeNative = Void Function(
  Pointer<Pointer<_EliminatedTeamNative>>,
);
// ignore: avoid_private_typedef_functions
typedef _PredictSomeTournamentFree = void Function(
  Pointer<Pointer<_EliminatedTeamNative>>,
);

final _PredictSomeTournamentFree _predictSomeTournamentFreeNative = _lop
    .lookup<NativeFunction<_PredictSomeTournamentFreeNative>>(
      'predict_some_tournaments_free',
    )
    .asFunction();

/// Predict according to lop's algorithmic model.
List<EliminatedTeam> predictSomeTournament() {
  // FIXME: Answer Stackoverflow question.
  // SEE: https://stackoverflow.com/questions/67313913/dart-flutter-ffiforeign-function-interface-calling-a-native-function-with-out.
  // FIXME: More descriptive names.
  final x = calloc.allocate<Uint64>(sizeOf<Uint64>());
  final y = calloc.allocate<Pointer<_EliminatedTeamNative>>(
    sizeOf<Pointer<_EliminatedTeamNative>>(),
  );

  final statusCode = _predictSomeTournamentNative(x, y);
  // FIXME: Better end-to-end error-handling (i.e: from lop to UI).
  final count = statusCode == 0 ? x.value : 0;

  final ts = <EliminatedTeam>[];
  for (var i = 0; i < count; i++) {
    final id = y.value[i].id.toDartString();
    final eids = <String>[];

    for (var j = 0; j < y.value[i].eliminating_teams_ids_count; j++) {
      eids.add(y.value[i].eliminating_teams_ids[j].toDartString());
    }

    ts.add(EliminatedTeam(id, eids));
  }

  _predictSomeTournamentFreeNative(y);

  calloc
    ..free(x)
    ..free(y);

  return ts;
}
