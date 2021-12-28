use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;

use super::residual_edge::ResidualEdge;
use super::FlowEdge;
use super::FlowNode;

pub(super) struct ResidualGraph {
  adjacency_matrix: BTreeMap<FlowNode, Vec<Rc<RefCell<ResidualEdge>>>>,
  nodes: HashSet<FlowNode>,
  constructor_guard: PhantomData<()>,
}

impl ResidualGraph {
  pub(super) fn new(
    edges: &[FlowEdge],
    source_node: &FlowNode,
    sink_node: &FlowNode,
  ) -> Self {
    const EDGES_COUNT_MIN: usize = 1;
    const EDGES_COUNT_MAX: usize = 10 * 1000;

    if edges.len() < EDGES_COUNT_MIN || edges.len() > EDGES_COUNT_MAX {
      panic!("Invalid edges length ({:?}).", edges.len());
    }
    if edges.len()
      != edges
        .iter()
        .map(|FlowEdge { from, to, .. }| from.join(to))
        .collect::<HashSet<_>>()
        .len()
    {
      panic!("Duplicate edges ({:?}).", edges);
    }

    let nodes: HashSet<FlowNode> = edges
      .iter()
      .flat_map(|FlowEdge { from, to, .. }| {
        vec![FlowNode::clone(from), FlowNode::clone(to)]
      })
      .collect();

    if !nodes.contains(source_node) || !nodes.contains(sink_node) {
      panic!(
        "Invalid source or sink ({:?}, {:?}, {:?}).",
        source_node, sink_node, nodes
      );
    }
    if source_node == sink_node {
      panic!(
        "Source must not equal sink ({:?}, {:?}).",
        source_node, sink_node
      );
    }

    // NOTE: `BTreeMap` used instead of `HashMap` because `RefCell` doesn't seem
    // to work with `HashMap`. This is contagious and causes other components to
    // use the same type.
    let mut adjacency_matrix: BTreeMap<
      FlowNode,
      Vec<Rc<RefCell<ResidualEdge>>>,
    > = BTreeMap::new();
    for node in &nodes {
      adjacency_matrix.insert(FlowNode::clone(node), Vec::new());
    }

    for edge in edges {
      let shared_edge = Rc::new(RefCell::new(ResidualEdge::new(
        FlowNode::clone(&edge.from),
        FlowNode::clone(&edge.to),
        edge.capacity,
      )));
      adjacency_matrix
        .get_mut(&edge.from)
        .unwrap()
        .push(Rc::clone(&shared_edge));
      adjacency_matrix
        .get_mut(&edge.to)
        .unwrap()
        .push(Rc::clone(&shared_edge));
    }

    Self {
      adjacency_matrix,
      nodes,
      constructor_guard: PhantomData,
    }
  }

  pub(super) fn edges(&self, node: &FlowNode) -> &[Rc<RefCell<ResidualEdge>>] {
    self.adjacency_matrix.get(node).unwrap()
  }

  pub(super) const fn nodes(&self) -> &HashSet<FlowNode> {
    &self.nodes
  }
}
