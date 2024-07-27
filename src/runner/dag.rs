use crate::parser::block_type::chain::ChainNode;
use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

pub struct Dag {}

impl Dag {

    // generate dependencies for drop_runs

    pub fn topo_sort(
        chain_nodes: &mut Vec<ChainNode>,
        chain_drop_id: String,
    ) -> Result<HashMap<String, Vec<String>>, TopoSortError> {

        let mut adj_list = HashMap::<String, Vec<String>>::new();

        // on removal, push to new list to return
        let mut return_adj_list = HashMap::<String, Vec<String>>::new();

        let mut in_degree = HashMap::<String, i32>::new();

        let return_nodes: Vec<ChainNode> = chain_nodes
            .drain(..)
            .map(|mut chain_node| {
                let depends_on = chain_node.remove_depends_on();

                let node_id = chain_node.id();

                match &depends_on {
                    Some(depends_on) => {
                        for dep in depends_on {
                            in_degree
                                .entry(dep.to_string())
                                .and_modify(|cnt| *cnt += 1)
                                .or_insert(1);
                        }
                    }
                    None => {}
                }

                adj_list.insert(node_id.to_string(), depends_on.unwrap_or_default());

                chain_node
            })
            .collect();

        chain_nodes.extend(return_nodes);

        let initial_leaf_nodes = get_leaf_nodes(&adj_list);

        for initial_leaf_node in &initial_leaf_nodes {
            return_adj_list.insert(initial_leaf_node.to_string(), Vec::<String>::new());
        }

        let mut q = VecDeque::<String>::from(initial_leaf_nodes);

        while !q.is_empty() {
            let leaf_node = q.pop_front().unwrap();

            for (other_node, remaining_deps) in &mut adj_list {
                let contained = remaining_deps.contains(&leaf_node);

                if contained {
                    // push this dependency to return list
                    return_adj_list
                        .entry(other_node.to_string())
                        .and_modify(|cur_list| {
                            cur_list.push(leaf_node.to_string());
                        })
                        .or_insert(Vec::<String>::from([leaf_node.to_string()]));

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
                    chain_drop_id,
                });
            }
        }

        Ok(return_adj_list)
    }

    pub fn get_leaf_nodes(&self, adj_list: &HashMap<String, Vec<String>>) -> Vec<String> {
        let mut leaf_nodes = Vec::<String>::new();
    
        for (k, v) in adj_list {
            if v.is_empty() {
                leaf_nodes.push(k.to_string());
            }
        }
    
        leaf_nodes
    }
    
    pub fn remove_node_from_adj_list(&self, adj_list: &mut HashMap<String, Vec<String>>, leaf_node: &String) {
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
    node: String,
    chain_drop_id: String,
}

impl fmt::Display for TopoSortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error parsing chain-- dependency cycle detected\nchain: {}\nsee node: {}",
            self.chain_drop_id, self.node
        )
    }
}