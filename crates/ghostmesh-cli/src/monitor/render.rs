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
