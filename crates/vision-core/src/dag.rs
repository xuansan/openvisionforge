use std::collections::{BTreeMap, VecDeque};

use crate::{Result, VisionError, Workflow};

pub fn topo_order(workflow: &Workflow) -> Result<Vec<String>> {
    let mut indegree = BTreeMap::new();
    let mut outgoing: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for node in &workflow.nodes {
        indegree.insert(node.id.clone(), 0usize);
    }

    for edge in &workflow.edges {
        if indegree.contains_key(&edge.from_node) && indegree.contains_key(&edge.to_node) {
            outgoing
                .entry(edge.from_node.clone())
                .or_default()
                .push(edge.to_node.clone());
            *indegree.entry(edge.to_node.clone()).or_default() += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(id, n)| (*n == 0).then_some(id.clone()))
        .collect::<VecDeque<_>>();
    let mut order = Vec::new();

    while let Some(id) = ready.pop_front() {
        order.push(id.clone());
        for next in outgoing.get(&id).into_iter().flatten() {
            let count = indegree.get_mut(next).expect("node exists");
            *count -= 1;
            if *count == 0 {
                ready.push_back(next.clone());
            }
        }
    }

    if order.len() == workflow.nodes.len() {
        Ok(order)
    } else {
        Err(VisionError::Cycle)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Workflow, WorkflowEdge, WorkflowNode};

    use super::topo_order;

    #[test]
    fn sorts_linear_workflow() {
        let workflow = Workflow {
            id: "w".to_string(),
            name: "w".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![node("a"), node("b")],
            edges: vec![WorkflowEdge {
                from_node: "a".to_string(),
                from_port: "out".to_string(),
                to_node: "b".to_string(),
                to_port: "in".to_string(),
            }],
        };

        assert_eq!(topo_order(&workflow).unwrap(), vec!["a", "b"]);
    }

    fn node(id: &str) -> WorkflowNode {
        WorkflowNode {
            id: id.to_string(),
            operator: "mock".to_string(),
            params: serde_json::Value::Null,
            execution: Default::default(),
        }
    }
}
