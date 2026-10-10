// TCNJ AI/ML Group

use iced::{
    Task, Element, Subscription, Theme,
    time,
    widget::pane_grid,
};
use iced::futures::stream::{self};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rhema_broadcast::ndi::{
    NdiStartRequest, NdiResolution, NdiFrameRate, NdiAlphaMode,
};

use crate::audio::{AudioCapture, AudioShared};
use crate::data::*;
use crate::ndi_worker::NdiHandle;
use crate::views;
use crate::speech::SpeechSession;
use crate::scripture::{SqliteVerseRepository, VerseRepository};
use crate::detection::{BibleReferenceDetector, ReferenceDetector};
use rhema_stt::TranscriptEvent;

// Some variants are handled in `update()` but not yet emitted by any view —
// they're wired up ahead of features still in progress (real STT streaming,
// dismissible tour/update banner, pane-grid drag/resize).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    // Loading
    Tick,
    LoadingComplete,

    // Transcription
    ToggleTranscription,
    TranscriptUpdated(String),
    SpeechEvent(u64, TranscriptEvent),
    AudioCaptureFailed(u64, String),
    BibleLoaded(Result<Arc<dyn VerseRepository>, String>),
    SearchCompleted(u64, Result<Vec<Verse>, String>),
    AddToDetections { session: u64, translation: String, result: Result<Vec<Verse>, String> },

    // Audio level - fired by subscription ~30fps when recording
    AudioFrame {
        /// Downsampled waveform for display (fixed length = DISPLAY_BARS)
        waveform: Vec<f32>,
        /// Smoothed RMS level 0..1
        rms: f32,
        /// Peak level 0..1
        peak: f32,
    },

    // Search
    SearchQueryChanged(String),
    SearchModeChanged(SearchMode),
    TranslationChanged(Translation),
    TranslationDropdownToggled,
    SearchSubmitted,

    // Queue
    AddToQueue(Verse),
    RemoveFromQueue(usize),
    PresentVerse(Verse),
    ClearQueue,

    // Live
    GoLiveToggled,
    ClearLive,

    // Tour
    TourNext,
    TourBack,
    TourSkip,
    TourDismiss,

    // Update banner
    InstallUpdate,
    DismissUpdate,

    // Timer
    TimerTick,

    // Toolbar
    OpenSettings,
    // OpenBroadcast,
    // OpenDisplay,
    // OpenCaptions,

    // Menubar
    // ToggleHelpMenu,
    OpenAboutLogos,

    // Pane grid
    PaneResized(pane_grid::ResizeEvent),
    PaneDragged(pane_grid::DragEvent),
    ResetLayout,

    // Cheap poll of the NDI worker's status — just forces a repaint so the
    // badge/error label pick up state changes that happen asynchronously on
    // the worker thread (session start/stop, errors).
    NdiStatusTick,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneKind {
    LiveTranscript,
    Search,
    Queue,
    RecentDetections,
    ProgramPreview,
    LiveDisplay,
}

/// Number of bars shown in the waveform visualiser.
pub const DISPLAY_BARS: usize = 40;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LATEST_VERSION: &str = "0.1.0";

pub struct Logos {
    pub loading: LoadingState,
    pub loading_start: Instant,

    // Transcript
    pub is_transcribing: bool,
    pub transcript_text: String,
    pub speech: Option<SpeechSession>,
    pub speech_status: String,
    pub scripture: Option<Arc<dyn VerseRepository>>,
    pub bible_error: Option<String>,
    pub search_error: Option<String>,
    pub search_pending: bool,
    search_revision: u64,
    detector: Arc<dyn ReferenceDetector>,
    // pub audio_level: f32,
    
    // Live audio capture handle — Some(...) while mic is open, None when stopped.
    pub audio_capture: Option<AudioCapture>,
    // Shared audio state (written by cpal thread, read by subscription)
    pub audio_shared: Option<Arc<Mutex<AudioShared>>>,
 
