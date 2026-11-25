pub mod ansi;
pub mod data;
pub mod render;

use ansi::{TermSize, CLEAR_SCREEN, HIDE_CURSOR, SHOW_CURSOR};
use data::{advance_simulation, mock_initial_state, MonitorState};
use ghostmesh_core::identity::PeerIdentity;
use ghostmesh_net::node::MeshNode;
use rand_core::OsRng;
use render::render_frame;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

struct CursorGuard;

impl Drop for CursorGuard {
    fn drop(&mut self) {
        let mut out = io::stdout().lock();
        let _ = write!(out, "{}", SHOW_CURSOR);
        let _ = out.flush();
    }
}

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

pub fn run_live_monitor(max_iterations: Option<u64>) {
    let identity = PeerIdentity::generate(&mut OsRng);
    let node = MeshNode::bind(identity, "127.0.0.1:0").expect("bind live mesh node");
    let start = Instant::now();
    let _guard = CursorGuard;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}", HIDE_CURSOR);
    let _ = handle.flush();

    let mut count = 0u64;
    loop {
        let size = TermSize::current();
        let state = MonitorState::from_live_node(&node, count, start.elapsed().as_secs());
        let frame = render_frame(&state, size.cols as usize);
        let _ = write!(handle, "{}{}", CLEAR_SCREEN, frame);
        let _ = handle.flush();

        count += 1;
        if max_iterations.is_some_and(|max| count >= max) {
            break;
        }
        sleep(Duration::from_millis(250));
    }
}
