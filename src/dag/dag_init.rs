use std::{
    collections::{HashMap, VecDeque},
    fmt,
    sync::{mpsc::Sender, Arc, Mutex},
};

use crate::{
    dag::Dag,
    dag_node::DagNode,
    dag_types::{DagNodeResult, NodeId},
};

impl Dag {
    pub fn new(tx: Sender<DagNodeResult>, nodes: Arc<Mutex<Vec<DagNode>>>) -> Dag {
        // topo sort

        // if cycle, panic

        Dag {
            tx,
            nodes,
            starting_nodes: Vec::new(),
            adjacency_list: HashMap::new(),
        }
    }

    pub fn topo_sort(
        &self,
        dag_nodes: &mut Vec<DagNode>,
        chain_drop_id: String,
    ) -> Result<HashMap<NodeId, Vec<NodeId>>, TopoSortError> {
        let mut adj_list = HashMap::<NodeId, Vec<NodeId>>::new();

        // on removal, push to new list to return
        let mut return_adj_list = adj_list.clone(); 

        let mut in_degree = HashMap::<NodeId, i32>::new();

        dag_nodes.iter_mut().for_each(|mut dag_node| {
            let depends_on = &dag_node.depends_on;
            let node_id = dag_node.node_id;
            adj_list.insert(node_id, depends_on.clone());
        });

        let initial_leaf_nodes = self.get_leaf_nodes(&adj_list);

        let mut q = VecDeque::<NodeId>::from(initial_leaf_nodes);

        while !q.is_empty() {
            let leaf_node_id = q.pop_front().unwrap();

            for (other_node, remaining_deps) in &mut adj_list {

                let contained = remaining_deps.contains(&leaf_node);

                if contained {
                    // push this dependency to return list
                    return_adj_list
                        .entry(*other_node)
                        .and_modify(|cur_list| {
                            cur_list.push(leaf_node);
                        })
                        .or_insert(Vec::<NodeId>::from([leaf_node]));

                    // remove from adj list
                    let idx_of_target =
                        remaining_deps.iter().position(|n| *n == leaf_node).unwrap();

                    remaining_deps.remove(idx_of_target);

                    // decrement in degree
                    in_degree
                        .entry(leaf_node.clone())
                        .and_modify(|cnt| *cnt -= 1);
                }
            }
        }

        for (node, remaining_in_degrees) in in_degree {
            if remaining_in_degrees > 0 {
                return Err(TopoSortError {
                    node,
                    // chain_drop_id,
                });
            }
        }

        Ok(return_adj_list)
    }

    pub fn get_leaf_nodes(&self, adj_list: &HashMap<NodeId, Vec<NodeId>>) -> Vec<NodeId> {
        let mut leaf_nodes = Vec::<NodeId>::new();

        for (k, v) in adj_list {
            if v.is_empty() {
                leaf_nodes.push(*k);
            }
        }

        leaf_nodes
    }

    pub fn remove_node_from_adj_list(
        &self,
        adj_list: &mut HashMap<NodeId, Vec<NodeId>>,
        leaf_node: &NodeId,
    ) {
        for remaining_deps in adj_list.values_mut() {
            let contained = remaining_deps.contains(leaf_node);

            if contained {
                let idx_of_target = remaining_deps
                    .iter()
                    .position(|n| *n == *leaf_node)
                    .unwrap();
                remaining_deps.remove(idx_of_target);
            }
        }
    }
}

#[derive(Debug)]
pub struct TopoSortError {
    node: NodeId,
    // chain_drop_id: String,
}

impl fmt::Display for TopoSortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error parsing chain-- dependency cycle detected\n\nsee node: {}",
            // "Error parsing chain-- dependency cycle detected\nchain: {}\nsee node: {}",
            self.node
        )
    }
}
