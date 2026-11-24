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

pub fn render_transfer_bars(
    transfers: &[TransferProgress],
    width: usize,
    tick: u64,
) -> String {
    let mut out = String::new();
    let border = "─".repeat(width.saturating_sub(35));
    out.push_str(&format!(
        "{}{}┌── ENCRYPTED CHUNK TRANSFERS {}\n",
        BRIGHT_CYAN, BOLD, border
    ));
    if transfers.is_empty() {
        out.push_str("│ [IDLE] No active chunk transfers\n");
        return out;
    }

    for t in transfers {
        let p = t.percent();
        let bar_len = 24;
        let filled = (bar_len * p as usize) / 100;
        let anim_char = match (tick + t.completed_chunks as u64) % 4 {
            0 => "▰",
            1 => "▱",
            2 => "▓",
            _ => "▒",
        };
        let bar: String = (0..bar_len)
            .map(|i| if i < filled { "█" } else if i == filled && p < 100 { anim_char } else { "░" })
            .collect();
        out.push_str(&format!(
            "│ {:<12} [{}{}{}] {:>3}% ({}/{} chunks) hash:{}\n",
            t.file_name,
            BRIGHT_GREEN,
            bar,
            RESET,
            p,
            t.completed_chunks,
            t.total_chunks,
            t.root_hash_prefix
        ));
    }
    out
}