    // Visualiser state (updated from AudioFrame messages)
    pub waveform: Vec<f32>,   // DISPLAY_BARS amplitudes in [0, 1]
    pub audio_rms: f32,
    pub audio_peak: f32,

    // Search
    pub search_query: String,
    pub search_mode: SearchMode,
    pub translation: Translation,
    pub translation_dropdown_open: bool,
    pub search_results: Vec<Verse>,

    // Preview / Live
    pub preview_verse: Option<Verse>,
    pub live_verse: Option<Verse>,
    pub go_live: bool,

    // Queue
    pub queue: Vec<Verse>,

    // Recent detections
    pub recent_detections: Vec<Verse>,

    // Tour
    pub tour_step: Option<TourStep>,
    pub show_update_banner: bool,

    // Menubar
    pub help_menu_open: bool,

    // Timer (remaining time in seconds)
    pub remaining_seconds: u32,

    // Pane grid layout
    pub pane_grid_state: pane_grid::State<PaneKind>,

    // Handle to the background NDI worker thread (owns the session, the
    // frame renderer, and all NDI FFI calls — see crate::ndi_worker).
    pub ndi: NdiHandle,
    // pub last_timer_tick: Instant,
}

fn tour_marker_path() -> PathBuf {
    // let dir =std::env::temp_dir();
    // println!("Temporary directory: {}", dir.display());
    std::env::temp_dir().join("logos-tour-seen.marker")

}

fn should_show_tour(marker_path: &Path) -> bool {
    !marker_path.exists()
}

fn mark_tour_seen(marker_path: &Path) {
    if let Some(parent) = marker_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(marker_path, "seen");
}

