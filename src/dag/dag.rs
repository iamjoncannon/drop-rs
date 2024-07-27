use std::{collections::HashMap, sync::{mpsc::Receiver, MutexGuard}, thread::spawn};

use crate::{
    dag_node::DagNode,
    dag_types::{CancellationReason, DagNodeResult, NodeId, NodeState},
    util::{current_time, sleep},
};
use core::time;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

pub struct Dag {
    pub tx: Sender<DagNodeResult>,
    pub starting_nodes: Vec<usize>,
    pub adjacency_list: HashMap<usize, Vec<usize>>,
    pub nodes: Arc<Mutex<Vec<DagNode>>>, // id corresponds to index in vector
}

impl Dag {


    pub fn node_result_listener(&self, rx: Receiver<DagNodeResult>, nodes: Arc<Mutex<Vec<DagNode>>>){
        spawn(move || {
            for dag_node_result in rx.recv() {
                let mut nodes_unlocked = nodes.lock().unwrap();
                let node = nodes_unlocked.get_mut(dag_node_result.node_id).unwrap();
                node.node_state = dag_node_result.next_state;
                node.result = Some(dag_node_result.payload);
            }
        });
    }

    pub fn determine_processing_state(&self) -> Vec<NodeId> {
        let mut node_that_need_processing = Vec::new();

        let unlocked_nodes = &self.nodes.lock().unwrap();

        for node in unlocked_nodes.iter() {
            let this_node_needs_processing = match node.node_state {
                NodeState::Pending => true,
                NodeState::Success(is_processed) => !is_processed,
                NodeState::TimedOut(is_processed) => !is_processed,
                NodeState::Failed(is_processed) => !is_processed,
                _ => false,
            };

            if this_node_needs_processing {
                node_that_need_processing.push(node.node_id);
            }
        }

        node_that_need_processing
    }

    pub fn process_nodes(&self, nodes_that_need_processing: Vec<NodeId>) {

        if nodes_that_need_processing.is_empty() {
            return;
        }

        let mut unlocked_nodes = self.nodes.lock().unwrap();

        let mut deps_to_start: Vec<usize> = Vec::new();
        let mut deps_to_cancel: Vec<usize> = Vec::new();

        for node_id in nodes_that_need_processing {
            let node = unlocked_nodes.get_mut(node_id).unwrap();

            match node.node_state {
                NodeState::Pending => self.start_node_job(node),
                NodeState::Success(_) => {
                    let dep_ids = self.adjacency_list.get(&node.node_id).unwrap();
                    deps_to_start.extend(dep_ids);
                    node.node_state = NodeState::Success(true);
                }
                NodeState::TimedOut(_) => {
                    let dep_ids = self.adjacency_list.get(&node.node_id).unwrap();
                    deps_to_cancel.extend(dep_ids);
                    node.node_state = NodeState::TimedOut(true);
                }
                NodeState::Failed(_) => {
                    let dep_ids = self.adjacency_list.get(&node.node_id).unwrap();
                    deps_to_cancel.extend(dep_ids);
                    node.node_state = NodeState::Failed(true);
                }
                _ => {}
            }
        }

        for dep_id in deps_to_start {
            let dep = unlocked_nodes.get_mut(dep_id).unwrap();
            self.start_node_job(dep);
        }

        for dep_id in deps_to_cancel {
            let dep = unlocked_nodes.get_mut(dep_id).unwrap();
            dep.node_state = NodeState::Cancelled(CancellationReason::DependencyFailure);
        }
    }

    pub fn process_success(
        &self,
        unlocked_nodes: &mut MutexGuard<Vec<DagNode>>,
        node: &mut DagNode,
    ) {
    }

    pub fn process_cancellation(
        &self,
        mut unlocked_nodes: MutexGuard<Vec<DagNode>>,
        node: &mut DagNode,
        cancellation_reason: CancellationReason,
    ) {
        let dep_ids = self.adjacency_list.get(&node.node_id).unwrap();

        dep_ids.iter().for_each(|dep_id| {
            let dep = unlocked_nodes.get_mut(*dep_id).unwrap();
            dep.node_state = NodeState::Cancelled(cancellation_reason);
        });
    }

    pub fn start_node_job(&self, node: &mut DagNode) {

        node.node_state = NodeState::Running;
        node.deadline = Some(current_time() + node.time_out);
        let tx_for_node = self.tx.clone();

        let call = node.call.clone();

        spawn(move ||{
            DagNode::run(call, tx_for_node);
        });

    }

    pub fn process_deadline_expiration(&self){
        let unlocked_nodes = &mut self.nodes.lock().unwrap();

        for node in unlocked_nodes.iter_mut() {

            if node.deadline.is_some() {
                
                
                if node.deadline.unwrap() < current_time() {
                    
                    node.node_state = NodeState::TimedOut(false);
                }
            }
        }
    }

    pub fn is_complete(&self) -> bool {

        let mut is_complete = true; 
        let unlocked_nodes = &self.nodes.lock().unwrap();

        for node in unlocked_nodes.iter() {

            let mut evaluate = |is_pending| {
                if is_pending {
                    is_complete = false;
                }
            };

            match node.node_state {
                NodeState::Pending => {
                    is_complete = false;
                },
                NodeState::Running => {
                    is_complete = false;
                },
                NodeState::Success(is_pending) => evaluate(is_pending),
                NodeState::TimedOut(is_pending) => evaluate(is_pending),
                NodeState::Failed(is_pending) => evaluate(is_pending),
                NodeState::Cancelled(is_pending) => {},
            }
        }

        is_complete
    }
}
