use disposition_model_common::Map;

use crate::TaffyNodeKind;

/// Map from each `taffy` node ID to its structural [`TaffyNodeKind`].
///
/// Only holds the wrapper / container nodes that have no diagram node ID or
/// [`TaffyNodeCtx`](crate::TaffyNodeCtx); used to label them when printing the
/// taffy tree.
pub type TaffyNodeToKind<'id> = Map<taffy::NodeId, TaffyNodeKind<'id>>;
