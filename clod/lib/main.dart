import 'package:flutter/material.dart';

import 'boa.dart';

void main() => runApp(const MyApp());

class MyApp extends StatelessWidget {
  // ignore: prefer_final_parameters
  const MyApp({super.key});

  @override
  Widget build(final BuildContext context) => const MaterialApp(
        title: 'Tournament Elimination',
        home: _EliminatedTeamsWidget(),
      );
}

class _EliminatedTeamsWidget extends StatefulWidget {
  // ignore: prefer_final_parameters, unused_element
  const _EliminatedTeamsWidget({super.key});

  @override
  _EliminatedTeamsWidgetState createState() => _EliminatedTeamsWidgetState();
}

class _EliminatedTeamsWidgetState extends State<_EliminatedTeamsWidget> {
  final _eliminatedTeams = predictTournamentEliminatedTeams();

  @override
  Widget build(final BuildContext context) => Scaffold(
        appBar: AppBar(
          title: const Text('Eliminated Teams'),
        ),
        body: _eliminatedTeams.isEmpty
            ? const Center(
                child:
                    Text('No eliminated teams: all teams have a chance to win'),
              )
            : ListView(
                padding: const EdgeInsets.symmetric(vertical: 8),
                children: _eliminatedTeams
                    .map(
                      (final eliminatedTeam) => ListTile(
                        leading: const Icon(
                          Icons.facebook,
                          size: 42,
                        ),
                        title: Text(eliminatedTeam.id),
                        subtitle:
                            Text(eliminatedTeam.eliminatingTeamsIds.join(', ')),
                      ),
                    )
                    .toList(),
              ),
      );
}
