import 'package:flutter/material.dart';

import 'lop.dart';

void main() => runApp(const MyApp());

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      title: 'Tournament Elimination',
      home: EliminatedTeamsWidget(),
    );
  }
}

class EliminatedTeamsWidget extends StatefulWidget {
  const EliminatedTeamsWidget({Key? key}) : super(key: key);

  @override
  _EliminatedTeamsWidgetState createState() => _EliminatedTeamsWidgetState();
}

class _EliminatedTeamsWidgetState extends State<EliminatedTeamsWidget> {
  final _eliminatedTeams = predictSomeTournament();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Eliminated Teams'),
      ),
      body: ListView(
        padding: const EdgeInsets.symmetric(vertical: 8),
        children: _eliminatedTeams
            .map((e) => ListTile(
                  leading: const Icon(
                    Icons.facebook,
                    size: 42,
                  ),
                  title: Text(e.id),
                  subtitle: Text(e.eliminatingTeamsIds.join(', ')),
                ))
            .toList(),
      ),
    );
  }
}
