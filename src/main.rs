use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;

type Node = char;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let graph_path = args.next().context("Requires graph path")?;
    let graph: Graph = read_graph(graph_path)?;

    let start_node: Node = args.next().context("Requires first query")?.chars().next().unwrap();
    let end_node: Node = args.next().context("Requires second query")?.chars().next().unwrap();
    let evidence: Vec<Node> = args.map(|s| s.chars().next().unwrap()).collect();

    let d_separated = d_separated(&graph, start_node, end_node, &evidence)?;

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

pub fn read_graph(path: impl AsRef<Path>) -> Result<Graph> {
    let string = std::fs::read_to_string(path)?;
    let mut graph: Graph = HashMap::new();

    for (line_idx, line) in string.lines().enumerate() {
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

/// Map from Tail to End(s)
pub type Graph = HashMap<Node, Vec<Edge>>;

/// A single edge from a graph
#[derive(Debug)]
pub struct Edge {
    /// True if this edge points toward `end`
    pub toward: bool,
    /// End of the edge
    pub end: Node,
}

pub fn d_separated(graph: &Graph, start_node: Node, end_node: Node, evidence: &[Node]) -> Result<bool> {
    let mut visited: HashSet<Node> = HashSet::new();
    let mut lifo: Vec<(Option<bool>, Node)> = vec![(None, start_node)];
    let evidence_set: HashSet<Node> = evidence.iter().copied().collect();

    while let Some((last_was_toward, node)) = lifo.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);

        let adjacent = graph.get(&node).context("Node not in graph")?;
        for edge in adjacent {
            let in_evidence = evidence_set.contains(&node);
            let blocked = match last_was_toward {
                None => false,
                Some(last_was_toward) => blocked(last_was_toward, in_evidence, edge.toward),
            };

            if !blocked {
                if edge.end == end_node {
                    return Ok(false);
                }
                lifo.push((Some(edge.toward), edge.end));
            }
        }
    }
    
    Ok(true)
}

fn blocked(last_was_toward: bool, in_evidence: bool, next_is_toward: bool) -> bool {
    match (last_was_toward, in_evidence, next_is_toward) {
        (false, true, true) => true,
        (true, true, true) => true,
        (false, true, false) => true,
        (true, false, false) => true,
        _ => false,
    }
}
