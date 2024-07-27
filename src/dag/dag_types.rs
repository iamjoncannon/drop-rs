use std::collections::HashMap;

pub type NodeId = usize;
pub type DeadlineInUnixMs = u128;

#[derive(Debug, Clone, Copy)]
pub enum CancellationReason {
    SigTerm,
    DependencyFailure,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeState {
    Pending, // not able to run due to dependencies
    Running, // job not yet completed

    Success(bool),  // successful, pending processing dependencies
    TimedOut(bool), // timedout, pending processing dependencies
    Failed(bool),   // failed, pending processing dependencies

    Cancelled(CancellationReason)
}

pub struct DagNodeResult {
    pub node_id: NodeId,
    pub payload: String,
    pub next_state: NodeState,
}