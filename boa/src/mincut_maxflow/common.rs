use std::cmp::Ordering;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops;
use std::sync::Arc;

pub(super) fn ensure_valid_edge_nodes(from: &FlowNode, to: &FlowNode) {
  assert!(from != to, "Invalid edge nodes ({:?}, ({:?}).", from, to);
}

#[must_use]
#[derive(Debug)]
pub(crate) struct FlowEdge {
  pub(super) from: Arc<FlowNode>,
  pub(super) to: Arc<FlowNode>,
  pub(super) capacity: Flow,
  constructor_guard: PhantomData<()>,
}

#[must_use]
impl FlowEdge {
  #[must_use]
  pub(crate) fn new(
    from: &Arc<FlowNode>,
    to: &Arc<FlowNode>,
    capacity: Flow,
  ) -> Self {
    ensure_valid_edge_nodes(from, to);

    Self {
      from: Arc::clone(from),
      to: Arc::clone(to),
      capacity,
      constructor_guard: PhantomData,
    }
  }
}

#[must_use]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FlowNode {
  pub(crate) id: Arc<String>,
  constructor_guard: PhantomData<()>,
}

#[must_use]
impl FlowNode {
  const JOINED_WITH_TAG: &'static str = "joined-with";
  const SOURCE_NODE_ID: &'static str = "s";
  const SINK_NODE_ID: &'static str = "t";

  #[must_use]
  pub(crate) fn source() -> Arc<Self> {
    Self::source_sink_helper(Self::SOURCE_NODE_ID)
  }
  #[must_use]
  pub(crate) fn sink() -> Arc<Self> {
    Self::source_sink_helper(Self::SINK_NODE_ID)
  }
  #[must_use]
  fn source_sink_helper(source_or_sink_id: &str) -> Arc<Self> {
    Arc::new(Self::internal_new(Arc::new(String::from(
      source_or_sink_id,
    ))))
  }

  #[must_use]
  pub(crate) fn new(id: &Arc<String>) -> Self {
    assert!(
      **id != Self::SOURCE_NODE_ID
        && **id != Self::SINK_NODE_ID
        && !id.contains(Self::JOINED_WITH_TAG),
      "Invalid ID ({:?}).",
      id,
    );

    Self::internal_new(Arc::clone(id))
  }

  #[must_use]
  pub(crate) fn join(&self, other: &Self) -> Self {
    let (Self { id: node1, .. }, Self { id: node2, .. }) = (self, other);
    Self::internal_new(Arc::new(format!(
      "{}-{}-{}",
      node1.min(node2),
      Self::JOINED_WITH_TAG,
      node1.max(node2),
    )))
  }

  #[must_use]
  fn internal_new(id: Arc<String>) -> Self {
    const NODE_ID_LENGTH_MIN: usize = 1;
    const NODE_ID_LENGTH_MAX: usize = 10 * 1000;
    assert!(
      id.len() >= NODE_ID_LENGTH_MIN && id.len() <= NODE_ID_LENGTH_MAX,
      "Invalid node ID ({:?}).",
      id,
    );

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
#[must_use]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Flow {
  Infinite,
  Regular(usize),
  NegativeExcess(usize),
}

#[must_use]
impl Flow {
  const INFINITE_FLOW_VALUE: isize = isize::MAX;

  #[must_use]
  fn from(v: isize) -> Self {
    if v == Self::INFINITE_FLOW_VALUE {
      Self::Infinite
    } else if v < 0 {
      Self::NegativeExcess(usize::try_from(v.abs()).unwrap())
    } else {
      Self::Regular(usize::try_from(v).unwrap())
    }
  }

  #[must_use]
  fn value(&self) -> isize {
    match self {
      Self::Infinite => Self::INFINITE_FLOW_VALUE,
      Self::Regular(v) => isize::try_from(*v).unwrap(),
      Self::NegativeExcess(v) => -(isize::try_from(*v).unwrap()),
    }
  }
}

#[must_use]
impl PartialOrd for Flow {
  #[must_use]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[must_use]
impl Ord for Flow {
  #[must_use]
  fn cmp(&self, other: &Self) -> Ordering {
    self.value().cmp(&other.value())
  }
}

#[must_use]
impl PartialEq for Flow {
  #[must_use]
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}
#[must_use]
impl Eq for Flow {}

#[must_use]
impl ops::Add for Flow {
  type Output = Self;

  #[must_use]
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

#[must_use]
impl ops::Sub for Flow {
  type Output = Self;

  #[must_use]
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

#[must_use]
impl ops::AddAssign for Flow {
  fn add_assign(&mut self, other: Self) {
    *self = *self + other;
  }
}

#[must_use]
impl ops::SubAssign for Flow {
  fn sub_assign(&mut self, other: Self) {
    *self = *self - other;
  }
}
