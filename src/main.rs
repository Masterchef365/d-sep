use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

/// Nodes are characters
type Node = char;

/// A single edge from a graph
#[derive(Debug)]
pub struct Edge {
    /// True if this edge points toward `end`
    pub toward: bool,
    /// End of the edge
    pub end: Node,
}

/// Map from Tail to End(s), adjacency list
pub type Graph = HashMap<Node, Vec<Edge>>;

fn main() -> Result<()> {
    // Parse args
    let mut args = std::env::args().skip(1);
    let graph_path = args.next().context("Requires graph path")?;

    let start_node: Node = args
        .next()
        .context("Requires first query")?
        .chars()
        .next()
        .unwrap();
    let end_node: Node = args
        .next()
        .context("Requires second query")?
        .chars()
        .next()
        .unwrap();
    let evidence: Vec<Node> = args.map(|s| s.chars().next().unwrap()).collect();

    // Read file
    let string = std::fs::read_to_string(graph_path)?;
    let graph = read_graph(&string)?;

    // Calculate
    let d_separated = d_separated(&graph, start_node, end_node, &evidence)?;

    // Output
    print!("{} and {} are ", start_node, end_node);
    if d_separated {
        println!("separated");
    } else {
        println!("not separated");
    }
    if !evidence.is_empty() {
        println!("Given evidence {:?}", evidence);
    }

    Ok(())
}

/// Read a graph from the specified string, in the form 'A -> B', line by line
pub fn read_graph(s: &str) -> Result<Graph> {
    let mut graph: Graph = HashMap::new();

    for (line_idx, line) in s.lines().enumerate() {
        // Ignore whitespace before and after
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split(" -> ");
        let mut parse_part = |part: &str| -> Result<Node> {
            let missing = || format!("Line {}: Missing {}", line_idx + 1, part);
            parts
                .next()
                .with_context(missing)?
                .chars()
                .next()
                .with_context(missing)
        };

        let tail = parse_part("tail")?;
        let head = parse_part("head")?;

        graph.entry(tail).or_default().push(Edge {
            toward: true,
            end: head,
        });

        graph.entry(head).or_default().push(Edge {
            toward: false,
            end: tail,
        });
    }

    Ok(graph)
}

/// Return Ok(true) if the paths are d-separated, Ok(false) otherwise. Can error if the start/end
/// nodes not in the graph
pub fn d_separated(
    graph: &Graph,
    start_node: Node,
    end_node: Node,
    evidence: &[Node],
) -> Result<bool> {
    // Nodes we have visited, and the directions we have visited them from. 
    let mut visited: HashSet<(Option<bool>, Node)> = HashSet::new();
    let mut lifo: Vec<(Option<bool>, Node)> = vec![(None, start_node)];
    let evidence_set: HashSet<Node> = evidence.iter().copied().collect();

    while let Some((last_was_toward, node)) = lifo.pop() {
        if visited.contains(&(last_was_toward, node)) {
            continue;
        }
        visited.insert((last_was_toward, node));
        println!("Visit {}", node);

        if node == end_node {
            return Ok(false);
        }

        let in_evidence = evidence_set.contains(&node);
        let descendants_in_ev = graph.get(&node).unwrap().iter().any(|n| n.toward && evidence.contains(&n.end));

        let adjacent = graph.get(&node).context("Node not in graph")?;
        for edge in adjacent {
            let blocked = match last_was_toward {
                None => false,
                Some(last_was_toward) => is_blocked(last_was_toward, in_evidence, edge.toward, descendants_in_ev),
            };
            println!("{} -> {}: blocked: {}, last: {:?}, cur: {}", node, edge.end, blocked, last_was_toward, edge.toward);

            if !blocked {
                lifo.push((Some(edge.toward), edge.end));
            }
        }
    }

    Ok(true)
}

/// Returns true if the situation calls for blockage
fn is_blocked(last_was_toward: bool, in_evidence: bool, next_is_toward: bool, descendants_in_ev: bool) -> bool {
    match (last_was_toward, in_evidence, next_is_toward, descendants_in_ev) {
        (false, true, true, _) => true,
        (true, true, true, _) => true,
        (false, true, false, _) => true,
        (true, false, false, false) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_assn_graph() {
        let graph = get_graph();
        assert!(!both_ways(&graph, 'A', 'B', &[]).unwrap());
        assert!(!both_ways(&graph, 'A', 'E', &[]).unwrap());
        assert!(both_ways(&graph, 'A', 'E', &['B']).unwrap());
        assert!(!both_ways(&graph, 'A', 'C', &[]).unwrap());
        assert!(!both_ways(&graph, 'A', 'F', &[]).unwrap());
        assert!(both_ways(&graph, 'A', 'F', &['C']).unwrap());
        assert!(!both_ways(&graph, 'A', 'G', &['C']).unwrap());
        assert!(both_ways(&graph, 'A', 'G', &['C', 'B']).unwrap());
        assert!(!both_ways(&graph, 'D', 'H', &['C', 'B']).unwrap());
        assert!(both_ways(&graph, 'D', 'H', &['C', 'B', 'F']).unwrap());
        assert!(!both_ways(&graph, 'C', 'G', &[]).unwrap());
        assert!(!both_ways(&graph, 'C', 'G', &['F']).unwrap());
        assert!(both_ways(&graph, 'C', 'G', &['F', 'A']).unwrap());
    }

    /// Test d-separation with the start and end nodes reversed to ensure the result is the same
    /// both ways. 
    fn both_ways(
        graph: &Graph,
        a: Node,
        b: Node,
        evidence: &[Node],
    ) -> Result<bool> {
        let ab = d_separated(graph, a, b, evidence)?;
        let ba = d_separated(graph, b, a, evidence)?;
        assert!(ab == ba, "Gives different results in opposite directions!");
        Ok(ab)
    }

    /// Get the test graph
    fn get_graph() -> Graph {
        let string = "
            A -> B
            A -> C
            C -> F
            D -> F
            B -> E
            E -> G
            F -> G
            F -> H
        ";
        read_graph(string).unwrap()
    }
}
