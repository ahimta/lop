use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::mincut_maxflow::residual_edge::ResidualEdge;
use crate::mincut_maxflow::FlowEdge;
use crate::mincut_maxflow::FlowNode;

#[must_use]
pub(super) struct ResidualGraph {
  adjacency_matrix: BTreeMap<FlowNode, Vec<Arc<RefCell<ResidualEdge>>>>,
  nodes: HashSet<FlowNode>,
  constructor_guard: PhantomData<()>,
}

impl ResidualGraph {
  #[must_use]
  pub(super) fn new(
    edges: &[FlowEdge],
    source_node: &FlowNode,
    sink_node: &FlowNode,
  ) -> Self {
    const EDGES_COUNT_MIN: usize = 1;
    const EDGES_COUNT_MAX: usize = 10 * 1000;

    assert!(
      edges.len() >= EDGES_COUNT_MIN && edges.len() <= EDGES_COUNT_MAX,
      "Invalid edges length ({:?}).",
      edges.len(),
    );
    assert!(
      edges.len()
        == edges
          .iter()
          .map(|FlowEdge { from, to, .. }| from.join(to))
          .collect::<HashSet<_>>()
          .len(),
      "Duplicate edges ({:?}).",
      edges,
    );

    let nodes: HashSet<FlowNode> = edges
      .iter()
      .flat_map(|FlowEdge { from, to, .. }| {
        vec![FlowNode::clone(from), FlowNode::clone(to)]
      })
      .collect();

    assert!(
      nodes.contains(source_node) && nodes.contains(sink_node),
      "Invalid source or sink ({:?}, {:?}, {:?}).",
      source_node,
      sink_node,
      nodes,
    );
    assert!(
      source_node != sink_node,
      "Source must not equal sink ({:?}, {:?}).",
      source_node,
      sink_node,
    );

    // NOTE: `BTreeMap` used instead of `HashMap` because `RefCell` doesn't seem
    // to work with `HashMap`. This is contagious and causes other components to
    // use the same type.
    let mut adjacency_matrix: BTreeMap<
      FlowNode,
      Vec<Arc<RefCell<ResidualEdge>>>,
    > = BTreeMap::new();
    for node in &nodes {
      adjacency_matrix.insert(FlowNode::clone(node), Vec::new());
    }

    for edge in edges {
      let shared_edge = Arc::new(RefCell::new(ResidualEdge::new(
        FlowNode::clone(&edge.from),
        FlowNode::clone(&edge.to),
        edge.capacity,
      )));
      adjacency_matrix
        .get_mut(&edge.from)
        .unwrap()
        .push(Arc::clone(&shared_edge));
      adjacency_matrix
        .get_mut(&edge.to)
        .unwrap()
        .push(Arc::clone(&shared_edge));
    }

    Self {
      adjacency_matrix,
      nodes,
      constructor_guard: PhantomData,
    }
  }

  #[must_use]
  pub(super) fn edges(&self, node: &FlowNode) -> &[Arc<RefCell<ResidualEdge>>] {
    self.adjacency_matrix.get(node).unwrap()
  }

  #[must_use]
  pub(super) const fn nodes(&self) -> &HashSet<FlowNode> {
    &self.nodes
  }
}