impl Logos {
    pub fn new() -> (Self, Task<Message>) {
        let marker_path = tour_marker_path();
        let should_show = should_show_tour(&marker_path);
        // let update_available = check_for_update();

        if should_show {
            mark_tour_seen(&marker_path);
        }

        // go_live starts as true — kick off the NDI session on the worker
        // thread immediately at launch. spawn()/start() return instantly;
        // the library load and NDI init happen off the GUI thread.
        let ndi = NdiHandle::spawn();
        ndi.start(NdiStartRequest {
            source_name: "Logos Bible Display".into(),
            resolution: NdiResolution::R1080p,
            frame_rate: NdiFrameRate::Fps30,
            alpha_mode: NdiAlphaMode::NoneOpaque,
        });

        (
            Self {
                loading: LoadingState::Loading(0.0),
                loading_start: Instant::now(),
                is_transcribing: false,
                transcript_text: String::new(),
                speech: None,
                speech_status: "Ready for offline transcription".into(),
                scripture: None,
                bible_error: None,
                search_error: None,
                search_pending: false,
                search_revision: 0,
                detector: Arc::new(BibleReferenceDetector),
                audio_capture: None,
                audio_shared: None,
                waveform: vec![0.0; DISPLAY_BARS],
                audio_rms: 0.0,
                audio_peak: 0.0,
                search_query: String::new(),
                search_mode: SearchMode::Book,
                // Previous default: translation: Translation::Esv,
                translation: Translation::Kjv,
                translation_dropdown_open: false,
                search_results: Vec::new(),
                preview_verse: None,
                live_verse: None,
                go_live: true,
                queue: Vec::new(),
                recent_detections: Vec::new(),
                tour_step: if should_show {
                    Some(TourStep::LiveTranscript)
                } else {
                    None
                },
                show_update_banner: check_for_update(),
                help_menu_open: false,
                remaining_seconds: 3600,
                pane_grid_state: pane_grid::State::with_configuration(default_pane_config()),
                ndi,
            },
            // Previous startup task: Task::none(),
            Task::perform(async {
                tokio::task::spawn_blocking(|| {
                    let path = std::env::var_os("LOGOS_BIBLE_DB").map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("data/rhema.db"));
                    SqliteVerseRepository::open(&path)
                }).await.map_err(|e| e.to_string()).and_then(|r| r)
            }, Message::BibleLoaded),
        )
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Push the current live verse text to the NDI worker thread so its
    /// next rendered frame reflects it. Cheap: just clones two short strings
    /// onto a channel, no rendering happens here.
    fn sync_ndi_verse(&self) {
        let (reference, text) = self
            .live_verse
            .as_ref()
            .map(|v| (v.reference.clone(), v.text.clone()))
            .unwrap_or_default();
        self.ndi.set_verse(reference, text);
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match &self.loading {
            LoadingState::Loading(_) => {
                time::every(Duration::from_millis(50)).map(|_| Message::Tick)
            }

            LoadingState::Ready => {
                let mut subs = vec![
                    time::every(Duration::from_secs(1)).map(|_| Message::TimerTick),
                ];

                if let Some(session) = &self.speech {
                    let id = session.id;
                    subs.push(Subscription::run_with_id(
                        ("speech", id),
                        stream::unfold(session.events.clone(), move |events| async move {
                            let event = events.lock().await.recv().await?;
                            Some((Message::SpeechEvent(id, event), events))
                        }),
                    ));
                }

                // Audio polling — active only while mic is open
                if let Some(shared) = &self.audio_shared {
                    let shared = shared.clone();
                    let session_id = self.speech.as_ref().map(|session| session.id).unwrap_or(0);
                    // Previous subscription identity: "audio_poll".
                    subs.push(Subscription::run_with_id(
                        ("audio_poll", session_id),
                        stream::unfold(shared, move |shared| async move {
                            tokio::time::sleep(Duration::from_millis(33)).await;
                            let (waveform, rms, peak) = {
                                let st = shared.lock().unwrap();
                                if let Some(error) = &st.error {
                                    let error = error.clone();
                                    drop(st);
                                    return Some((Message::AudioCaptureFailed(session_id, error), shared));
                                }
                                let waveform = downsample_waveform(&st.samples, DISPLAY_BARS);
                                (waveform, st.rms, st.peak)
                            };
                            Some((Message::AudioFrame { waveform, rms, peak }, shared))
                        }),
                    ));
                }

                // Frame rendering/sending happens on the NDI worker thread
                // (see crate::ndi_worker) — this just polls its status at a
                // low rate so the badge/error label stay current after a
                // Go Live toggle.
                if self.go_live {
                    subs.push(
                        time::every(Duration::from_millis(200)).map(|_| Message::NdiStatusTick),
                    );
                }

                Subscription::batch(subs)
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let elapsed = self.loading_start.elapsed().as_secs_f32();
                let progress = (elapsed / 2.5).min(1.0); // 2.5s loading
                if progress >= 1.0 {
                    self.loading = LoadingState::Ready;
                } else {
                    self.loading = LoadingState::Loading(progress);
                }
            }

            Message::TimerTick => {
                if self.remaining_seconds > 0 {
                    self.remaining_seconds -= 1;
                }
            }

            // Message::ToggleTranscription => {
            //     self.is_transcribing = !self.is_transcribing;
            //     if self.is_transcribing {
            //         self.transcript_text = String::new();
            //     }
            // }

             // ── Transcription / mic ───────────────────────────────────────────
//             Message::ToggleTranscription => {
//                 if self.is_transcribing {
//                     // Stop recording — drop the capture handle, which stops the stream
//                     self.audio_capture = None;
//                     self.audio_shared = None;
//                     self.is_transcribing = false;
//                     self.waveform = vec![0.0; DISPLAY_BARS];
//                     self.audio_rms = 0.0;
//                     self.audio_peak = 0.0;
//                 } else {
//                     // Start recording
//                     match AudioCapture::start() {
//                         Ok(capture) => {
//                             let shared = capture.shared.clone();
//                             self.audio_capture = Some(capture);
//                             self.audio_shared = Some(shared);
//                             self.is_transcribing = true;
//                             self.transcript_text = String::new();
//                         }
//                         Err(e) => {
//                             eprintln!("Mic error: {e}");
//                             // Still flip the UI so the user sees something
//                             self.is_transcribing = false;
//                         }
//                     }
//                 }
//             }
//
            Message::ToggleTranscription => {
                if self.is_transcribing {
                    self.stop_transcription();
                    self.speech_status = "Transcription stopped".into();
                } else {
                    match SpeechSession::local().and_then(|(session, sender)| {
                        AudioCapture::start_with_sender(Some(sender)).map(|capture| (session, capture))
                    }) {
                        Ok((session, capture)) => {
                            self.audio_shared = Some(capture.shared.clone());
                            self.audio_capture = Some(capture);
                            self.speech = Some(session);
                            self.is_transcribing = true;
                            self.transcript_text.clear();
                            self.speech_status = "Loading Whisper model…".into();
                        }
                        Err(error) => { self.speech_status = error; }
                    }
                }
            }

            // Audio frame from the polling subscription
            Message::AudioCaptureFailed(id, error) => {
                if self.speech.as_ref().map(|session| session.id) != Some(id) { return Task::none(); }
                self.stop_transcription();
                self.speech_status = error;
            }
            Message::AudioFrame { waveform, rms, peak } => {
                self.waveform = waveform;
                self.audio_rms = rms;
                self.audio_peak = peak;
            }
 
// Previous transcript-only update:
//             Message::TranscriptUpdated(text) => {
//                 self.transcript_text = text;
//             }
            Message::TranscriptUpdated(text) => {
                self.transcript_text = text;
                return self.detect_transcript();
            }
            Message::BibleLoaded(result) => {
                match result {
                    Ok(repository) => { self.scripture = Some(repository); self.bible_error = None; }
                    Err(error) => { self.bible_error = Some(error); }
                }
            }
            Message::SpeechEvent(id, event) => {
                if self.speech.as_ref().map(|s| s.id) != Some(id) { return Task::none(); }
                match event {
                    TranscriptEvent::Final { transcript, .. } if !transcript.trim().is_empty() => {
                        let mut text = self.transcript_text.clone();
                        if !text.is_empty() { text.push(' '); }
                        text.push_str(transcript.trim());
                        // Bound transcript memory while keeping enough context for split references.
                        if text.len() > 32_000 {
                            let start = text.char_indices().find(|(i, _)| *i >= text.len() - 32_000).map(|(i, _)| i).unwrap_or(0);
                            text = text[start..].to_string();
                        }
                        self.speech_status = "Listening (offline Whisper)".into();
                        return self.update(Message::TranscriptUpdated(text));
                    }
                    TranscriptEvent::Error(error) => { self.stop_transcription(); self.speech_status = error; }
                    TranscriptEvent::Disconnected => { self.stop_transcription(); self.speech_status = "Speech provider disconnected".into(); }
                    TranscriptEvent::Connected => { self.speech_status = "Listening (offline Whisper)".into(); }
                    _ => {}
                }
            }
            Message::AddToDetections { session, translation, result } => {
                if self.speech.as_ref().map(|s| s.id) != Some(session) || self.translation.label() != translation { return Task::none(); }
                match result {
                    Ok(verses) => {
                        for verse in verses {
                            self.recent_detections.retain(|v| v.reference != verse.reference || v.translation != verse.translation);
                            self.recent_detections.insert(0, verse);
                        }
                        self.recent_detections.truncate(30);
                    }
                    Err(error) => self.speech_status = format!("Verse detection: {error}"),
                }
            }
            Message::SearchCompleted(revision, result) => {
                if revision != self.search_revision { return Task::none(); }
                self.search_pending = false;
                match result {
                    Ok(verses) => { self.search_results = verses; self.search_error = None; }
                    Err(error) => { self.search_results.clear(); self.search_error = Some(error); }
                }
            }

            Message::SearchQueryChanged(q) => {
                self.invalidate_search();
                self.search_query = q;
                self.translation_dropdown_open = false;
            }

            Message::SearchModeChanged(mode) => {
                self.invalidate_search();
                self.search_mode = mode;
                self.search_results.clear();
                self.search_query.clear();
            }

            Message::TranslationChanged(t) => {
                self.invalidate_search();
                self.translation = t;
                self.translation_dropdown_open = false;
                return self.detect_transcript();
            }

            Message::TranslationDropdownToggled => {
                self.translation_dropdown_open = !self.translation_dropdown_open;
            }

//             Message::SearchSubmitted => {
//                 if !self.search_query.is_empty() {
//                     // Simulate search results with sample data
//                     self.search_results = sample_search_results(&self.search_query, &self.translation);
//                 }
//                 self.translation_dropdown_open = false;
//             }
//
            Message::SearchSubmitted => {
                self.invalidate_search();
                self.translation_dropdown_open = false;
                if self.search_query.trim().is_empty() { return Task::none(); }
                let Some(repository) = self.scripture.clone() else {
                    self.search_error = Some(self.bible_error.clone().unwrap_or_else(|| "Bible database is still loading".into()));
                    return Task::none();
                };
                self.search_pending = true;
                let revision = self.search_revision;
                let query = self.search_query.clone();
                let mode = self.search_mode.clone();
                let translation = self.translation.label().to_string();
                return Task::perform(async move {
                    tokio::task::spawn_blocking(move || repository.search(&query, &mode, &translation))
                        .await.map_err(|e| e.to_string()).and_then(|r| r)
                }, move |result| Message::SearchCompleted(revision, result));
            }

            // Queue
            Message::AddToQueue(verse) => {
                self.queue.push(verse);
            }

            Message::RemoveFromQueue(idx) => {
                if idx < self.queue.len() {
                    self.queue.remove(idx);
                }
            }

            Message::PresentVerse(verse) => {
                self.preview_verse = Some(verse.clone());
                if self.go_live {
                    self.live_verse = Some(verse);
                    self.sync_ndi_verse();
                }
            }

            Message::ClearQueue => {
                self.queue.clear();
            }

            Message::GoLiveToggled => {
                self.go_live = !self.go_live;
                if self.go_live {
                    self.live_verse = self.preview_verse.clone();
                    self.sync_ndi_verse();
                    // Start NDI session on the worker thread — gracefully
                    // degrades (via ndi.error()) if the library isn't found.
                    self.ndi.start(NdiStartRequest {
                        source_name: "Logos Bible Display".into(),
                        resolution: NdiResolution::R1080p,
                        frame_rate: NdiFrameRate::Fps30,
                        alpha_mode: NdiAlphaMode::NoneOpaque,
                    });
                } else {
                    self.live_verse = None;
                    self.sync_ndi_verse();
                    self.ndi.stop();
                }
            }

            Message::ClearLive => {
                self.live_verse = None;
                self.preview_verse = None;
                self.sync_ndi_verse();
            }

            // Tour
            Message::TourNext => {
                if let Some(step) = &self.tour_step.clone() {
                    self.tour_step = step.next();
                    if self.tour_step == Some(TourStep::Done) {
                        self.tour_step = None;
                    }
                }
            }

            Message::TourBack => {
                if let Some(step) = &self.tour_step.clone() {
                    self.tour_step = step.prev();
                }
            }

            Message::TourSkip | Message::TourDismiss => {
                self.tour_step = None;
            }

            Message::InstallUpdate => {
                self.show_update_banner = false;
                // Would trigger real update in production
            }

            Message::DismissUpdate => {
                self.show_update_banner = false;
            }

            Message::LoadingComplete => {
                self.loading = LoadingState::Ready;
            }

            Message::OpenSettings  => {
                // Would open modals in full implementation
            }

            // Message::ToggleHelpMenu => {
            //     self.help_menu_open = !self.help_menu_open;
            // }

            Message::OpenAboutLogos => {
                self.help_menu_open = false;
                // Placeholder for the About Logos action.
            }

            Message::PaneDragged(event) => {
                // Only swap on center drop — edge drops would create new splits
                if let pane_grid::DragEvent::Dropped {
                    pane,
                    target: pane_grid::Target::Pane(other, pane_grid::Region::Center),
                } = event
                {
                    self.pane_grid_state.swap(pane, other);
                }
            }

            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.pane_grid_state.resize(split, ratio);
            }

            Message::ResetLayout => {
                self.pane_grid_state =
                    pane_grid::State::with_configuration(default_pane_config());
            }

            Message::NdiStatusTick => {
                // No-op: processing this message is enough to trigger a
                // repaint, which re-reads self.ndi.any_active()/error() in
                // view() with whatever the worker thread has published.
            }
        }

