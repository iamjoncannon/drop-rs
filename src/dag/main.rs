/// 
/// this module is a proof of concept for a directed acyclic graph, 
/// for use with an future end to end testing feature 
/// 
// #![allow(warnings)]

use std::sync::{mpsc::channel, Arc, Mutex};

use dag::Dag;
use dag_node::DagNode;
use dag_types::DagNodeResult;

pub mod dag;
pub mod dag_init;
pub mod dag_node;
pub mod dag_types;
pub mod util;

fn main() {

}

pub fn run_dag(dag_nodes: Vec::<DagNode>){
    let (tx, rx) = channel::<DagNodeResult>();

    let nodes = Arc::new(Mutex::new(dag_nodes));

    let dag = Dag::new(tx.clone(), nodes.clone());

    dag.node_result_listener(rx, nodes);

    loop {
        let nodes_that_need_processing = dag.determine_processing_state();

        dag.process_nodes(nodes_that_need_processing);

        dag.process_deadline_expiration();

        if dag.is_complete() {
            break;
        }
    }
}


#[cfg(test)]
mod tests {

    #[test]
    pub fn test_1() {

    }
}
