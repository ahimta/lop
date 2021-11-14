import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

final DynamicLibrary _lop = Platform.isAndroid
    ? DynamicLibrary.open("liblop.so")
    : DynamicLibrary.process();

class _EliminatedTeamNative extends Struct {
  external Pointer<Utf8> id;
  @Uint64()
  external int eliminating_teams_ids_count;
  external Pointer<Pointer<Utf8>> eliminating_teams_ids;
}

class EliminatedTeam {
  String id;
  List<String> eliminatingTeamsIds;

  EliminatedTeam(this.id, this.eliminatingTeamsIds);
}

typedef _PredictSomeTournamentNative = Int32 Function(
    Pointer<Uint64>, Pointer<Pointer<_EliminatedTeamNative>>);
typedef _PredictSomeTournament = int Function(
    Pointer<Uint64>, Pointer<Pointer<_EliminatedTeamNative>>);

final _PredictSomeTournament _predictSomeTournamentNative = _lop
    .lookup<NativeFunction<_PredictSomeTournamentNative>>(
        "predict_some_tournaments")
    .asFunction();

typedef _PredictSomeTournamentFreeNative = Void Function(
    Pointer<Pointer<_EliminatedTeamNative>>);
typedef _PredictSomeTournamentFree = void Function(
    Pointer<Pointer<_EliminatedTeamNative>>);

final _PredictSomeTournamentFree _predictSomeTournamentFreeNative = _lop
    .lookup<NativeFunction<_PredictSomeTournamentFreeNative>>(
        "predict_some_tournaments_free")
    .asFunction();

List<EliminatedTeam> predictSomeTournament() {
  // FIXME: Answer Stackoverflow question.
  // SEE: https://stackoverflow.com/questions/67313913/dart-flutter-ffiforeign-function-interface-calling-a-native-function-with-out.
  final Pointer<Uint64> x = calloc.allocate<Uint64>(sizeOf<Uint64>());
  final Pointer<Pointer<_EliminatedTeamNative>> y =
      calloc.allocate<Pointer<_EliminatedTeamNative>>(
          sizeOf<Pointer<_EliminatedTeamNative>>());
  final z = _predictSomeTournamentNative(x, y);
  final int count = x.value;

  final List<EliminatedTeam> ts = [];
  for (var i = 0; i < count; i++) {
    final String id = y.value[i].id.toDartString();
    final List<String> eids = <String>[];

    for (var j = 0; j < y.value[i].eliminating_teams_ids_count; j++) {
      eids.add(y.value[i].eliminating_teams_ids[j].toDartString());
    }

    ts.add(EliminatedTeam(id, eids));
  }

  _predictSomeTournamentFreeNative(y);

  return ts;
}