        Task::none()
    }

    fn invalidate_search(&mut self) {
        self.search_revision = self.search_revision.wrapping_add(1);
        self.search_pending = false;
        self.search_error = None;
        self.search_results.clear();
    }

    fn stop_transcription(&mut self) {
        self.audio_capture = None;
        self.audio_shared = None;
        self.speech = None;
        self.is_transcribing = false;
        self.waveform.fill(0.0);
        self.audio_rms = 0.0;
        self.audio_peak = 0.0;
    }

    fn detect_transcript(&self) -> Task<Message> {
        let Some(session) = &self.speech else { return Task::none(); };
        let Some(repository) = self.scripture.clone() else { return Task::none(); };
        let id = session.id;
        let translation = self.translation.label().to_string();
        let label = translation.clone();
        // Only recent context is needed; completed detections are retained separately.
        let recent = self.transcript_text.chars().rev().take(512).collect::<String>().chars().rev().collect::<String>();
        let references = self.detector.detect(&recent);
        if references.is_empty() { return Task::none(); }
        Task::perform(async move {
            tokio::task::spawn_blocking(move || {
                let mut verses = Vec::new();
                for reference in references { verses.extend(repository.lookup(&reference, &translation)?); }
                Ok(verses)
            }).await.map_err(|e| e.to_string()).and_then(|r| r)
        }, move |result| Message::AddToDetections { session: id, translation: label.clone(), result })
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.loading {
            LoadingState::Loading(progress) => views::loading::view(*progress),
            LoadingState::Ready => views::main::view(self),
        }
    }

    // pub fn remaining_formatted(&self) -> String {
    //     let h = self.remaining_seconds / 3600;
    //     let m = (self.remaining_seconds % 3600) / 60;
    //     let s = self.remaining_seconds % 60;
    //     format!("{:01}:{:02}:{:02}", h, m, s)
    // }
}

