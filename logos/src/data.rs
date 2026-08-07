// TCNJ AI/ML Group

/// A single Bible verse
#[derive(Debug, Clone)]
pub struct Verse {
    pub reference: String,
    pub text: String,
    // Recorded on every verse but not yet surfaced in the UI.
    #[allow(dead_code)]
    pub translation: String,
}

impl Verse {
    pub fn new(reference: &str, text: &str, translation: &str) -> Self {
        Self {
            reference: reference.to_string(),
            text: text.to_string(),
            translation: translation.to_string(),
        }
    }
}

/// Translation options
#[derive(Debug, Clone, PartialEq)]
pub enum Translation {
    Niv,
    Kjv,
    Esv,
    Nlt,
    Nasb,
}

impl Translation {
    pub fn label(&self) -> &str {
        match self {
            Translation::Niv => "NIV",
            Translation::Kjv => "KJV",
            Translation::Esv => "ESV",
            Translation::Nlt => "NLT",
            Translation::Nasb => "NASB",
        }
    }

    pub fn all() -> Vec<Translation> {
        vec![
            Translation::Niv,
            Translation::Kjv,
            Translation::Esv,
            Translation::Nlt,
            Translation::Nasb,
        ]
    }
}

/// Search mode
#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Book,
    Context,
}

/// Tour step
#[derive(Debug, Clone, PartialEq)]
pub enum TourStep {
    LiveTranscript,
    AiDetections,
    BookSearch,
    ContextSearch,
    VerseQueue,
    Preview,
    LiveOutput,
    Broadcast,
    Done,
}

impl TourStep {
    pub fn title(&self) -> &str {
        match self {
            TourStep::LiveTranscript => "Live Transcript",
            TourStep::AiDetections => "AI Detections",
            TourStep::BookSearch => "Book Search",
            TourStep::ContextSearch => "Context Search",
            TourStep::VerseQueue => "Verse Queue",
            TourStep::Preview => "Preview",
            TourStep::LiveOutput => "Live Output",
            TourStep::Broadcast => "Broadcast",
            TourStep::Done => "Done",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            TourStep::LiveTranscript => {
                "Click \"Start transcript\" to begin. Detected verses are highlighted."
            }
            TourStep::AiDetections => "Detected verses appear here. Present or queue them.",
            TourStep::BookSearch => "Search by reference like \"John 3:16\".",
            TourStep::ContextSearch => "Find verses by theme, quote, or paraphrase.",
            TourStep::VerseQueue => "Queued verses ready to present.",
            TourStep::Preview => "See how verses look before going live.",
            TourStep::LiveOutput => {
                "Toggle \"Go live\" to broadcast verses.\nShortcut: L"
            }
            TourStep::Broadcast => "Configure NDI, Spout, or Syphon output.",
            TourStep::Done => "You're all set!",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            TourStep::LiveTranscript => 1,
            TourStep::AiDetections => 2,
            TourStep::BookSearch => 3,
            TourStep::ContextSearch => 4,
            TourStep::VerseQueue => 5,
            TourStep::Preview => 6,
            TourStep::LiveOutput => 7,
            TourStep::Broadcast => 8,
            TourStep::Done => 9,
        }
    }

    pub fn total() -> usize {
        10
    }

    pub fn next(&self) -> Option<TourStep> {
        match self {
            TourStep::LiveTranscript => Some(TourStep::AiDetections),
            TourStep::AiDetections => Some(TourStep::BookSearch),
            TourStep::BookSearch => Some(TourStep::ContextSearch),
            TourStep::ContextSearch => Some(TourStep::VerseQueue),
            TourStep::VerseQueue => Some(TourStep::Preview),
            TourStep::Preview => Some(TourStep::LiveOutput),
            TourStep::LiveOutput => Some(TourStep::Broadcast),
            TourStep::Broadcast => Some(TourStep::Done),
            TourStep::Done => None,
        }
    }

    pub fn prev(&self) -> Option<TourStep> {
        match self {
            TourStep::LiveTranscript => None,
            TourStep::AiDetections => Some(TourStep::LiveTranscript),
            TourStep::BookSearch => Some(TourStep::AiDetections),
            TourStep::ContextSearch => Some(TourStep::BookSearch),
            TourStep::VerseQueue => Some(TourStep::ContextSearch),
            TourStep::Preview => Some(TourStep::VerseQueue),
            TourStep::LiveOutput => Some(TourStep::Preview),
            TourStep::Broadcast => Some(TourStep::LiveOutput),
            TourStep::Done => Some(TourStep::Broadcast),
        }
    }
}

/// App loading state
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    Loading(f32), // progress 0.0 - 1.0
    Ready,
}