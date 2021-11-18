import 'package:flutter/material.dart';

import 'boa.dart';

void main() => runApp(const MyApp());

class MyApp extends StatelessWidget {
  const MyApp({final Key? key}) : super(key: key);

  @override
  Widget build(final BuildContext context) => const MaterialApp(
        title: 'Tournament Elimination',
        home: EliminatedTeamsWidget(),
      );
}

class EliminatedTeamsWidget extends StatefulWidget {
  const EliminatedTeamsWidget({final Key? key}) : super(key: key);

  @override
  _EliminatedTeamsWidgetState createState() => _EliminatedTeamsWidgetState();
}

class _EliminatedTeamsWidgetState extends State<EliminatedTeamsWidget> {
  final _eliminatedTeams = predictTournamentEliminatedTeams();

  @override
  Widget build(final BuildContext context) => Scaffold(
        appBar: AppBar(
          title: const Text('Eliminated Teams'),
        ),
        body: ListView(
          padding: const EdgeInsets.symmetric(vertical: 8),
          children: _eliminatedTeams
              .map(
                (final eliminatedTeam) => ListTile(
                  leading: const Icon(
                    Icons.facebook,
                    size: 42,
                  ),
                  title: Text(eliminatedTeam.id),
                  subtitle: Text(eliminatedTeam.eliminatingTeamsIds.join(', ')),
                ),
              )
              .toList(),
        ),
      );
}