// Downsample `WAVEFORM_LEN` samples → `n_bars` amplitude values
/// by taking the RMS of each chunk, then normalising 0..1.
fn downsample_waveform(samples: &[f32], n_bars: usize) -> Vec<f32> {
    if samples.is_empty() {
        return vec![0.0; n_bars];
    }
    let chunk = (samples.len() / n_bars).max(1);
    let mut bars: Vec<f32> = (0..n_bars)
        .map(|i| {
            let start = (i * samples.len() / n_bars).min(samples.len());
            let end = ((i + 1) * samples.len() / n_bars).min(samples.len());
            let slice = &samples[start..end];
            if slice.is_empty() {
                return 0.0;
            }
            let rms = (slice.iter().map(|s| s * s).sum::<f32>() / slice.len() as f32).sqrt();
            rms
        })
        .collect();
 
    // Normalise to 0..1 based on the observed peak in this frame
    let max = bars.iter().cloned().fold(0.0f32, f32::max).max(1e-6);
    // Use a soft ceiling: normalise against 0.3 RMS or observed max, whichever is larger.
    // This makes quiet speech fill the visualiser nicely.
    let ceiling = max.max(0.05);
    bars.iter_mut().for_each(|b| *b = (*b / ceiling).min(1.0));
    let _ = chunk;
    bars
}

