use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

type Node = char;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let graph_path = args.next().context("Requires graph path")?;
    let graph = read_graph(graph_path)?;
    dbg!(&graph);

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
