pub const CLEAR_SCREEN: &str = "\x1B[2J\x1B[H";
pub const HIDE_CURSOR: &str = "\x1B[?25l";
pub const SHOW_CURSOR: &str = "\x1B[?25h";
pub const RESET: &str = "\x1B[0m";
pub const BOLD: &str = "\x1B[1m";
pub const DIM: &str = "\x1B[2m";
pub const GREEN: &str = "\x1B[32m";
pub const BRIGHT_GREEN: &str = "\x1B[92m";
pub const CYAN: &str = "\x1B[36m";
pub const BRIGHT_CYAN: &str = "\x1B[96m";
pub const YELLOW: &str = "\x1B[33m";
pub const RED: &str = "\x1B[31m";
pub const BG_BLACK: &str = "\x1B[40m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermSize {
    pub cols: u16,
    pub rows: u16,
}

impl TermSize {
    pub fn fallback() -> Self {
        Self { cols: 80, rows: 24 }
    }
}
