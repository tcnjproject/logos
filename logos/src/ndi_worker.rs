// TCNJ AI/ML Group
//
// Runs the NDI session (library load, frame rendering, frame send) on a
// dedicated background thread so the iced update/view cycle never blocks on
// FFI calls or per-frame CPU work. The GUI only ever touches a small shared
// status struct and a command channel — both effectively free to poll.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rhema_broadcast::ndi::{NdiRuntime, NdiStartRequest};

use crate::broadcast::FrameRenderer;

const SESSION_ID: &str = "live";
const FRAME_INTERVAL: Duration = Duration::from_millis(33);

enum NdiCommand {
    Start(NdiStartRequest),
    Stop,
    SetVerse { reference: String, text: String },
}

#[derive(Default, Clone)]
struct NdiStatus {
    active: bool,
    error: Option<String>,
}

/// GUI-side handle to the background NDI worker thread.
pub struct NdiHandle {
    tx: Sender<NdiCommand>,
    status: Arc<Mutex<NdiStatus>>,
}

impl NdiHandle {
    /// Spawn the worker thread and return a handle to it.
    pub fn spawn() -> Self {
        let status = Arc::new(Mutex::new(NdiStatus::default()));
        let (tx, rx) = mpsc::channel();

        let worker_status = status.clone();
        thread::Builder::new()
            .name("ndi-broadcast".into())
            .spawn(move || run(&rx, &worker_status))
            .expect("failed to spawn NDI worker thread");

        Self { tx, status }
    }

    /// Ask the worker to (re)start the NDI session. Returns immediately —
    /// the actual library load/init happens on the worker thread.
    pub fn start(&self, request: NdiStartRequest) {
        let _ = self.tx.send(NdiCommand::Start(request));
    }

    /// Ask the worker to stop the NDI session. Returns immediately.
    pub fn stop(&self) {
        let _ = self.tx.send(NdiCommand::Stop);
    }

    /// Update the verse text the worker renders into outgoing frames.
    pub fn set_verse(&self, reference: String, text: String) {
        let _ = self.tx.send(NdiCommand::SetVerse { reference, text });
    }

    /// Cheap poll of the worker's current status — a single mutex lock, no FFI.
    pub fn any_active(&self) -> bool {
        self.status.lock().unwrap().active
    }

    /// Cheap poll of the last session-level error (start/stop failures).
    pub fn error(&self) -> Option<String> {
        self.status.lock().unwrap().error.clone()
    }
}

fn run(rx: &Receiver<NdiCommand>, status: &Arc<Mutex<NdiStatus>>) {
    let mut runtime = NdiRuntime::default();
    let renderer = FrameRenderer::new();
    let mut reference = String::new();
    let mut verse_text = String::new();

    loop {
        match rx.recv_timeout(FRAME_INTERVAL) {
            Ok(cmd) => apply(cmd, &mut runtime, &mut reference, &mut verse_text, status),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        while let Ok(cmd) = rx.try_recv() {
            apply(cmd, &mut runtime, &mut reference, &mut verse_text, status);
        }

        if !runtime.any_active() {
            continue;
        }

        let Some(info) = runtime.current_info(SESSION_ID) else {
            continue;
        };

        let rgba = match &renderer {
            Some(r) => r.render_verse(&reference, &verse_text, info.width, info.height),
            None => {
                let mut frame = vec![0u8; (info.width * info.height * 4) as usize];
                for chunk in frame.chunks_exact_mut(4) {
                    chunk[3] = 255;
                }
                frame
            }
        };

        if let Err(e) = runtime.send_frame_rgba(SESSION_ID, info.width, info.height, &rgba) {
            eprintln!("NDI frame error: {e}");
        }
    }
}

fn apply(
    cmd: NdiCommand,
    runtime: &mut NdiRuntime,
    reference: &mut String,
    verse_text: &mut String,
    status: &Arc<Mutex<NdiStatus>>,
) {
    match cmd {
        NdiCommand::Start(request) => {
            let result = runtime.start(SESSION_ID.into(), request);
            let mut st = status.lock().unwrap();
            match result {
                Ok(_) => {
                    st.active = true;
                    st.error = None;
                }
                Err(e) => {
                    st.active = false;
                    st.error = Some(e.to_string());
                }
            }
        }
        NdiCommand::Stop => {
            runtime.stop(SESSION_ID);
            let mut st = status.lock().unwrap();
            st.active = false;
            st.error = None;
        }
        NdiCommand::SetVerse { reference: r, text } => {
            *reference = r;
            *verse_text = text;
        }
    }
}
