use std::cmp::Ordering;
use std::convert::TryFrom;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops;
use std::rc::Rc;

pub(super) fn ensure_valid_edge_nodes(from: &FlowNode, to: &FlowNode) {
  if from == to {
    panic!("Invalid edge nodes ({:?}, ({:?}).", from, to);
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) struct FlowNode {
  pub(crate) id: Rc<String>,
  constructor_guard: PhantomData<()>,
}

const JOINED_WITH_TAG: &str = "-joined-with-";

impl FlowNode {
  pub(crate) fn new(id: Rc<String>) -> Self {
    if id.contains(JOINED_WITH_TAG) {
      panic!(
        "Only joined nodes can contain the joined-with tag value ({:?}, {:?}).",
        id, JOINED_WITH_TAG
      );
    }

    Self::internal_new(id)
  }

  pub(crate) fn join(&self, other: &Self) -> Self {
    let (Self { id: node1, .. }, Self { id: node2, .. }) = (self, other);
    Self::internal_new(Rc::new(format!(
      "{}-{}-{}",
      node1.min(node2),
      JOINED_WITH_TAG,
      node1.max(node2),
    )))
  }

  fn internal_new(id: Rc<String>) -> Self {
    const NODE_ID_LENGTH_MIN: usize = 1;
    const NODE_ID_LENGTH_MAX: usize = 10 * 1000;
    if id.len() < NODE_ID_LENGTH_MIN || id.len() > NODE_ID_LENGTH_MAX {
      panic!("Invalid node ID ({:?}).", id);
    }

    Self {
      id,
      constructor_guard: PhantomData,
    }
  }
}

// NOTE: `Flow` can be constructed directly (no constructor-guard) because any
// value for it is valid and its operators handle all cases and panic for
// degenerate cases and it's very hard to beat the current model without adding
// undue complexity.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Flow {
  Infinite,
  Regular(usize),
  NegativeExcess(usize),
}

impl Flow {
  const INFINITE_FLOW_VALUE: isize = isize::MAX;

  fn from(v: isize) -> Self {
    if v == Self::INFINITE_FLOW_VALUE {
      Self::Infinite
    } else if v < 0 {
      Self::NegativeExcess(usize::try_from(v.abs()).unwrap())
    } else {
      Self::Regular(usize::try_from(v).unwrap())
    }
  }

  fn value(&self) -> isize {
    match self {
      Self::Infinite => Self::INFINITE_FLOW_VALUE,
      Self::Regular(v) => isize::try_from(*v).unwrap(),
      Self::NegativeExcess(v) => -(isize::try_from(*v).unwrap()),
    }
  }
}

impl PartialOrd for Flow {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Flow {
  fn cmp(&self, other: &Self) -> Ordering {
    self.value().cmp(&other.value())
  }
}

impl PartialEq for Flow {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}
impl Eq for Flow {}

impl ops::Add for Flow {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    match (self, other) {
      (
        flow1 @ (Self::Regular(_) | Self::NegativeExcess(_)),
        flow2 @ (Self::Regular(_) | Self::NegativeExcess(_)),
      ) => Self::from(flow1.value().checked_add(flow2.value()).unwrap()),
      // NOTE(ACCIDENTAL-FLOW-BLACKHOLE): This behavior is important to prevent
      // accidentally moving flow to an infinity blackhole.
      (Self::Infinite, _) | (_, Self::Infinite) => {
        panic!("Can't add to infinity.")
      }
    }
  }
}

impl ops::Sub for Flow {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    match (self, other) {
      // NOTE(ACCIDENTAL-FLOW-BLACKHOLE): This behavior is important to prevent
      // accidentally moving flow to an infinity blackhole.
      (Self::Infinite, Self::NegativeExcess(_)) => {
        panic!("Can't subtract negative-excess from infinity.")
      }
      (
        flow1 @ (Self::Regular(_) | Self::NegativeExcess(_) | Self::Infinite),
        flow2 @ (Self::Regular(_) | Self::NegativeExcess(_) | Self::Infinite),
      ) => Self::from(flow1.value().checked_sub(flow2.value()).unwrap()),
    }
  }
}

impl ops::AddAssign for Flow {
  fn add_assign(&mut self, other: Self) {
    *self = *self + other;
  }
}

impl ops::SubAssign for Flow {
  fn sub_assign(&mut self, other: Self) {
    *self = *self - other;
  }
}

#[derive(Debug)]
pub(crate) struct FlowEdge {
  pub(crate) from: FlowNode,
  pub(crate) to: FlowNode,
  pub(crate) capacity: Flow,
  constructor_guard: PhantomData<()>,
}

impl FlowEdge {
  pub(crate) fn new(from: FlowNode, to: FlowNode, capacity: Flow) -> Self {
    ensure_valid_edge_nodes(&from, &to);
    Self {
      from,
      to,
      capacity,
      constructor_guard: PhantomData,
    }
  }
}
