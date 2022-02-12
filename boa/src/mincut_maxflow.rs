pub(super) mod common;
mod residual_edge;
mod residual_graph;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::Arc;

use self::common::Flow;
use self::common::FlowEdge;
use self::common::FlowNode;
use self::residual_edge::ResidualEdge;
use self::residual_graph::ResidualGraph;

type EdgeTo = Arc<RefCell<BTreeMap<FlowNode, Arc<RefCell<ResidualEdge>>>>>;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct MincutMaxflow {
  pub(super) mincut: HashSet<FlowNode>,
  pub(super) maxflow: Flow,
  pub(super) source_full: bool,
  constructor_guard: PhantomData<()>,
}

pub(super) fn calculate_mincut_maxflow(
  edges: &[FlowEdge],
  source_node: &FlowNode,
  sink_node: &FlowNode,
) -> MincutMaxflow {
  let graph = ResidualGraph::new(edges, source_node, sink_node);

  let mut current_max_flow = Flow::Regular(0);
  ensure_feasibility(&graph, source_node, sink_node, current_max_flow);
  current_max_flow = get_excess(&graph, sink_node);

  let edge_to: EdgeTo = Arc::new(RefCell::new(BTreeMap::new()));
  let mut marked = HashSet::new();
  while has_augmenting_path(
    &graph,
    source_node,
    sink_node,
    &edge_to,
    &mut marked,
  ) {
    let bottlenick =
      get_bottlenick(source_node, sink_node, Arc::clone(&edge_to));
    current_max_flow = augment_flow(
      source_node,
      sink_node,
      Arc::clone(&edge_to),
      bottlenick,
      current_max_flow,
    );
  }

  let source_full = graph
    .edges(source_node)
    .iter()
    .filter(|node| node.borrow().from == *source_node)
    .all(|node| node.borrow().capacity == node.borrow().flow);

  let mincut_maxflow = MincutMaxflow {
    mincut: marked,
    maxflow: current_max_flow,
    source_full,
    constructor_guard: PhantomData,
  };
  ensure_optimality(&graph, source_node, sink_node, &mincut_maxflow);

  mincut_maxflow
}

fn ensure_feasibility(
  graph: &ResidualGraph,
  source_node: &FlowNode,
  sink_node: &FlowNode,
  current_max_flow: Flow,
) {
  let source_excess = current_max_flow + get_excess(graph, source_node);
  assert!(
    source_excess == Flow::Regular(0),
    "Invalid excess at source ({:?}).",
    source_excess,
  );

  let sink_excess = current_max_flow - get_excess(graph, sink_node);
  assert!(
    sink_excess == Flow::Regular(0),
    "Invalid excess at sink ({:?}).",
    sink_excess,
  );

  for node in graph.nodes() {
    if node == source_node || node == sink_node {
      continue;
    }

    let excess = get_excess(graph, node);
    assert!(
      excess == Flow::Regular(0),
      "Invalid net flow out of ({:?}).",
      node,
    );
  }
}

fn get_excess(graph: &ResidualGraph, node: &FlowNode) -> Flow {
  let mut excess = Flow::Regular(0);

  for edge in graph.edges(node) {
    if *node == edge.borrow().from {
      excess -= edge.borrow().flow;
    } else {
      excess += edge.borrow().flow;
    }
  }

  excess
}

fn has_augmenting_path(
  graph: &ResidualGraph,
  source_node: &FlowNode,
  sink_node: &FlowNode,
  edge_to: &EdgeTo,
  marked: &mut HashSet<FlowNode>,
) -> bool {
  edge_to.borrow_mut().clear();
  marked.clear();

  let mut queue: VecDeque<FlowNode> = VecDeque::new();
  queue.push_back(FlowNode::clone(source_node));
  marked.insert(FlowNode::clone(source_node));

  while !queue.is_empty() && !marked.contains(sink_node) {
    let node = queue.pop_front().unwrap();
    for edge in graph.edges(&node) {
      let other = FlowNode::clone(edge.borrow().other(&node));
      if marked.contains(&other)
        || edge.borrow().residual_capacity_to(&other) <= Flow::Regular(0)
      {
        continue;
      }

      edge_to
        .borrow_mut()
        .insert(FlowNode::clone(&other), Arc::clone(edge));

      marked.insert(FlowNode::clone(&other));
      queue.push_back(FlowNode::clone(&other));
    }
  }

  marked.contains(sink_node)
}

