import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'boa.dart';

void main() => runApp(const MyApp());

class MyApp extends StatelessWidget {
  // ignore: prefer_final_parameters
  const MyApp({super.key});

  @override
  Widget build(final BuildContext context) => const MaterialApp(
        title: 'Tournament Elimination',
        home: _TournamentsWidget(),
      );
}

class _TournamentsWidget extends StatefulWidget {
  // ignore: prefer_final_parameters, unused_element
  const _TournamentsWidget({super.key});

  @override
  _TournamentsWidgetState createState() => _TournamentsWidgetState();
}

class _TournamentsWidgetState extends State<_TournamentsWidget> {
  final _tournaments = getTournaments();

  @override
  Widget build(final BuildContext context) => Scaffold(
        appBar: AppBar(
          title: const Text('Tournaments'),
        ),
        body: _tournaments.isEmpty
            ? const Center(
                child: Text('No tournaments available!'),
              )
            : ListView(
                padding: const EdgeInsets.symmetric(vertical: 8),
                children: _tournaments
                    .map(
                      (final tournament) => ListTile(
                        leading: const Icon(
                          Icons.facebook,
                          size: 42,
                        ),
                        title: Text(tournament.name),
                        onTap: () => Navigator.push<void>(
                          context,
                          MaterialPageRoute(
                            builder: (final context) =>
                                _TeamsWidget(tournament: tournament),
                          ),
                        ),
                      ),
                    )
                    .toList(),
              ),
      );
}

class _TeamsWidget extends StatelessWidget {
  const _TeamsWidget({
    required this.tournament,
    // ignore: prefer_final_parameters, unused_element
    super.key,
  });

  final Tournament tournament;

  @override
  void debugFillProperties(final DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(DiagnosticsProperty<Tournament>('tournament', tournament));
  }

  @override
  Widget build(final BuildContext context) => Scaffold(
        appBar: AppBar(
          title: Text(tournament.name),
        ),
        body: tournament.teams.isEmpty
            ? const Center(
                child: Text('No teams available!'),
              )
            : SingleChildScrollView(
                // ignore: avoid_redundant_argument_values
                scrollDirection: Axis.vertical,
                child: SingleChildScrollView(
                  scrollDirection: Axis.horizontal,
                  child: DataTable(
                    columns: const <DataColumn>[
                      DataColumn(
                        label: Text('Rank'),
                      ),
                      DataColumn(
                        label: Text('Name'),
                      ),
                      DataColumn(
                        label: Text('Matches Left'),
                      ),
                      DataColumn(
                        label: Text('Matches Drawn'),
                      ),
                      DataColumn(
                        label: Text('Matches Won'),
                      ),
                      DataColumn(
                        label: Text('Earned Points'),
                      ),
                      DataColumn(
                        label: Text('Remaining Points'),
                      ),
                      DataColumn(
                        label: Text('Elimination Status'),
                      ),
                      DataColumn(
                        label: Text('Eliminating Teams'),
                      ),
                    ],
                    rows: tournament.teams
                        .map(
                          (final team) => DataRow(
                            cells: <DataCell>[
                              DataCell(Text(team.rank.toString())),
                              DataCell(Text(team.name)),
                              DataCell(Text(team.matchesLeft.toString())),
                              DataCell(Text(team.matchesDrawn.toString())),
                              DataCell(Text(team.matchesWon.toString())),
                              DataCell(Text(team.earnedPoints.toString())),
                              DataCell(Text(team.remainingPoints.toString())),
                              DataCell(Text(team.eliminationStatus.toString())),
                              DataCell(Text(team.eliminatingTeams.join(', '))),
                            ],
                          ),
                        )
                        .toList(),
                  ),
                ),
              ),
      );
}
