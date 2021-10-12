use std::marker::PhantomData;

use super::common::ensure_valid_edge_nodes;
use super::common::Flow;
use super::common::FlowNode;

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
    if new_flow > self.capacity {
      panic!("Overflow ({:?}, {:?}).", self, delta);
    }
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
    if !(*node == self.from || *node == self.to) {
      panic!("Invalid node ({:?}, {:?}).", self, node);
    }
  }
}
