use super::ansi::*;
use super::data::*;

pub fn render_header(tick: u64, width: usize) -> String {
    let title = " GHOSTMESH :: P2P MESH CONSOLE ";
    let tick_str = format!("[TICK {:06}]", tick);
    let rem = width.saturating_sub(title.len() + tick_str.len() + 4);
    let border = "═".repeat(rem);
    format!(
        "{}{}{}{}{}{}{}{}\n",
        BRIGHT_GREEN, BOLD, title, CYAN, border, YELLOW, tick_str, RESET
    )
}

pub fn render_node_topology(nodes: &[MonitorNode], width: usize) -> String {
    let mut out = String::new();
    let border = "─".repeat(width.saturating_sub(35));
    out.push_str(&format!(
        "{}{}┌── ACTIVE MESH TOPOLOGY ({} NODES) {}\n",
        BRIGHT_CYAN,
        BOLD,
        nodes.len(),
        border
    ));
    for (i, node) in nodes.iter().enumerate() {
        let (badge, color) = node.status.badge();
        let branch = if i + 1 == nodes.len() { "└──" } else { "├──" };
        out.push_str(&format!(
            "{}{} {}{:<10}{} [{}] addr:{:<15} v:{} links:{}\n",
            BRIGHT_CYAN,
            branch,
            BRIGHT_GREEN,
            node.name,
            color,
            badge,
            node.addr,
            node.crdt_version,
            node.active_links
        ));
    }
    out.push_str(RESET);
    out
}

pub fn render_latency_matrix(
    nodes: &[MonitorNode],
    matrix: &[Vec<Option<u32>>],
    width: usize,
) -> String {
    let mut out = String::new();
    let border = "─".repeat(width.saturating_sub(28));
    out.push_str(&format!(
        "{}{}┌── RTT LATENCY MATRIX {}\n",
        BRIGHT_CYAN, BOLD, border
    ));
    out.push_str("│ FROM\\TO    ");
    for n in nodes {
        out.push_str(&format!("{:<8} ", n.name));
    }
    out.push('\n');

    for (i, n) in nodes.iter().enumerate() {
        out.push_str(&format!("│ {:<10} ", n.name));
        for (j, _) in nodes.iter().enumerate() {
            if i == j {
                out.push_str(&format!("{}-       {}", DIM, RESET));
            } else if let Some(Some(ms)) = matrix.get(i).map(|row| row.get(j)).flatten() {
                let color = if *ms < 10 {
                    GREEN
                } else if *ms < 50 {
                    YELLOW
                } else {
                    RED
                };
                out.push_str(&format!("{}{:<4}ms  {}", color, ms, RESET));
            } else {
                out.push_str(&format!("{}--      {}", RED, RESET));
            }
        }
        out.push('\n');
    }
    out
}
