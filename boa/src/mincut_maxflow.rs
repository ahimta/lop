pub(super) mod common;
mod residual_edge;
mod residual_graph;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::mincut_maxflow::common::Flow;
use crate::mincut_maxflow::common::FlowEdge;
use crate::mincut_maxflow::common::FlowNode;
use crate::mincut_maxflow::residual_edge::ResidualEdge;
use crate::mincut_maxflow::residual_graph::ResidualGraph;

type EdgeTo = Arc<RefCell<BTreeMap<Arc<FlowNode>, Arc<RefCell<ResidualEdge>>>>>;

#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub(super) struct MincutMaxflow {
  pub(super) mincut: HashSet<Arc<FlowNode>>,
  pub(super) maxflow: Flow,
  pub(super) source_full: bool,
  constructor_guard: PhantomData<()>,
}

#[must_use]
pub(super) fn calculate_mincut_maxflow(
  edges: &[FlowEdge],
  source_node: &Arc<FlowNode>,
  sink_node: &Arc<FlowNode>,
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
    let bottlenick = get_bottlenick(source_node, sink_node, &edge_to);
    current_max_flow = augment_flow(
      source_node,
      sink_node,
      &edge_to,
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
    if &**node == source_node || &**node == sink_node {
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

#[must_use]
fn get_excess(graph: &ResidualGraph, node: &FlowNode) -> Flow {
  let mut excess = Flow::Regular(0);

  for edge in graph.edges(node) {
    if *node == *edge.borrow().from {
      excess -= edge.borrow().flow;
    } else {
      excess += edge.borrow().flow;
    }
  }

  excess
}

#[must_use]
fn has_augmenting_path(
  graph: &ResidualGraph,
  source_node: &Arc<FlowNode>,
  sink_node: &Arc<FlowNode>,
  edge_to: &EdgeTo,
  marked: &mut HashSet<Arc<FlowNode>>,
) -> bool {
  edge_to.borrow_mut().clear();
  marked.clear();

  let mut queue: VecDeque<Arc<FlowNode>> = VecDeque::new();
  queue.push_back(Arc::clone(source_node));
  marked.insert(Arc::clone(source_node));

  while !queue.is_empty() && !marked.contains(sink_node) {
    let node = queue.pop_front().unwrap();
    for edge in graph.edges(&node) {
      let other = Arc::clone(edge.borrow().other(&node));
      if marked.contains(&other)
        || edge.borrow().residual_capacity_to(&other) <= Flow::Regular(0)
      {
        continue;
      }

      edge_to
        .borrow_mut()
        .insert(Arc::clone(&other), Arc::clone(edge));

      marked.insert(Arc::clone(&other));
      queue.push_back(Arc::clone(&other));
    }
  }

  marked.contains(sink_node)
}

#[must_use]
struct SinkToSourceIterator {
  edge_to: EdgeTo,
  source_node: Arc<FlowNode>,
  current_node: Arc<FlowNode>,
}

impl SinkToSourceIterator {
  #[must_use]
  fn new(
    source_node: &Arc<FlowNode>,
    sink: &Arc<FlowNode>,
    edge_to: &EdgeTo,
  ) -> Self {
    Self {
      edge_to: Arc::clone(edge_to),
      source_node: Arc::clone(source_node),
      current_node: Arc::clone(sink),
    }
  }
}

impl Iterator for SinkToSourceIterator {
  type Item = (Arc<FlowNode>, Arc<RefCell<ResidualEdge>>);

  #[must_use]
  fn next(&mut self) -> Option<Self::Item> {
    if self.current_node == self.source_node {
      return None;
    }

    let edge =
      Arc::clone(self.edge_to.borrow().get(&self.current_node).unwrap());
    let node = Arc::clone(&self.current_node);
    self.current_node = Arc::clone(edge.borrow().other(&self.current_node));

    Some((node, edge))
  }
}

#[must_use]
fn get_bottlenick(
  source_node: &Arc<FlowNode>,
  sink_node: &Arc<FlowNode>,
  edge_to: &EdgeTo,
) -> Flow {
  let mut bottlenick = Flow::Infinite;

  for (node, edge) in SinkToSourceIterator::new(source_node, sink_node, edge_to)
  {
    let residual = edge.borrow().residual_capacity_to(&node);
    bottlenick = bottlenick.min(residual);
  }

  bottlenick
}

#[must_use]
fn augment_flow(
  source_node: &Arc<FlowNode>,
  sink_node: &Arc<FlowNode>,
  edge_to: &EdgeTo,
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

#[must_use]
fn get_mincut_flow(
  graph: &ResidualGraph,
  mincut_maxflow: &MincutMaxflow,
) -> Flow {
  let mut max_flow_of_mincut = Flow::Regular(0);

  for node in &mincut_maxflow.mincut {
    for edge in graph.edges(node) {
      if &edge.borrow().from != node
        || mincut_maxflow.mincut.contains(&edge.borrow().to)
      {
        continue;
      }

      max_flow_of_mincut += edge.borrow().capacity;
    }
  }

  max_flow_of_mincut
}

#[must_use]
struct TestExample {
  edges: Vec<FlowEdge>,
  expected_mincut_maxflow: MincutMaxflow,
}

#[allow(clippy::too_many_lines)]
pub(super) fn test() {
  #[cfg(test)]
  use pretty_assertions::assert_eq;

  let examples = vec![
    TestExample {
      edges: vec![
        (FlowNode::source(), make_node("1"), Flow::Regular(10)),
        (FlowNode::source(), make_node("2"), Flow::Regular(5)),
        (FlowNode::source(), make_node("3"), Flow::Regular(15)),
        (make_node("1"), make_node("2"), Flow::Regular(4)),
        (make_node("1"), make_node("4"), Flow::Regular(9)),
        (make_node("1"), make_node("5"), Flow::Regular(15)),
        (make_node("2"), make_node("3"), Flow::Regular(4)),
        (make_node("2"), make_node("5"), Flow::Regular(8)),
        (make_node("3"), make_node("6"), Flow::Regular(16)),
        (make_node("4"), make_node("5"), Flow::Regular(15)),
        (make_node("4"), FlowNode::sink(), Flow::Regular(10)),
        (make_node("5"), make_node("6"), Flow::Regular(15)),
        (make_node("5"), FlowNode::sink(), Flow::Regular(10)),
        (make_node("6"), make_node("2"), Flow::Regular(6)),
        (make_node("6"), FlowNode::sink(), Flow::Regular(10)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| FlowEdge::new(&from, &to, capacity))
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![
          FlowNode::source(),
          make_node("2"),
          make_node("3"),
          make_node("6"),
        ]
        .into_iter()
        .collect(),
        maxflow: Flow::Regular(28),
        source_full: false,
        constructor_guard: PhantomData,
      },
    },
    TestExample {
      edges: vec![
        (FlowNode::source(), make_node("1"), Flow::Regular(10)),
        (FlowNode::source(), make_node("2"), Flow::Regular(5)),
        (make_node("1"), make_node("3"), Flow::Regular(9)),
        (make_node("1"), make_node("4"), Flow::Regular(4)),
        (make_node("2"), make_node("1"), Flow::Regular(4)),
        (make_node("2"), make_node("4"), Flow::Regular(8)),
        (make_node("3"), make_node("4"), Flow::Regular(15)),
        (make_node("3"), FlowNode::sink(), Flow::Regular(10)),
        (make_node("4"), FlowNode::sink(), Flow::Regular(10)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| FlowEdge::new(&from, &to, capacity))
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![FlowNode::source()].into_iter().collect(),
        maxflow: Flow::Regular(15),
        source_full: true,
        constructor_guard: PhantomData,
      },
    },
    TestExample {
      edges: vec![
        (FlowNode::source(), make_node("alice"), Flow::Regular(1)),
        (FlowNode::source(), make_node("bob"), Flow::Regular(1)),
        (FlowNode::source(), make_node("carol"), Flow::Regular(1)),
        (FlowNode::source(), make_node("dave"), Flow::Regular(1)),
        (FlowNode::source(), make_node("eliza"), Flow::Regular(1)),
        (make_node("alice"), make_node("adobe"), Flow::Infinite),
        (make_node("alice"), make_node("amazon"), Flow::Infinite),
        (make_node("alice"), make_node("google"), Flow::Infinite),
        (make_node("bob"), make_node("adobe"), Flow::Infinite),
        (make_node("bob"), make_node("amazon"), Flow::Infinite),
        (make_node("carol"), make_node("adobe"), Flow::Infinite),
        (make_node("carol"), make_node("facebook"), Flow::Infinite),
        (make_node("carol"), make_node("google"), Flow::Infinite),
        (make_node("dave"), make_node("amazon"), Flow::Infinite),
        (make_node("dave"), make_node("yahoo"), Flow::Infinite),
        (make_node("eliza"), make_node("amazon"), Flow::Infinite),
        (make_node("eliza"), make_node("yahoo"), Flow::Infinite),
        (make_node("adobe"), FlowNode::sink(), Flow::Regular(1)),
        (make_node("amazon"), FlowNode::sink(), Flow::Regular(1)),
        (make_node("facebook"), FlowNode::sink(), Flow::Regular(1)),
        (make_node("google"), FlowNode::sink(), Flow::Regular(1)),
        (make_node("yahoo"), FlowNode::sink(), Flow::Regular(1)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| FlowEdge::new(&from, &to, capacity))
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![FlowNode::source()].into_iter().collect(),
        maxflow: Flow::Regular(5),
        source_full: true,
        constructor_guard: PhantomData,
      },
    },
    TestExample {
      edges: vec![
        (FlowNode::source(), make_node("1"), Flow::Regular(100)),
        (FlowNode::source(), make_node("2"), Flow::Regular(100)),
        (make_node("1"), make_node("2"), Flow::Regular(1)),
        (make_node("1"), FlowNode::sink(), Flow::Regular(100)),
        (make_node("2"), FlowNode::sink(), Flow::Regular(100)),
      ]
      .into_iter()
      .map(|(from, to, capacity)| FlowEdge::new(&from, &to, capacity))
      .collect(),

      expected_mincut_maxflow: MincutMaxflow {
        mincut: vec![FlowNode::source()].into_iter().collect(),
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
      calculate_mincut_maxflow(&edges, &FlowNode::source(), &FlowNode::sink(),),
      expected_mincut_maxflow
    );
  }
}

fn make_node(s: &str) -> Arc<FlowNode> {
  Arc::new(FlowNode::new(&Arc::new(String::from(s))))
}

#[cfg(test)]
mod tests {
  use super::test;

  #[test]
  fn test_mincut_maxflow() {
    test()
  }
}
