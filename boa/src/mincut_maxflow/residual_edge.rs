use std::marker::PhantomData;

use crate::mincut_maxflow::common::ensure_valid_edge_nodes;
use crate::mincut_maxflow::common::Flow;
use crate::mincut_maxflow::common::FlowNode;

#[derive(Debug)]
pub(super) struct ResidualEdge {
  pub(super) from: FlowNode,
  pub(super) to: FlowNode,
  pub(super) capacity: Flow,
  pub(super) flow: Flow,
  constructor_guard: PhantomData<()>,
}

impl ResidualEdge {
  pub(super) fn new(from: FlowNode, to: FlowNode, capacity: Flow) -> Self {
    ensure_valid_edge_nodes(&from, &to);
    Self {
      from,
      to,
      capacity,
      flow: Flow::Regular(0),
      constructor_guard: PhantomData,
    }
  }

  pub(super) fn add_residual_flow_to(&mut self, node: &FlowNode, delta: Flow) {
    self.ensure_valid_node(node);

    let new_flow = if *node == self.from {
      self.flow - delta
    } else {
      self.flow + delta
    };
    assert!(
      new_flow <= self.capacity,
      "Overflow ({:?}, {:?}).",
      self,
      delta,
    );
    self.flow = new_flow;
  }

  pub(super) fn other(&self, node: &FlowNode) -> &FlowNode {
    self.ensure_valid_node(node);

    if *node == self.from {
      &self.to
    } else {
      &self.from
    }
  }

  pub(super) fn residual_capacity_to(&self, node: &FlowNode) -> Flow {
    self.ensure_valid_node(node);

    if *node == self.from {
      self.flow
    } else {
      self.capacity - self.flow
    }
  }

  fn ensure_valid_node(&self, node: &FlowNode) {
    assert!(
      *node == self.from || *node == self.to,
      "Invalid node ({:?}, {:?}).",
      self,
      node,
    );
  }
}