struct SinkToSourceIterator {
  edge_to: EdgeTo,
  source_node: FlowNode,
  current_node: FlowNode,
}

impl SinkToSourceIterator {
  fn new(source_node: &FlowNode, sink: &FlowNode, edge_to: EdgeTo) -> Self {
    Self {
      edge_to,
      source_node: FlowNode::clone(source_node),
      current_node: FlowNode::clone(sink),
    }
  }
}

impl Iterator for SinkToSourceIterator {
  type Item = (FlowNode, Arc<RefCell<ResidualEdge>>);

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_node == self.source_node {
      return None;
    }

    let edge =
      Arc::clone(self.edge_to.borrow().get(&self.current_node).unwrap());
    let node = FlowNode::clone(&self.current_node);
    self.current_node =
      FlowNode::clone(edge.borrow().other(&self.current_node));

    Some((node, edge))
  }
}

fn get_bottlenick(
  source_node: &FlowNode,
  sink_node: &FlowNode,
  edge_to: EdgeTo,
) -> Flow {
  let mut bottlenick = Flow::Infinite;

  for (node, edge) in SinkToSourceIterator::new(source_node, sink_node, edge_to)
  {
    let residual = edge.borrow().residual_capacity_to(&node);
    bottlenick = bottlenick.min(residual);
  }

  bottlenick
}

fn augment_flow(
  source_node: &FlowNode,
  sink_node: &FlowNode,
  edge_to: EdgeTo,
  bottlenick: Flow,
  maxflow: Flow,
) -> Flow {
  for (node, edge) in SinkToSourceIterator::new(source_node, sink_node, edge_to)
  {
    edge.borrow_mut().add_residual_flow_to(&node, bottlenick);
  }

  bottlenick + maxflow
}

fn ensure_optimality(
  graph: &ResidualGraph,
  source_node: &FlowNode,
  sink_node: &FlowNode,
  mincut_maxflow: &MincutMaxflow,
) {
  ensure_feasibility(graph, source_node, sink_node, mincut_maxflow.maxflow);

  assert!(
    mincut_maxflow.mincut.contains(source_node),
    "Source not in min-cut ({:?}. {:?}).",
    source_node,
    mincut_maxflow.mincut,
  );
  assert!(
    !mincut_maxflow.mincut.contains(sink_node),
    "Sink in min-cut ({:?}, {:?}).",
    sink_node,
    mincut_maxflow.mincut,
  );

  let mincut_flow = get_mincut_flow(graph, mincut_maxflow);
  assert!(
    mincut_maxflow.maxflow == mincut_flow,
    "Max-flow flow ({:?}) doesn't match min-cut flow ({:?}).",
    mincut_maxflow.maxflow,
    mincut_flow,
  );
}

fn get_mincut_flow(
  graph: &ResidualGraph,
  mincut_maxflow: &MincutMaxflow,
) -> Flow {
  let mut max_flow_of_mincut = Flow::Regular(0);

  for node in &mincut_maxflow.mincut {
    for edge in graph.edges(node) {
      if edge.borrow().from != *node
        || mincut_maxflow.mincut.contains(&edge.borrow().to)
      {
        continue;
      }

      max_flow_of_mincut += edge.borrow().capacity;
    }
  }

  max_flow_of_mincut
}

struct TestExample {
  edges: Vec<FlowEdge>,
  expected_mincut_maxflow: MincutMaxflow,
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  let source_node = "s";
  let sink_node = "t";

