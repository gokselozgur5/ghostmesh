pub mod ansi;
pub mod data;
pub mod render;

use ansi::{TermSize, CLEAR_SCREEN, HIDE_CURSOR, SHOW_CURSOR};
use data::{advance_simulation, mock_initial_state};
use render::render_frame;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

pub fn run_monitor(max_iterations: Option<u64>) {
    let mut state = mock_initial_state();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}", HIDE_CURSOR);
    let _ = handle.flush();

    let mut count = 0;
    loop {
        let size = TermSize::current();
        let frame = render_frame(&state, size.cols as usize);
        let _ = write!(handle, "{}{}", CLEAR_SCREEN, frame);
        let _ = handle.flush();

        count += 1;
        if let Some(max) = max_iterations {
            if count >= max {
                break;
            }
        }

        sleep(Duration::from_millis(250));
        advance_simulation(&mut state);
    }
    let _ = write!(handle, "{}", SHOW_CURSOR);
    let _ = handle.flush();
}