// Legacy sample search retained for reference; production uses rhema-bible.
// fn sample_search_results(query: &str, translation: &Translation) -> Vec<Verse> {
//     let q = query.to_lowercase();
//
//     let all_verses = vec![
//         Verse::new(
//             "John 3:16",
//             "For God so loved the world that he gave his one and only Son, that whoever believes in him shall not perish but have eternal life.",
//             translation.label(),
//         ),
//         Verse::new(
//             "John 3:17",
//             "For God did not send his Son into the world to condemn the world, but to save the world through him.",
//             translation.label(),
//         ),
//         Verse::new(
//             "Romans 8:28",
//             "And we know that in all things God works for the good of those who love him, who have been called according to his purpose.",
//             translation.label(),
//         ),
//         Verse::new(
//             "Psalm 23:1",
//             "The LORD is my shepherd, I lack nothing.",
//             translation.label(),
//         ),
//         Verse::new(
//             "Philippians 4:13",
//             "I can do all this through him who gives me strength.",
//             translation.label(),
//         ),
//         Verse::new(
//             "Isaiah 40:31",
//             "But those who hope in the LORD will renew their strength. They will soar on wings like eagles; they will run and not grow weary, they will walk and not be faint.",
//             translation.label(),
//         ),
//         Verse::new(
//             "Jeremiah 29:11",
//             "For I know the plans I have for you, declares the LORD, plans to prosper you and not to harm you, plans to give you hope and a future.",
//             translation.label(),
//         ),
//         Verse::new(
//             "Proverbs 3:5",
//             "Trust in the LORD with all your heart and lean not on your own understanding.",
//             translation.label(),
//         ),
//         Verse::new(
//             "Jonah 2:8",
//             "Those who regard worthless idols forsake their own Mercy.",
//             translation.label(),
//         ),
//     ];
//
//     all_verses
//         .into_iter()
//         .filter(|v| {
//             v.reference.to_lowercase().contains(&q)
//                 || v.text.to_lowercase().contains(&q)
//         })
//         .take(6)
//         .collect()
// }
//
fn version_parts(version: &str) -> (u32, u32, u32) {
    let mut parts = version.split('.').map(|part| part.parse::<u32>().unwrap_or(0));

    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

fn check_for_update() -> bool {
    let current = version_parts(CURRENT_VERSION);
    let latest = version_parts(LATEST_VERSION);

    current.0 < latest.0
        || (current.0 == latest.0 && current.1 < latest.1)
        || (current.0 == latest.0 && current.1 == latest.1 && current.2 < latest.2)
}

pub fn default_pane_config() -> pane_grid::Configuration<PaneKind> {
    use pane_grid::{Axis, Configuration};
    // Two independent horizontal splits, each starting at the same ratio
    // so Search and Recent Detections open at the same height by default,
    // but can be resized independently.
    Configuration::Split {
        axis: Axis::Vertical,
        ratio: 0.22,
        a: Box::new(Configuration::Pane(PaneKind::LiveTranscript)),
        b: Box::new(Configuration::Split {
            axis: Axis::Vertical,
            ratio: 0.74,
            a: Box::new(Configuration::Split {
                axis: Axis::Horizontal,
                ratio: 0.54,  // center column: Preview+LiveDisplay above, Search below
                a: Box::new(Configuration::Split {
                    axis: Axis::Vertical,
                    ratio: 0.5,
                    a: Box::new(Configuration::Pane(PaneKind::ProgramPreview)),
                    b: Box::new(Configuration::Pane(PaneKind::LiveDisplay)),
                }),
                b: Box::new(Configuration::Pane(PaneKind::Search)),
            }),
            b: Box::new(Configuration::Split {
                axis: Axis::Horizontal,
                ratio: 0.54,  // right column: Queue above, Recent Detections below
                a: Box::new(Configuration::Pane(PaneKind::Queue)),
                b: Box::new(Configuration::Pane(PaneKind::RecentDetections)),
            }),
        }),
    }
}
