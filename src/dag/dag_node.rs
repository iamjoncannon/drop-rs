use std::sync::mpsc::Sender;

use crate::{
    dag_types::{DagNodeResult, DeadlineInUnixMs, NodeId, NodeState},
    util::sleep,
};

#[derive(Debug,Clone)]
pub struct Call {
    pub node_id: NodeId,
    pub simulated_processing_time: u64,
    pub simulated_next_state: NodeState,
    pub simulated_result: String,
}

#[derive(Debug)]
pub struct DagNode {
    pub node_id: NodeId,
    pub node_state: NodeState,
    pub time_out: u128,
    pub depends_on: Vec<NodeId>,
    pub call: Call,
    
    pub deadline: Option<DeadlineInUnixMs>,
    pub result: Option<String>,

}

impl DagNode {
    pub fn run(call: Call, tx: Sender<DagNodeResult>) {
        sleep(call.simulated_processing_time);

        let result = DagNodeResult {
            node_id: call.node_id,
            next_state: call.simulated_next_state,
            payload: call.simulated_result.to_string(),
        };

        tx.send(result);
    }
}