  let examples = vec![
    TestExample {
      edges: vec![
        (source_node, "1", Flow::Regular(10)),
        (source_node, "2", Flow::Regular(5)),
        (source_node, "3", Flow::Regular(15)),
        ("1", "2", Flow::Regular(4)),
        ("1", "4", Flow::Regular(9)),
        ("1", "5", Flow::Regular(15)),
        ("2", "3", Flow::Regular(4)),
        ("2", "5", Flow::Regular(8)),
        ("3", "6", Flow::Regular(16)),
        ("4", "5", Flow::Regular(15)),
        ("4", sink_node, Flow::Regular(10)),
        ("5", "6", Flow::Regular(15)),
        ("5", sink_node, Flow::Regular(10)),
        ("6", "2", Flow::Regular(6)),
        ("6", sink_node, Flow::Regular(10)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| {
        FlowEdge::new(
          FlowNode::new(Arc::new(from.to_string())),
          FlowNode::new(Arc::new(to.to_string())),
          capacity,
        )
      })
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![source_node, "2", "3", "6"]
          .into_iter()
          .map(|s| FlowNode::new(Arc::new(s.to_string())))
          .collect(),
        maxflow: Flow::Regular(28),
        source_full: false,
        constructor_guard: PhantomData,
      },
    },
    TestExample {
      edges: vec![
        (source_node, "1", Flow::Regular(10)),
        (source_node, "2", Flow::Regular(5)),
        ("1", "3", Flow::Regular(9)),
        ("1", "4", Flow::Regular(4)),
        ("2", "1", Flow::Regular(4)),
        ("2", "4", Flow::Regular(8)),
        ("3", "4", Flow::Regular(15)),
        ("3", sink_node, Flow::Regular(10)),
        ("4", sink_node, Flow::Regular(10)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| {
        FlowEdge::new(
          FlowNode::new(Arc::new(from.to_string())),
          FlowNode::new(Arc::new(to.to_string())),
          capacity,
        )
      })
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![source_node]
          .into_iter()
          .map(|s| FlowNode::new(Arc::new(s.to_string())))
          .collect(),
        maxflow: Flow::Regular(15),
        source_full: true,
        constructor_guard: PhantomData,
      },
    },
    TestExample {
      edges: vec![
        (source_node, "alice", Flow::Regular(1)),
        (source_node, "bob", Flow::Regular(1)),
        (source_node, "carol", Flow::Regular(1)),
        (source_node, "dave", Flow::Regular(1)),
        (source_node, "eliza", Flow::Regular(1)),
        ("alice", "adobe", Flow::Infinite),
        ("alice", "amazon", Flow::Infinite),
        ("alice", "google", Flow::Infinite),
        ("bob", "adobe", Flow::Infinite),
        ("bob", "amazon", Flow::Infinite),
        ("carol", "adobe", Flow::Infinite),
        ("carol", "facebook", Flow::Infinite),
        ("carol", "google", Flow::Infinite),
        ("dave", "amazon", Flow::Infinite),
        ("dave", "yahoo", Flow::Infinite),
        ("eliza", "amazon", Flow::Infinite),
        ("eliza", "yahoo", Flow::Infinite),
        ("adobe", sink_node, Flow::Regular(1)),
        ("amazon", sink_node, Flow::Regular(1)),
        ("facebook", sink_node, Flow::Regular(1)),
        ("google", sink_node, Flow::Regular(1)),
        ("yahoo", sink_node, Flow::Regular(1)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| {
        FlowEdge::new(
          FlowNode::new(Arc::new(from.to_string())),
          FlowNode::new(Arc::new(to.to_string())),
          capacity,
        )
      })
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![source_node]
          .into_iter()
          .map(|s| FlowNode::new(Arc::new(s.to_string())))
          .collect(),
        maxflow: Flow::Regular(5),
        source_full: true,
        constructor_guard: PhantomData,
      },
    },
    TestExample {
      edges: vec![
        (source_node, "1", Flow::Regular(100)),
        (source_node, "2", Flow::Regular(100)),
        ("1", "2", Flow::Regular(1)),
        ("1", sink_node, Flow::Regular(100)),
        ("2", sink_node, Flow::Regular(100)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| {
        FlowEdge::new(
          FlowNode::new(Arc::new(from.to_string())),
          FlowNode::new(Arc::new(to.to_string())),
          capacity,
        )
      })
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![source_node]
          .into_iter()
          .map(|s| FlowNode::new(Arc::new(s.to_string())))
          .collect(),
        maxflow: Flow::Regular(200),
        source_full: true,
        constructor_guard: PhantomData,
      },
    },
  ];

  for TestExample {
    edges,
    expected_mincut_maxflow,
  } in examples
  {
    assert_eq!(
      calculate_mincut_maxflow(
        &edges,
        &FlowNode::new(Arc::new(source_node.to_string())),
        &FlowNode::new(Arc::new(sink_node.to_string()))
      ),
      expected_mincut_maxflow
    );
  }
}

#[cfg(test)]
mod tests {
  use super::test;

  #[test]
  fn test_mincut_maxflow() {
    test()
  }
}
