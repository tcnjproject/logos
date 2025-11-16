# Product Requirements Document (PRD)
## AI Sermon Assistant for Clergy

**Version**: 1.1 (Revised)  
**Date**: November 16, 2025  
**Status**: Approved for Development  
**Owner**: Product Development Team

---

## 1. Executive Summary

### 1.1 Product Vision
An intelligent desktop application that assists church operators during live sermons by automatically detecting Bible references from the pastor's speech and displaying scripture passages on projection screens in real-time, with full manual control capabilities.

### 1.2 Problem Statement
During live sermons, church media operators struggle to:
- Quickly locate and display Bible verses as pastors reference them
- Keep up with the pace of sermon delivery
- Find verses when only partial quotes or paraphrases are given
- Switch between multiple Bible translations smoothly
- Maintain focus on the sermon while operating display equipment

### 1.3 Solution Overview
A Windows desktop application that:
- Listens to live audio from the church sound system
- Automatically transcribes and detects Bible references (explicit and implicit)
- Presents multiple translation options to the operator
- Displays selected verses on projection screens with customizable formatting
- Works completely offline with no internet dependency
- Provides full manual control as a fallback

### 1.4 Success Metrics
- **Speed**: Scripture displayed on projector within 3-4 seconds of verbal reference
- **Accuracy**: 90%+ correct detection of explicit references
- **Semantic Match**: 75%+ correct detection of common implicit quotes (60%+ for all verses)
- **Reliability**: 99%+ uptime during services (no crashes)
- **Adoption**: Reduces operator workload by 70%

---

## 2. Product Goals & Objectives

### 2.1 Primary Goals
1. **Real-time Reference Detection**: Automatically identify Bible references from live sermon audio
2. **Multi-Translation Support**: Display verses in 4 different Bible versions simultaneously
3. **Operator Control**: Maintain human oversight for all displayed content (NO auto-display)
4. **Offline Operation**: Function completely without internet connectivity
5. **Professional Display**: Provide customizable, high-quality projection output

### 2.2 Secondary Goals
1. Maintain comprehensive logs of references used during sermons
2. Support manual Bible search and navigation with keyboard shortcuts
3. Enable easy customization of display formatting
4. Provide session management for different services/preachers
5. Include operator training materials and in-app guidance

### 2.3 Non-Goals (Out of Scope for v1.0)
- Video recording or streaming capabilities
- Presentation slide management (PowerPoint/Keynote)
- Lyrics or worship song display
- Multi-language support (English only in v1.0, Spanish prioritized for v2.0)
- Cloud synchronization
- Mobile app versions
- Commentary or study notes
- Automatic display without operator confirmation

---

## 3. User Personas

### 3.1 Primary User: Church Media Operator
**Demographics**:
- Age: 25-60
- Technical skill: Basic to intermediate
- Role: Volunteer or staff member managing audio/visual during services

**Goals**:
- Quickly display accurate Bible verses during sermons
- Minimize errors and delays
- Focus on sermon content, not technical operations
- Provide high-quality visual experience for congregation

**Pain Points**:
- Difficulty keeping up with pastor's pace
- Struggling to find verses from partial quotes
- Switching between Bible translations manually
- Missing references due to multitasking

**User Story**: *"As a media operator, I want the system to automatically detect Bible references so I can confirm and display them quickly without losing focus on the sermon."*

### 3.2 Secondary User: Pastor/Preacher
**Demographics**:
- Age: 30-70
- Technical skill: Varies widely
- Role: Delivers sermons, references multiple scriptures

**Goals**:
- Have verses displayed promptly and accurately
- Freedom to quote or paraphrase without exact citations
- Support multiple Bible translations for congregation preference

**Pain Points**:
- Delays in scripture display disrupting sermon flow
- Incorrect verses displayed
- Having to slow down or repeat references

**User Story**: *"As a pastor, I want to freely reference scriptures knowing they'll be displayed accurately, whether I cite 'John 3:16' or quote 'for God so loved the world.'"*

### 3.3 Tertiary User: IT Administrator
**Demographics**:
- Age: 25-55
- Technical skill: Intermediate to advanced
- Role: Manages church technology infrastructure

**Goals**:
- Easy installation and configuration
- Reliable operation during services
- Minimal maintenance requirements
- Compatibility with existing AV equipment

**Pain Points**:
- Complex software requiring constant troubleshooting
- Compatibility issues with church hardware
- Unclear system requirements

**User Story**: *"As an IT admin, I want to install the software once, configure it properly, and have it work reliably every Sunday without intervention."*

---

## 4. Functional Requirements

### 4.1 Audio Capture & Processing

#### FR-1.1: Audio Input Configuration
- **Priority**: P0 (Critical)
- **Description**: System shall accept audio input from church mixing board via USB or line-in interface
- **Acceptance Criteria**:
  - Supports standard Windows audio input devices
  - Allows operator to select input device from dropdown
  - Displays real-time audio level indicator
  - Automatically detects audio device disconnection
  - Shows audio quality indicator (Good/Fair/Poor)

#### FR-1.2: Voice Activity Detection
- **Priority**: P1 (High)
- **Description**: System shall distinguish between speech and non-speech audio (music, silence, congregation responses)
- **Acceptance Criteria**:
  - Filters out ambient noise below configurable threshold
  - Detects speech boundaries for phrase chunking
  - Minimizes processing during non-speech periods
  - Distinguishes pastor voice from congregation responses (voice fingerprinting)

#### FR-1.3: Real-time Transcription
- **Priority**: P0 (Critical)
- **Description**: System shall transcribe spoken words to text with <2 second latency
- **Acceptance Criteria**:
  - Uses Whisper small.en model for English speech-to-text
  - Processes audio in streaming/chunked mode to reduce buffering delay
  - Displays live transcript in operator window
  - Maintains transcription accuracy >90% for clear speech
  - Handles religious terminology correctly (Bible names, theological terms)

#### FR-1.4: Audio Quality Calibration
- **Priority**: P1 (High)
- **Description**: System shall provide audio setup wizard for optimal configuration
- **Acceptance Criteria**:
  - Calibration wizard tests with pastor's voice during setup
  - Real-time feedback on audio quality (signal strength, clarity, noise level)
  - Displays clear messaging: "Audio quality: Good/Fair/Poor - AI may struggle"
  - Suggests adjustments for poor audio (increase gain, reduce noise, reposition mic)
  - Saves calibration profile per audio device

### 4.2 Reference Detection

#### FR-2.1: Explicit Reference Detection
- **Priority**: P0 (Critical)
- **Description**: System shall detect explicitly stated Bible references using pattern matching
- **Acceptance Criteria**:
  - Recognizes formats: "John 3:16", "John chapter 3 verse 16", "First Corinthians 13"
  - Supports all 66 Bible book names and common abbreviations
  - Detects verse ranges (e.g., "verses 1-5", "1 through 5")
  - Handles chapter-only references (e.g., "Psalm 23")
  - Detection accuracy >95% for properly cited references

**Supported Reference Formats**:
```
- Book Chapter:Verse (John 3:16)
- Book Chapter Verse (John 3 16)
- Book ch. Chapter v. Verse (John ch. 3 v. 16)
- Ordinal Book (First Corinthians, 2nd Timothy)
- Book alone (Philemon, Jude)
- Verse ranges (John 3:16-18, Romans 8:1-4)
- Multiple verses (Matthew 5:3, 5, 7)
```

#### FR-2.2: Semantic/Implicit Quote Detection
- **Priority**: P0 (Critical)
- **Description**: System shall detect Bible verses from paraphrases or partial quotes using semantic matching
- **Acceptance Criteria**:
  - Uses sentence embeddings to find similar verses
  - Returns top 5 candidates with confidence scores
  - Tiered confidence display: "High confidence" (>90%), "Good match" (85-90%), "Possible match" (75-85%)
  - Processes semantic search within 500ms
  - Detection accuracy >75% for top-100 most-quoted verses, >60% for all verses
  - Initially prioritizes high-frequency verses for better accuracy

**Examples**:
```
Spoken: "God loved the world so much"
→ Detects: John 3:16 (High confidence: 96%)

Spoken: "Love is patient, love is kind"
→ Detects: 1 Corinthians 13:4 (High confidence: 98%)

Spoken: "The Lord is my shepherd"
→ Detects: Psalm 23:1 (High confidence: 97%)

Spoken: "Nothing can separate us from God's love"
→ Detects: Romans 8:38-39 (Good match: 88%)
```

#### FR-2.3: Context-Aware Detection
- **Priority**: P2 (Medium)
- **Description**: System shall use recent context to improve detection accuracy
- **Acceptance Criteria**:
  - Tracks recently mentioned books/chapters
  - Prioritizes matches from current context
  - Handles pronouns ("verse 5" after "John 3:16" → John 3:5)
  - Context window duration adjustable (30s - 5min, default 2min) based on preaching style
  - Resets context on book change or extended silence
  - Handles rapid-fire references across different books
  - Maintains context through long illustrations within same chapter

#### FR-2.4: Match Ranking & Confidence Scoring
- **Priority**: P1 (High)
- **Description**: System shall rank multiple possible matches by confidence level
- **Acceptance Criteria**:
  - Displays confidence percentage for each match
  - Ranks explicit matches higher than semantic matches
  - Combines detection methods for better accuracy
  - Highlights most likely match for operator
  - Color-codes by confidence (green >90%, yellow 85-90%, orange 75-85%, gray <75%)

#### FR-2.5: User Feedback Loop
- **Priority**: P2 (Medium)
- **Description**: System shall learn from operator corrections to improve matching
- **Acceptance Criteria**:
  - "Was this correct?" checkbox after displaying verse
  - Tracks false positives and false negatives
  - Displays match accuracy stats in settings
  - Optional anonymous usage data to improve model (opt-in)

### 4.3 Bible Database & Versions

#### FR-3.1: Multi-Version Support
- **Priority**: P0 (Critical)
- **Description**: System shall support multiple Bible translations simultaneously
- **Acceptance Criteria**:
  - Includes 4 default versions: KJV, NIV, AMPC, TPT
  - Stores all versions in local SQLite database with FTS5 indexing
  - Displays all 4 versions in parallel for operator review
  - Allows operator to select display version per session
  - Shows copyright indicators for licensed versions

#### FR-3.2: Version Management
- **Priority**: P1 (High)
- **Description**: Operator shall be able to configure active Bible versions
- **Acceptance Criteria**:
  - Settings panel to select/deselect versions
  - Ability to reorder version display priority
  - Changes take effect immediately without restart
  - Default version persists across sessions
  - Clear licensing information displayed per version
  - Links to obtain public display licenses where required

#### FR-3.3: Verse Lookup Performance
- **Priority**: P1 (High)
- **Description**: System shall retrieve verses instantly
- **Acceptance Criteria**:
  - Database queries complete in <50ms
  - Full-text search across all verses in <200ms
  - Supports verse ranges (e.g., John 3:16-18)
  - Caches recently accessed chapters in memory
  - Pre-computed common verse ranges for faster lookup

**Database Schema Optimizations**:
```sql
-- Full-text search index
CREATE INDEX idx_verses_text ON verses(text) USING FTS5;

-- Fast book/chapter lookup
CREATE INDEX idx_verses_book_chapter ON verses(book_id, chapter);

-- Pre-computed verse ranges
CREATE TABLE verse_ranges (
  range_key TEXT PRIMARY KEY,
  verse_ids TEXT,
  book_id INTEGER,
  chapter INTEGER
);

-- Version metadata with licensing info
CREATE TABLE versions (
  id INTEGER PRIMARY KEY,
  code TEXT UNIQUE,
  name TEXT,
  copyright_year INTEGER,
  copyright_holder TEXT,
  license_type TEXT, -- 'public_domain', 'restricted', 'licensed'
  license_url TEXT,
  display_attribution TEXT
);
```

### 4.4 Operator Interface (Control Window)

#### FR-4.1: Live Transcript Display
- **Priority**: P0 (Critical)
- **Description**: Operator shall see real-time transcription of sermon audio
- **Acceptance Criteria**:
  - Scrolling text display with timestamps
  - Auto-scrolls to latest text
  - Highlights detected references in transcript
  - Maintains last 5 minutes of transcript visible
  - Shows audio quality indicator in header

#### FR-4.2: Match Suggestions Panel
- **Priority**: P0 (Critical)
- **Description**: System shall present detected references as selectable cards
- **Acceptance Criteria**:
  - Displays each match with confidence percentage
  - Shows verse preview (first 50 characters)
  - Includes book, chapter, verse reference
  - Tiered confidence display: "High confidence", "Good match", "Possible match"
  - Color-codes by confidence (green >90%, yellow 85-90%, orange 75-85%)
  - Maximum 5 matches displayed at once
  - Keyboard navigation support (Tab, Enter to select)

#### FR-4.3: Four-Version Preview
- **Priority**: P0 (Critical)
- **Description**: Operator shall review selected verse in all 4 translations before display
- **Acceptance Criteria**:
  - Tabbed interface for each version
  - Shows full verse text for each translation
  - Highlights differences between versions (optional)
  - "Push to Screen" button for each version (keyboard: Space)
  - Verse reference clearly labeled
  - Copyright attribution visible when required
  - Auto-scales text for long verses to fit preview

#### FR-4.4: Manual Search Mode
- **Priority**: P0 (Critical)
- **Description**: Operator shall be able to search Bible manually without AI
- **Acceptance Criteria**:
  - Dropdown/searchable list of all Bible books
  - Chapter number input (with validation)
  - Verse number/range input
  - Autocomplete for book names
  - Keyboard shortcuts for quick navigation (Ctrl+F to focus search)
  - Works independently of audio/AI system
  - Accessible via prominent button or keyboard shortcut

#### FR-4.5: Keyboard Shortcuts
- **Priority**: P0 (Critical - moved from v1.1)**
- **Description**: All critical operations accessible via keyboard
- **Acceptance Criteria**:
  - **Space**: Push selected verse to screen
  - **Esc**: Clear projector display (blank screen)
  - **Arrow Up/Down**: Navigate match suggestions
  - **Arrow Left/Right**: Navigate version tabs
  - **Ctrl+F**: Focus manual search
  - **Ctrl+H**: Toggle history panel
  - **Ctrl+1/2/3/4**: Quick-select version 1/2/3/4
  - **Ctrl+Z**: Show previous verse
  - **F11**: Toggle projector fullscreen
  - All shortcuts customizable in settings
  - Quick reference card (printable PDF)

#### FR-4.6: Session Management
- **Priority**: P2 (Medium)
- **Description**: Operator shall manage sermon session details
- **Acceptance Criteria**:
  - Create new session with date, preacher name, sermon title
  - Display current session info in header
  - End session to finalize logs
  - View previous session summaries
  - Quick access to recent sessions

#### FR-4.7: History Log
- **Priority**: P1 (High)
- **Description**: Operator shall see all references displayed during current session
- **Acceptance Criteria**:
  - Chronological list of displayed verses
  - Includes timestamp, reference, version shown
  - Click to re-display previous verse
  - Keyboard shortcut (Ctrl+Z) to show last verse
  - Export session log as text/PDF

#### FR-4.8: In-App Tutorial
- **Priority**: P1 (High)
- **Description**: First-time users receive guided onboarding
- **Acceptance Criteria**:
  - Interactive tutorial overlay on first launch
  - Highlights key interface elements
  - Step-by-step walkthrough (5 steps, <2 minutes)
  - Can be skipped or replayed from Help menu
  - Video walkthrough available (3-5 minute explainer)

### 4.5 Projector Output (Display Window)

#### FR-5.1: Full-Screen Verse Display
- **Priority**: P0 (Critical)
- **Description**: System shall display verses on projector in full-screen mode
- **Acceptance Criteria**:
  - Renders on secondary display (auto-detect projector)
  - Fills screen with optimized layout
  - No visible window chrome/controls for audience
  - Smooth transitions between verses (<300ms)
  - Supports common resolutions (1080p, 4K)

#### FR-5.2: Customizable Typography
- **Priority**: P0 (Critical)
- **Description**: Operator shall customize all text formatting
- **Acceptance Criteria**:
  - Font family selection (system fonts + common church fonts)
  - Font size (40-200px range)
  - Auto-scaling for long verses (adjusts size to fit screen)
  - Font weight (normal, bold, extra-bold)
  - Text color (full color picker)
  - Background color (full color picker, including images)
  - Text alignment (left, center, right)
  - Line spacing adjustment
  - Settings preview before applying
  - Text shadow/outline for better readability (optional)

#### FR-5.3: Reference Information Display
- **Priority**: P1 (High)
- **Description**: Verse reference shall be visible but de-emphasized
- **Acceptance Criteria**:
  - Shows "Book Chapter:Verse" format
  - Smaller font size than main text (20-60px)
  - Configurable position (top-left, top-right, bottom-left, bottom-right, center-bottom)
  - Can be toggled on/off
  - Same color customization as main text

#### FR-5.4: Copyright Attribution Display
- **Priority**: P1 (High)
- **Description**: Required copyright notices displayed with verses
- **Acceptance Criteria**:
  - Displays attribution text for licensed versions (e.g., "Scripture taken from NIV®...")
  - Small, unobtrusive positioning (bottom-right or bottom-left)
  - Font size 12-18px (smaller than reference)
  - Can be customized but not removed for copyrighted versions
  - Auto-populated from version metadata

#### FR-5.5: Navigation Controls
- **Priority**: P1 (High)
- **Description**: Operator shall navigate verses from display window
- **Acceptance Criteria**:
  - Hidden overlay controls (visible to operator only)
  - Previous/Next verse buttons
  - Return to operator window button
  - Keyboard shortcuts (Arrow keys, Esc)
  - Controls fade out after 3 seconds of inactivity

#### FR-5.6: Display Modes
- **Priority**: P2 (Medium)
- **Description**: System shall support different display layouts
- **Acceptance Criteria**:
  - Single verse mode (default)
  - Multi-verse mode for verse ranges (auto-paginate if >3 verses)
  - Verse range pagination (e.g., "John 3:16-18" shows all verses, "1 of 1" if fits, "1 of 2" if paginated)
  - Split-screen mode (compare 2 versions side-by-side)
  - Blank screen mode (hide content quickly with Esc)

### 4.6 Settings & Configuration

#### FR-6.1: Audio Settings
- **Priority**: P1 (High)
- **Description**: Operator shall configure audio input parameters
- **Acceptance Criteria**:
  - Select input device from available devices
  - Adjust sensitivity threshold
  - Test audio input with level meter
  - Enable/disable voice activity detection
  - Run calibration wizard
  - Save device-specific profiles

#### FR-6.2: Display Settings
- **Priority**: P0 (Critical)
- **Description**: Operator shall configure all display formatting
- **Acceptance Criteria**:
  - Font settings (as per FR-5.2)
  - Reference position and size
  - Background color/image
  - Text alignment and line spacing
  - Text shadow/outline options
  - Live preview pane showing changes
  - Save/Load preset configurations
  - Reset to defaults option

#### FR-6.3: Bible Settings
- **Priority**: P1 (High)
- **Description**: Operator shall manage Bible versions and preferences
- **Acceptance Criteria**:
  - Select 4 active versions from installed versions
  - Reorder version display priority
  - Set default display version
  - Configure version abbreviations
  - View licensing information per version
  - Links to obtain public display licenses
  - Import additional Bible versions (future)

#### FR-6.4: Detection Settings
- **Priority**: P2 (Medium)
- **Description**: Operator shall tune reference detection sensitivity
- **Acceptance Criteria**:
  - Adjust semantic match threshold (75-95%)
  - Enable/disable detection methods independently
  - Set context window duration (30s - 5min)
  - Configure tiered confidence display thresholds
  - View detection accuracy stats
  - NO auto-display option (always requires confirmation)

#### FR-6.5: Keyboard Shortcut Settings
- **Priority**: P1 (High)
- **Description**: Operator shall customize keyboard shortcuts
- **Acceptance Criteria**:
  - Rebind all shortcuts to preferred keys
  - Conflict detection (warns if key already assigned)
  - Reset to defaults option
  - Export/import shortcut configurations
  - Printable reference card

#### FR-6.6: System Settings
- **Priority**: P2 (Medium)
- **Description**: System configuration and maintenance options
- **Acceptance Criteria**:
  - Select projector display
  - Enable/disable logging
  - Database maintenance (vacuum, backup)
  - Check for updates (manual)
  - View system diagnostics
  - Export diagnostic report for troubleshooting

### 4.7 Logging & History

#### FR-7.1: Session Logging
- **Priority**: P1 (High)
- **Description**: System shall log all displayed verses per session
- **Acceptance Criteria**:
  - Records timestamp, reference, version, detection method
  - Stores transcript snippet triggering detection
  - Includes confidence score for AI detections
  - Links to sermon session metadata
  - Tracks operator corrections (feedback loop)

#### FR-7.2: Log Viewing
- **Priority**: P2 (Medium)
- **Description**: Operator shall review past sermon logs
- **Acceptance Criteria**:
  - List all sessions with date, preacher, title
  - View detailed log for each session
  - Search logs by reference, date, preacher
  - Filter by detection method
  - View accuracy statistics

#### FR-7.3: Log Export
- **Priority**: P2 (Medium)
- **Description**: Operator shall export session logs
- **Acceptance Criteria**:
  - Export as plain text
  - Export as PDF with formatting
  - Export as CSV for analysis
  - Include all session metadata
  - Include accuracy stats

### 4.8 Performance Requirements

#### FR-8.1: End-to-End Latency
- **Priority**: P0 (Critical)
- **Description**: Total time from speech to projector display
- **Acceptance Criteria**:
  - **Target**: 3-4 seconds average
  - **Maximum**: 5 seconds for 95th percentile
  - **Breakdown**:
    - Audio buffering/streaming: 0.5-1.0s (reduced via streaming)
    - Transcription: 0.5-1.0s
    - Detection: 0.2-0.5s
    - Semantic search (if needed): 0.5-1.0s
    - UI update: 0.1-0.2s
    - Operator confirmation: 0.5-1.0s

#### FR-8.2: Application Responsiveness
- **Priority**: P0 (Critical)
- **Description**: UI shall remain responsive during processing
- **Acceptance Criteria**:
  - UI interactions respond in <100ms
  - No blocking operations on main thread
  - Progress indicators for long operations
  - Graceful degradation under high CPU load
  - Process health checks between main and renderer

#### FR-8.3: Resource Usage
- **Priority**: P1 (High)
- **Description**: System shall operate within resource constraints
- **Acceptance Criteria**:
  - RAM usage <1GB under normal operation
  - CPU usage <50% average (spikes allowed during speech)
  - Disk usage <800MB for full installation
  - No memory leaks over 4-hour sessions

---

## 5. Non-Functional Requirements

### 5.1 Performance

#### NFR-1.1: Startup Time
- Application launches in <5 seconds
- Models load in <10 seconds on first run
- Subsequent launches use cached models (<3 seconds)

#### NFR-1.2: Detection Accuracy
- Explicit references: >95% accuracy
- Semantic matches: >75% accuracy for top-100 verses, >60% for all verses
- False positive rate: <5%

#### NFR-1.3: Transcription Quality
- Word Error Rate (WER): <10% for clear audio
- Handles religious terminology correctly (Bible names, theological terms)

### 5.2 Reliability

#### NFR-2.1: Uptime
- 99.9% uptime during typical 2-hour service
- Graceful error recovery without restart
- Auto-reconnect to audio device if disconnected

#### NFR-2.2: Data Integrity
- No data loss during crashes
- Database corruption prevention
- Automatic backup before updates
- Session auto-recovery after crash

#### NFR-2.3: Error Handling
- All errors logged with context
- User-friendly error messages
- Fallback to manual mode if AI fails
- IPC heartbeat monitoring

### 5.3 Usability

#### NFR-3.1: Learning Curve
- New operators productive within 15 minutes
- Manual mode usable without training
- AI features optional, not required
- In-app tutorial completes in <2 minutes

#### NFR-3.2: Accessibility
- Keyboard navigation for all functions
- High contrast mode support
- Font size adjustable for operator window
- WCAG 2.0 AA compliance for interface

#### NFR-3.3: User Interface
- Clean, uncluttered operator interface
- Clear visual hierarchy
- Consistent design language
- Projector display optimized for 1080p and 4K

### 5.4 Compatibility

#### NFR-4.1: Operating System
- Windows 10 (64-bit) minimum
- Windows 11 supported
- Installer size <200MB (excluding AI models)

#### NFR-4.2: Hardware Requirements
**Minimum**:
- CPU: Intel i5 6th gen (or AMD equivalent)
- RAM: 4GB
- Storage: 2GB free space
- Audio: USB or 3.5mm line-in
- Display: 1920x1080 (operator) + 1920x1080 (projector)

**Recommended**:
- CPU: Intel i5 8th gen or better
- RAM: 8GB
- Storage: 5GB free space (for logs)
- Display: 1920x1080 (operator) + 4K (projector)

#### NFR-4.3: Audio Interface
- Supports standard Windows audio devices
- USB audio interfaces (class-compliant)
- 3.5mm line-in/microphone input
- Sample rates: 16kHz, 44.1kHz, 48kHz

#### NFR-4.4: Display Output
- HDMI output to projector
- DisplayPort support
- USB-C with DisplayPort Alt Mode
- Extended desktop mode (not mirrored)

### 5.5 Security & Privacy

#### NFR-5.1: Data Privacy
- No data transmitted to external servers
- All processing local to machine
- Sermon transcripts stored locally only
- Optional log deletion
- Anonymous usage telemetry (opt-in only)
- Secure contextBridge for IPC

#### NFR-5.2: File Security
- Database encryption (optional)
- Secure file permissions
- No sensitive data in logs

### 5.6 Maintainability

#### NFR-6.1: Code Quality
- Modular architecture
- Comprehensive error logging
- Unit test coverage >70%
- Integration tests for critical paths

#### NFR-6.2: Diagnostics
- Built-in diagnostic mode
- Performance profiling tools
- Log viewer for troubleshooting
- System information export

### 5.7 Licensing & Legal

#### NFR-7.1: Bible Version Licensing
- Compliance with all translation copyrights
- Display appropriate copyright notices on screen
- User responsible for obtaining public performance licenses
- Clear documentation of licensing requirements
- Installation wizard warning about licensing
- In-app links to licensing information
- Copyright metadata stored per version

#### NFR-7.2: Software Licensing
- Open-source components properly attributed
- Third-party library compliance
- End User License Agreement (EULA)

---

## 6. Technical Specifications

### 6.1 Architecture

#### Technology Stack
- **Desktop Framework**: Electron 28.x
- **UI Library**: React 18.x
- **Styling**: Tailwind CSS 3.x
- **Database**: SQLite 3.x (better-sqlite3)
- **Speech-to-Text**: Whisper.cpp (small.en model)
- **Semantic Search**: Transformers.js (all-MiniLM-L6-v2)
- **Audio Processing**: node-speaker / Web Audio API
- **Fuzzy Search**: Fuse.js

#### System Architecture
```
┌─────────────────────────────────────────────┐
│           Electron Main Process              │
│  ┌────────────┐  ┌──────────────────────┐   │
│  │   Audio    │  │   Window Manager     │   │
│  │  Capture   │  │ - Operator Window    │   │
│  │ (Streaming)│  │ - Projector Window   │   │
│  └────────────┘  └──────────────────────┘   │
│  ┌────────────┐  ┌──────────────────────┐   │
│  │  Whisper   │  │   Bible Database     │   │
│  │    STT     │  │   (SQLite + FTS5)    │   │
│  │ (Chunked)  │  │   + Version Metadata │   │
│  └────────────┘  └──────────────────────┘   │
│  ┌──────────────────────────────────────┐   │
│  │      Detection Engine                │   │
│  │  - Reference Parser (Regex)          │   │
│  │  - Semantic Matcher (Embeddings)     │   │
│  │  - Match Ranker (Tiered Confidence)  │   │
│  │  - Context Tracker                   │   │
│  └──────────────────────────────────────┘   │
│  ┌──────────────────────────────────────┐   │
│  │      IPC Heartbeat Monitor           │   │
│  └──────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
                    ↕ IPC + ContextBridge
┌─────────────────────────────────────────────┐
│        Electron Renderer Processes           │
│  ┌──────────────────┐  ┌─────────────────┐  │
│  │ Operator Window  │  │ Projector Window│  │
│  │   (React App)    │  │   (React App)   │  │
│  │ - Tutorial       │  │ - Auto-scale    │  │
│  │ - Keyboard Nav   │  │ - Copyright     │  │
│  └──────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────┘
```

### 6.2 Data Models

#### Key Data Structures
```typescript
// Detected Reference
interface DetectedReference {
  id: string;
  timestamp: Date;
  reference: {
    book: string;
    chapter: number;
    verseStart: number;
    verseEnd?: number;
  };
  confidence: number;
  confidenceTier: 'high' | 'good' | 'possible'; // Added
  method: 'explicit' | 'semantic' | 'manual';
  transcriptSnippet: string;
  matches: VerseMatch[];
  operatorFeedback?: 'correct' | 'incorrect'; // Added for learning
}

// Verse Match
interface VerseMatch {
  verseId: number;
  book: string;
  chapter: number;
  verse: number;
  versions: {
    [versionCode: string]: string; // e.g., { KJV: "text...", NIV: "text..." }
  };
  copyright: {
    [versionCode: string]: string; // Attribution text
  };
}

// Display Settings
interface DisplaySettings {
  font: {
    family: string;
    size: number;
    weight: 'normal' | 'bold' | 'extra-bold';
    color: string;
    autoScale: boolean; // Added for long verses
    shadow?: {
      enabled: boolean;
      offsetX: number;
      offsetY: number;
      blur: number;
      color: string;
    };
  };
  background: {
    color: string;
    image?: string;
  };
  reference: {
    size: number;
    position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'center-bottom';
    visible: boolean;
  };
  copyright: { // Added
    size: number;
    position: 'bottom-left' | 'bottom-right';
    visible: boolean;
  };
  alignment: 'left' | 'center' | 'right';
  lineSpacing: number;
}

// Sermon Session
interface SermonSession {
  id: number;
  date: Date;
  title?: string;
  preacher?: string;
  logs: SessionLog[];
  stats: {
    totalReferences: number;
    explicitDetections: number;
    semanticDetections: number;
    manualSearches: number;
    averageConfidence: number;
  };
}

// Session Log
interface SessionLog {
  id: number;
  timestamp: Date;
  reference: string; // e.g., "John 3:16"
  versionDisplayed: string;
  detectionMethod: string;
  confidence: number;
  transcriptSnippet: string;
  operatorFeedback?: 'correct' | 'incorrect';
}

// Bible Version Metadata
interface BibleVersion {
  id: number;
  code: string; // e.g., "NIV"
  name: string; // e.g., "New International Version"
  copyrightYear: number;
  copyrightHolder: string;
  licenseType: 'public_domain' | 'restricted' | 'licensed';
  licenseUrl?: string;
  displayAttribution: string; // e.g., "Scripture taken from NIV®..."
  requiresLicense: boolean;
}

// Audio Calibration Profile
interface AudioProfile {
  deviceId: string;
  deviceName: string;
  sensitivity: number;
  noiseThreshold: number;
  vadEnabled: boolean;
  qualityScore: 'good' | 'fair' | 'poor';
  calibrationDate: Date;
}

// Keyboard Shortcut Configuration
interface KeyboardShortcuts {
  pushToScreen: string; // Default: "Space"
  clearScreen: string; // Default: "Escape"
  navigateUp: string; // Default: "ArrowUp"
  navigateDown: string; // Default: "ArrowDown"
  navigateLeft: string; // Default: "ArrowLeft"
  navigateRight: string; // Default: "ArrowRight"
  focusSearch: string; // Default: "Ctrl+F"
  toggleHistory: string; // Default: "Ctrl+H"
  quickVersion1: string; // Default: "Ctrl+1"
  quickVersion2: string; // Default: "Ctrl+2"
  quickVersion3: string; // Default: "Ctrl+3"
  quickVersion4: string; // Default: "Ctrl+4"
  previousVerse: string; // Default: "Ctrl+Z"
  toggleFullscreen: string; // Default: "F11"
}
```

### 6.3 API Specifications

#### IPC Channels (Electron)
```javascript
// Main → Renderer
'audio-status' // Audio device connection status + quality
'audio-quality-update' // Real-time audio quality metrics
'stt-transcript' // Real-time transcript updates
'reference-detected' // New reference detected with confidence tier
'verse-loaded' // Verse data loaded with copyright info
'projector-connected' // Projector display status
'heartbeat' // IPC health check (every 5s)

// Renderer → Main
'get-audio-devices' // Request available audio devices
'set-audio-device' // Set active audio device
'run-audio-calibration' // Start calibration wizard
'start-listening' // Start audio capture
'stop-listening' // Stop audio capture
'search-verse' // Manual verse search
'display-verse' // Push verse to projector (with version + copyright)
'navigate-verse' // Previous/next verse
'update-settings' // Save display settings
'update-shortcuts' // Save keyboard shortcuts
'export-log' // Export session log
'log-operator-feedback' // Log "was this correct?" response
'request-diagnostics' // Get system diagnostic data
'heartbeat-ack' // Acknowledge heartbeat
```

### 6.4 Whisper Integration Details

**Streaming/Chunked Processing**:
```javascript
// Instead of fixed 1.5s buffer:
const audioChunks = [];
let processingQueue = [];

audioStream.on('data', (chunk) => {
  audioChunks.push(chunk);
  
  // Process when we have 0.5-1s of audio
  if (audioChunks.length >= MIN_CHUNK_SIZE) {
    processingQueue.push(audioChunks.splice(0));
    processNextChunk();
  }
});

async function processNextChunk() {
  if (processingQueue.length === 0) return;
  
  const chunk = processingQueue.shift();
  const transcript = await whisper.transcribe(chunk);
  
  // Send to detection engine immediately
  detectReferences(transcript);
  
  // Continue processing queue
  if (processingQueue.length > 0) {
    processNextChunk();
  }
}
```

**Voice Fingerprinting** (optional enhancement):
```javascript
// Distinguish pastor from congregation
const voiceProfile = await calibrateVoiceProfile(pastorSample);

audioStream.on('data', (chunk) => {
  const similarity = compareVoiceProfile(chunk, voiceProfile);
  
  if (similarity > THRESHOLD) {
    // This is likely the pastor speaking
    processChunk(chunk);
  } else {
    // Congregation or music, skip
    return;
  }
});
```

---

## 7. User Interface Specifications

### 7.1 Operator Window Layout

#### Window Dimensions
- **Minimum**: 1280x720px
- **Recommended**: 1920x1080px
- **Resizable**: Yes
- **Layout**: Fixed panels with proportional resizing

#### Main Sections
```
┌──────────────────────────────────────────────────────┐
│ Header: Session Info | Audio Quality: Good ● | [⚙]  │
├────────────────┬─────────────────────────────────────┤
│                │                                      │
│   Transcript   │   Matches Panel                     │
│   Panel        │   ┌───────────────────┐             │
│   (Live STT)   │   │ HIGH CONFIDENCE   │             │
│                │   │ John 3:16 (96%)   │             │
│   "For God so  │   │ "For God so..."   │ ← Selected  │
│   loved the    │   └───────────────────┘             │
│   world..."    │   ┌───────────────────┐             │
│                │   │ GOOD MATCH        │             │
│   [Shortcuts]  │   │ John 3:17 (88%)   │             │
│                │   │ "For God did..."  │             │
│                │   └───────────────────┘             │
│                │                                      │
├────────────────┴─────────────────────────────────────┤
│   Preview Panel (4 Versions) + Copyright             │
│   [KJV] [NIV] [AMPC] [TPT]                           │
│   "For God so loved the world, that he gave..."      │
│   NIV® Copyright © 1973, 2011 by Biblica, Inc.®     │
│   [Push to Screen → Space] Was this correct? □      │
├──────────────────────────────────────────────────────┤
│ Manual: [Book▼ Ctrl+F] [Ch#] [V#] [Go] OR [History] │
└──────────────────────────────────────────────────────┘
```

**Tutorial Overlay** (First Launch):
```
┌──────────────────────────────────────────┐
│  Welcome to AI Sermon Assistant!         │
│                                           │
│  This is the transcript panel →          │
│  [1/5]                      [Next]       │
└──────────────────────────────────────────┘
```

### 7.2 Projector Window Layout

#### Window Dimensions
- **Fullscreen**: Automatically sized to display resolution
- **Common Resolutions**: 1920x1080, 3840x2160 (4K)
- **Aspect Ratios**: 16:9 (primary), 16:10 supported

#### Display Layout
```
┌────────────────────────────────────────────────┐
│                                                │
│           John 3:16 (KJV)                      │
│                                                │
│                                                │
│         For God so loved the world,            │
│         that he gave his only begotten         │
│         Son, that whosoever believeth          │
│         in him should not perish, but          │
│         have everlasting life.                 │
│                                                │
│                                                │
│                                                │
│    KJV: Public Domain          [Pagination]   │
└────────────────────────────────────────────────┘
```

**Long Verse Auto-Scaling**:
```
┌────────────────────────────────────────────────┐
│         Romans 8:38-39 (NIV)                   │
│                                                │
│  For I am convinced that neither death nor     │
│  life, neither angels nor demons, neither      │
│  the present nor the future, nor any powers,   │
│  neither height nor depth, nor anything else   │
│  in all creation, will be able to separate us  │
│  from the love of God that is in Christ Jesus  │
│  our Lord.                                     │
│                                                │
│  NIV® Copyright © 2011 by Biblica, Inc.®      │
└────────────────────────────────────────────────┘
```

**Verse Range Pagination** (if >3 verses):
```
┌────────────────────────────────────────────────┐
│         Matthew 5:3-5 (ESV)                    │
│                                                │
│  Blessed are the poor in spirit, for theirs    │
│  is the kingdom of heaven.                     │
│                                                │
│  Blessed are those who mourn, for they shall   │
│  be comforted.                                 │
│                                                │
│  Blessed are the meek, for they shall inherit  │
│  the earth.                                    │
│                                                │
│  Page 1 of 1                                   │
│  ESV® Copyright © 2001 by Crossway             │
└────────────────────────────────────────────────┘
```

### 7.3 Color Scheme

#### Operator Window (Default)
- **Primary**: #2563EB (Blue)
- **Secondary**: #64748B (Slate)
- **Success/High Confidence**: #16A34A (Green)
- **Warning/Good Match**: #F59E0B (Amber)
- **Caution/Possible Match**: #F97316 (Orange)
- **Error**: #DC2626 (Red)
- **Background**: #F8FAFC (Light Gray)
- **Text**: #1E293B (Dark Gray)
- **Audio Quality Good**: #16A34A (Green)
- **Audio Quality Fair**: #F59E0B (Amber)
- **Audio Quality Poor**: #DC2626 (Red)

#### Projector Window (Customizable)
- **Default Background**: #000000 (Black)
- **Default Text**: #FFFFFF (White)
- **Default Reference**: #CCCCCC (Light Gray)
- **Default Copyright**: #999999 (Darker Gray, smaller)

### 7.4 Typography

#### Operator Window
- **Primary Font**: Inter, system-ui, sans-serif
- **Monospace**: 'Courier New', monospace (for transcripts)
- **Sizes**: 12px (small), 14px (body), 16px (headings), 20px (titles)

#### Projector Window (Customizable)
- **Default Font**: Arial, sans-serif
- **Verse Text**: 60-120px (default 80px, auto-scales for long verses)
- **Reference Text**: 24-48px (default 32px)
- **Copyright Text**: 12-18px (default 14px)

---

## 8. User Workflows

### 8.1 Typical Service Workflow

#### Pre-Service Setup (5 minutes)
1. Operator launches application
2. **NEW**: Audio quality indicator shows "Good" (green) or runs calibration if needed
3. Verifies audio input connection
4. Tests microphone/line-in with pastor doing sound check
5. Confirms projector display working
6. Creates new sermon session (date, preacher name, title)
7. Reviews/adjusts display settings if needed
8. **NEW**: Glances at keyboard shortcuts quick reference

#### During Service (Automated Flow)
1. Pastor begins preaching
2. System transcribes speech in real-time (streaming chunks)
3. Reference detected: "John 3:16"
4. Match card appears: **"HIGH CONFIDENCE - John 3:16 (96%)"**
5. Operator presses **Arrow Down** to select, **Enter** or **Space** to preview
6. Preview panel shows verse in all 4 versions with copyright notices
7. Operator reviews and presses **Space** (or clicks "Push to Screen")
8. Verse appears on projector with copyright attribution within 3 seconds
9. **NEW**: Operator optionally checks "Was this correct?" for feedback
10. Process repeats for each reference throughout sermon

#### During Service (Semantic Match)
1. Pastor says: "Love is patient, love is kind..."
2. System shows: **"GOOD MATCH - 1 Corinthians 13:4 (89%)"**
3. Operator verifies this is correct reference
4. Pushes to screen with **Space**
5. Congregation sees verse with attribution

#### During Service (Manual Fallback)
1. Pastor mentions obscure reference AI doesn't catch
2. Operator presses **Ctrl+F** to focus manual search
3. Types "Hab" → autocomplete suggests "Habakkuk"
4. Enters chapter 2, verse 4, presses **Enter**
5. Preview shows all versions
6. Presses **Space** to push desired version to screen
7. Total time: <10 seconds

#### Post-Service (Optional)
1. Operator clicks "End Session"
2. Reviews session log with accuracy stats
3. Exports log as PDF for pastor
4. Closes application

### 8.2 First-Time Setup Workflow

#### Installation (10 minutes)
1. Run installer executable
2. Accept EULA (includes Bible licensing warnings)
3. Select installation directory
4. Installer downloads AI models (~500MB)
5. Installer imports Bible databases with copyright metadata
6. Pre-computes verse embeddings (background process)
7. Application launches for first time

#### Initial Configuration (5-7 minutes)
1. **Welcome wizard appears with tutorial option**
2. **NEW**: Interactive tutorial overlay (5 steps, <2 minutes) - can skip
3. **Audio Calibration Wizard**:
   - Select audio input device
   - Pastor speaks test phrase
   - System displays audio quality: "Good - Ready to use"
   - Saves calibration profile
4. Select projector display (auto-detected)
5. Choose 4 default Bible versions (shows licensing info)
6. Test display with sample verse
7. Adjust font size/color to preference with live preview
8. **NEW**: Review keyboard shortcuts quick reference
9. Complete setup → Ready to use

### 8.3 Settings Adjustment Workflow

#### Changing Display Font
1. Operator clicks Settings button (⚙️)
2. Navigate to Display Settings tab
3. Select font family from dropdown
4. Adjust size with slider (preview updates live)
5. Enable auto-scaling for long verses (checkbox)
6. Choose text color with color picker
7. Choose background color
8. **NEW**: Add text shadow for readability (optional)
9. Preview pane shows real-time changes
10. Click "Apply" → Settings saved
11. Next displayed verse uses new formatting

#### Managing Bible Versions
1. Open Settings → Bible Settings
2. See current 4 active versions with licensing status
3. Click "Change Versions"
4. Modal shows all installed versions with indicators:
   - ✓ KJV (Public Domain)
   - ⚠️ NIV (Requires Public Display License) [View License Info]
   - ✓ AMPC (Licensed)
   - ⚠️ TPT (Requires License) [Get License]
5. Select/deselect to choose 4 active
6. System warns if selecting copyrighted version without acknowledging license
7. Drag to reorder display priority
8. Click "Save" → Preview panel updates with new versions

#### Customizing Keyboard Shortcuts
1. Open Settings → Keyboard Shortcuts
2. See list of all actions with current bindings
3. Click on shortcut to rebind (e.g., "Push to Screen: Space")
4. Press new key combination
5. System warns if conflict detected
6. Apply changes or reset to defaults
7. **Download printable reference card (PDF)**

### 8.4 Error Recovery Workflows

#### Audio Disconnection
1. System detects audio device disconnected
2. **Audio quality indicator turns red: "Poor - Device lost"**
3. Alert: "Audio input lost. Reconnect device or select different input."
4. Operator plugs in USB audio interface
5. System auto-detects and reconnects
6. Runs quick calibration check
7. **Green indicator returns: "Good - Reconnected"**
8. Transcription resumes

#### Poor Audio Quality During Service
1. **Audio quality indicator shows "Fair" (amber) or "Poor" (red)**
2. Tooltip: "Audio quality degraded - increase gain or reduce noise"
3. Operator adjusts mixing board levels
4. System monitors and updates indicator in real-time
5. If quality doesn't improve, operator uses manual mode more frequently

#### AI Detection Failure
1. Pastor quotes verse but AI doesn't detect
2. Operator notices no matches appearing
3. Operator presses **Ctrl+F**, uses manual search as backup
4. Finds and displays verse manually
5. **Optionally marks as "not detected" in feedback log**
6. Service continues uninterrupted
7. After service, operator reviews logs to see if issue is recurring

#### Application Crash (Rare)
1. Crash occurs during service
2. Operator relaunches application (5 seconds)
3. **App auto-recovers last session with notification**
4. Shows "Session Restored - John 3:16 was last displayed"
5. Last displayed verse still on projector (window persisted)
6. Operator continues from current point
7. No data lost (session log intact)

#### Low Confidence Matches
1. Semantic detection shows: **"POSSIBLE MATCH - Psalm 100:1 (78%)"**
2. Operator is unsure if this is correct
3. Operator clicks match to preview
4. Sees verse text doesn't match pastor's quote
5. Operator dismisses and uses manual search instead
6. **Marks as "incorrect match" for learning feedback**

---

## 9. Constraints & Assumptions

### 9.1 Technical Constraints

#### Hardware Limitations
- Minimum i5 6th gen CPU required (Whisper model constraint)
- 4GB RAM minimum (models require ~500MB loaded in memory)
- Windows-only for v1.0 (Electron supports others, but focused scope)
- **Note**: 3-4 second latency target may be 4-5 seconds on minimum spec hardware

#### Software Constraints
- Offline-only operation (no cloud fallback)
- English language only for v1.0 (Spanish prioritized for v2.0)
- Maximum 4 Bible versions displayed simultaneously (UI space constraint)
- SQLite database size limit: ~2TB (sufficient for hundreds of Bible versions)

#### Performance Constraints
- Speech-to-text processing limited by CPU speed
- Semantic search speed depends on embedding computation
- Cannot guarantee <3 second latency on minimum spec hardware
- Real-time transcription requires continuous audio input
- Streaming processing reduces but doesn't eliminate buffering delay

### 9.2 Assumptions

#### User Environment
- Church has existing sound system with accessible audio output
- Available USB port or line-in on computer
- Projector connected via HDMI/DisplayPort
- Windows PC meets minimum specifications
- Operator has basic computer literacy
- **Operator can dedicate attention during sermon (not multitasking heavily)**

#### Usage Patterns
- Typical sermon: 30-45 minutes
- Average 10-20 scripture references per sermon
- Pastor speaks clearly into microphone
- Minimal background noise during sermon (worship music ended)
- Operator present and attentive throughout service
- **Operator willing to provide feedback for system improvement**

#### Audio Quality
- Clear audio signal from mixing board
- Pastor's voice isolated on primary channel (or mixable)
- Minimal feedback, echo, or distortion
- Consistent audio levels (not clipping or too quiet)
- **Audio quality varies by church - calibration helps adapt**

#### Network & Internet
- No internet required for operation
- Initial setup may require internet for model downloads (~500MB)
- Updates distributed as offline installers
- **Churches may have restricted/no internet access**

### 9.3 Dependencies

#### Third-Party Software
- Windows Audio API for audio capture
- DirectX/Windows Display API for multi-monitor
- Electron runtime (bundled with app)
- Node.js native modules (compiled for Windows)

#### Third-Party Services
- None (fully offline operation)
- Optional: Anonymous telemetry endpoint (opt-in, for improving model)

#### External Data Sources
- Bible text files (provided with installer)
- Pre-trained AI models (Whisper, sentence-transformers)
- **Must maintain model archives independently**

#### Licensing Dependencies
- Must comply with Bible translation copyrights
- User responsible for obtaining public display licenses
- Open-source component licenses (MIT, Apache 2.0)
- **Legal review required before including copyrighted translations**

---

## 10. Development Roadmap

### 10.1 Version 1.0 (MVP) - 13 Weeks

#### Phase 1: Foundation & Data (Weeks 1-2)
**Deliverables**:
- Database schema implemented (with copyright metadata)
- Bible text imported for all 4 versions (KJV, NIV, AMPC, TPT)
- Copyright/licensing metadata populated
- Basic Electron shell with dual windows
- IPC with contextBridge security
- Project structure and build system
- Version control setup

**Success Criteria**:
- App launches without errors
- Database queries execute successfully with FTS5
- Both windows render basic UI
- IPC communication working

#### Phase 2: Manual Mode + Keyboard Nav (Week 3)
**Deliverables**:
- Manual Bible search interface
- Book/chapter/verse selector components
- Autocomplete for book names
- Four-version preview panel
- Push-to-screen functionality
- Basic projector display
- **Keyboard shortcut system (Ctrl+F, Space, Arrows)**
- **Printable quick reference card**

**Success Criteria**:
- Operator can search any verse manually
- Verse displays on projector with basic formatting
- All 4 versions accessible
- **All manual operations keyboard-accessible**

#### Phase 3: Audio & Transcription (Weeks 4-5)
**Deliverables**:
- Audio capture from line-in/USB
- **Audio calibration wizard**
- **Audio quality monitoring and indicator**
- Whisper STT integration (streaming/chunked)
- Live transcript display
- Audio settings panel
- Voice activity detection
- **Optional: Voice fingerprinting for pastor identification**

**Success Criteria**:
- Real-time transcription visible in operator window
- **Audio quality indicator working (Good/Fair/Poor)**
- **Streaming processing reduces latency to <1s**
- Audio level indicator working

#### Phase 4: Explicit Reference Detection (Weeks 6-7)
**Deliverables**:
- Regex-based reference parser
- Book name/abbreviation recognition (all 66 books)
- Reference extraction from transcript
- Match card UI components
- Detection confidence scoring
- **Tiered confidence display (High/Good/Possible)**
- Context tracking system (adjustable window)

**Success Criteria**:
- Detects "John 3:16" format references
- Handles common variations and abbreviations
- 95%+ accuracy on explicit references
- **Context-aware detection working (handles "verse 5" after "John 3:16")**

#### Phase 5: Semantic Matching (Week 8)
**Deliverables**:
- Sentence embedding generation (all-MiniLM-L6-v2)
- Pre-computed verse embeddings (priority: top-100 verses)
- Semantic similarity search
- Implicit quote detection
- Combined detection ranking
- **Test with alternative models if accuracy insufficient**

**Success Criteria**:
- Detects paraphrases of well-known verses
- 75%+ accuracy on top-100 verses, 60%+ on all verses
- Sub-second semantic search
- **False positive rate <5%**

#### Phase 6: Display Customization + Copyright (Week 9)
**Deliverables**:
- Settings window implementation
- Font customization controls
- **Auto-scaling for long verses**
- **Text shadow/outline options**
- Display settings (colors, alignment, line spacing)
- **Copyright attribution display on projector**
- Live preview pane
- Settings persistence
- Save/load preset configurations

**Success Criteria**:
- All display settings functional
- **Copyright notices display correctly per version**
- Changes apply immediately with preview
- Settings saved across sessions
- **Long verses auto-scale to fit screen**

#### Phase 7: Logging, Tutorial & Polish (Week 10)
**Deliverables**:
- Session management system
- Event logging to database
- **Operator feedback tracking ("Was this correct?")**
- History log display
- Log export functionality (text, PDF, CSV)
- **In-app tutorial overlay (5 steps, <2 minutes)**
- **Video walkthrough recording (3-5 minutes)**
- Error handling throughout
- **IPC heartbeat monitoring**
- **Diagnostics export tool**

**Success Criteria**:
- Complete session logs captured
- Export as text/PDF/CSV working
- **Feedback loop functional**
- No unhandled errors
- **Tutorial guides new users successfully**

#### Phase 8: Testing & Optimization (Weeks 11-13) **[Extended]**
**Deliverables**:
- Performance profiling and optimization
- Database query optimization
- Memory leak detection and fixes
- End-to-end testing with real sermon audio (10+ samples)
- User acceptance testing (3-5 beta churches)
- **Audio quality variability testing (poor/fair/good conditions)**
- **Edge case handling (rapid references, obscure books, long pauses)**
- Bug fixes and refinements
- Documentation and help system
- **Keyboard shortcut reference card finalized**
- Windows installer package
- **Setup video tutorial**

**Success Criteria**:
- Meets all performance targets (3-4 seconds on recommended, 4-5s on minimum)
- Zero critical bugs
- Successful testing with 3+ real churches
- **>90% operator satisfaction in beta testing**
- Ready for production deployment

### 10.2 Version 1.1 (Enhancements) - 4 Weeks

**Target Features**:
- Additional Bible versions support (ESV, NASB, NLT, NKJV, etc.)
- Import custom Bible translations
- Multiple display layouts (split-screen, comparison)
- Enhanced context tracking
- Better handling of verse ranges (auto-pagination)
- Improved semantic matching accuracy (fine-tuning on religious corpus)
- Background image support for projector
- Dark mode for operator interface
- Export sermon outlines with references

### 10.3 Version 2.0 (Advanced Features) - 8 Weeks

**Target Features**:
- **Multi-language support (Spanish priority, then French, Portuguese)**
- Commentary integration (optional popups)
- Cross-reference suggestions
- Sermon outline tracking
- Integration with presentation software (PowerPoint/Keynote)
- Cloud backup for logs (optional, opt-in)
- Mobile companion app (preview/control from phone)
- Advanced analytics (most-used verses, detection accuracy trends)
- Custom styling themes (templates)
- Multiple projector support
- Real-time collaboration (backup operator)
- Advanced audio: speaker diarization (multiple pastors)

---

## 11. Success Metrics & KPIs

### 11.1 Performance Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| End-to-end latency | 3-4 seconds (recommended), 4-5s (minimum) | Timestamp from speech to display |
| Explicit detection accuracy | >95% | Manual testing with 100 references |
| Semantic detection accuracy (top-100) | >75% | Testing with 50 common paraphrases |
| Semantic detection accuracy (all verses) | >60% | Testing with 50 random paraphrases |
| False positive rate | <5% | Count of incorrect detections |
| Application uptime | 99.9% per session | Crash tracking over 100 services |
| Startup time | <10 seconds (first), <3s (subsequent) | Time from launch to ready state |
| Memory usage | <1GB | Monitor during 2-hour sessions |
| Audio quality detection | 90% accuracy | Compare to manual assessment |

### 11.2 User Experience Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Time to first display (manual) | <10 seconds | User testing |
| Learning curve | <15 minutes | New user training time |
| Tutorial completion rate | >80% | Analytics (opt-in) |
| Operator satisfaction | >4.5/5 rating | Post-service surveys |
| Pastor satisfaction | >4.0/5 rating | Post-service surveys |
| Error recovery time | <30 seconds | Testing failure scenarios |
| Settings change time | <2 minutes | Timed user tasks |
| Keyboard shortcut adoption | >60% | Usage logs (opt-in) |

### 11.3 Business Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Adoption rate | 80% of trained users continue using | Usage logs |
| Workload reduction | 70% fewer manual lookups | Compare manual vs. auto |
| Time savings | 5-10 minutes per service | Before/after comparison |
| User retention | 90% at 3 months | Usage tracking |
| Feedback participation | >50% provide feedback | Log analysis |

### 11.4 Quality Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Critical bugs | 0 in production | Bug tracking system |
| Code coverage | >70% | Automated testing |
| Documentation completeness | 100% of features | Manual audit |
| User-reported issues | <5 per month (post v1.0) | Support ticket system |
| Operator feedback accuracy | >85% mark detections correct | Feedback loop data |

---

## 12. Risks & Mitigation

### 12.1 Technical Risks

#### Risk: Whisper transcription accuracy insufficient for religious terminology
- **Probability**: Medium
- **Impact**: High
- **Mitigation**: 
  - Extensive testing with various audio quality levels
  - Fine-tune voice activity detection parameters
  - Fallback to manual mode prominently available
  - Consider Whisper "medium" model for better accuracy (trade-off: speed)
  - Build religious terminology dictionary for post-processing corrections
  - **Audio calibration wizard to optimize per-environment**

#### Risk: Semantic matching produces too many false positives
- **Probability**: Medium → **Mitigated by tiered confidence display**
- **Impact**: Medium
- **Mitigation**:
  - **Implement tiered confidence thresholds (High/Good/Possible)**
  - Show multiple candidates ranked by confidence, let operator choose
  - Context tracking to improve relevance
  - **Initial focus on top-100 most-quoted verses (higher accuracy)**
  - Continuous refinement based on operator feedback
  - **Never auto-display - always require human confirmation**

#### Risk: Cannot meet 3-4 second latency target on minimum hardware
- **Probability**: Low → **Medium on minimum spec**
- **Impact**: High
- **Mitigation**:
  - **Adjust target: 3-4s on recommended, 4-5s acceptable on minimum**
  - Extensive performance profiling early (Phase 3)
  - Optimize critical paths (database queries, embeddings)
  - **Streaming audio processing to reduce buffering delay**
  - Provide recommended specs prominently in documentation
  - Consider GPU acceleration for future versions
  - **Display performance warning if system is underpowered**

#### Risk: Audio device compatibility issues across church setups
- **Probability**: Medium → **High due to church AV diversity**
- **Impact**: Medium
- **Mitigation**:
  - Test with multiple audio interfaces (USB, line-in, pro audio)
  - **Audio calibration wizard for setup optimization**
  - **Real-time audio quality monitoring**
  - Clear setup documentation with troubleshooting flowchart
  - Audio troubleshooting guide
  - Manual mode always available as fallback
  - **Support for wide range of sample rates (16kHz-48kHz)**

#### Risk: Whisper/Transformers.js model availability changes
- **Probability**: Low
- **Impact**: High
- **Mitigation**:
  - Bundle models with installer (not download)
  - **Archive models independently on secure storage**
  - Abstraction layer for easy model swapping
  - Monitor upstream projects closely
  - **Have backup models tested and ready**

### 12.2 User Experience Risks

#### Risk: Operator overwhelmed by interface complexity
- **Probability**: Medium → **Low with tutorial**
- **Impact**: High
- **Mitigation**:
  - **Interactive tutorial on first launch**
  - Iterative UI testing with real operators
  - Simplified default view (hide advanced features)
  - **Keyboard shortcuts for efficiency**
  - **Video walkthrough (3-5 minutes)**
  - Clear visual hierarchy and labeling
  - **Progressive disclosure of advanced features**

#### Risk: AI detections interrupt worship flow with errors
- **Probability**: Low → **Very Low with tiered confidence**
- **Impact**: High
- **Mitigation**:
  - **Operator confirmation ALWAYS required (NO auto-display)**
  - **Tiered confidence display prevents low-quality suggestions**
  - Easy dismiss/ignore functionality
  - Audio alerts optional and disabled by default
  - Manual mode as primary fallback
  - **Operator feedback loop to improve over time**

#### Risk: Display formatting unsuitable for various projector types
- **Probability**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Extensive display testing (1080p, 4K, various aspect ratios)
  - **Live preview pane shows exactly what will display**
  - Multiple preset configurations (dark background, light background, high contrast)
  - **Auto-scaling for long verses**
  - Easy reset to defaults
  - **Text shadow/outline for readability**

#### Risk: Keyboard shortcuts cause confusion or conflicts
- **Probability**: Low
- **Impact**: Low
- **Mitigation**:
  - **Customizable keyboard shortcuts**
  - Conflict detection during rebinding
  - **Printable quick reference card**
  - Default shortcuts follow common conventions
  - Tutorial includes keyboard shortcut overview
  - Can disable shortcuts and use mouse-only

### 12.3 Business Risks

#### Risk: Bible translation licensing issues delay launch
- **Probability**: Low → **Requires legal review**
- **Impact**: Critical
- **Mitigation**:
  - **Legal review of all included translations BEFORE Phase 1**
  - Clear EULA explaining user responsibilities
  - **Installation wizard warning about licensing requirements**
  - Copyright notices displayed prominently on screen
  - **In-app links to obtain licenses**
  - Only include properly licensed or public domain versions
  - **User must acknowledge licensing terms during setup**
  - Document licensing requirements clearly

#### Risk: Adoption resistance from traditional operators
- **Probability**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Position as assistant, not replacement
  - **Manual mode equally prominent as AI mode**
  - Training and support materials
  - Testimonials from early adopters
  - **Free trial period for churches to test**
  - Emphasize time savings and reduced stress
  - Pastor testimonials about improved service flow

#### Risk: Competition from established church software
- **Probability**: Low → **No competitors offer AI detection**
- **Impact**: Medium
- **Mitigation**:
  - Focus on AI differentiation (unique feature)
  - Better user experience than legacy software
  - Competitive pricing/licensing
  - Active user community
  - **Faster iteration and feature development**
  - **Modern UI vs. dated competitors**

### 12.4 External Risks

#### Risk: Windows API changes break functionality
- **Probability**: Low
- **Impact**: Medium
- **Mitigation**:
  - Use stable, well-documented APIs
  - Test on multiple Windows versions (10, 11)
  - Regular compatibility testing
  - Quick patch release process
  - **Electron framework abstracts many OS differences**

---

## 13. Open Questions & Decisions

### 13.1 Product Questions

#### 1. Multi-operator support
**Question**: Should multiple operators be able to control the system simultaneously? (e.g., backup operator)
- **Decision needed by**: Phase 2
- **Impact**: Architecture complexity, real-time sync
- **Recommendation**: Defer to v2.0, add as "operator handoff" feature

#### 2. Offline updates
**Question**: How should users receive software updates without internet?
- **Decision needed by**: Phase 8
- **Impact**: Distribution strategy
- **Recommendation**: Offline installer downloads from church admin's computer, USB distribution

#### 3. Bible version imports
**Question**: Should users be able to import their own translations (e.g., from BibleGateway)?
- **Decision needed by**: Phase 2
- **Impact**: Database design, legal considerations
- **Recommendation**: v1.1 feature, requires license verification mechanism

#### 4. Presentation integration
**Question**: Should we integrate with PowerPoint/Keynote or keep as standalone?
- **Decision needed by**: Post-v1.0
- **Impact**: Scope expansion
- **Recommendation**: v2.0 feature, significant complexity

#### 5. Telemetry and privacy
**Question**: Should we collect anonymous usage data to improve detection accuracy?
- **Decision needed by**: Phase 7
- **Impact**: Privacy policy, user trust
- **Recommendation**: **Opt-in only, clearly disclosed, with detailed privacy policy**

### 13.2 Technical Questions

#### 1. Embedding model selection
**Question**: Is all-MiniLM-L6-v2 sufficient, or should we test others (e.g., mpnet-base-v2)?
- **Decision needed by**: Phase 5
- **Impact**: Accuracy, performance, model size
- **Recommendation**: **Test both, choose based on accuracy/speed trade-off. Document in Phase 5.**
- **Status**: ⏳ Testing required

#### 2. Database optimization strategy
**Question**: Should we use vector database extension (sqlite-vss) or in-memory similarity search?
- **Decision needed by**: Phase 5
- **Impact**: Performance, complexity, dependencies
- **Recommendation**: **Start with in-memory (simpler), migrate to sqlite-vss if performance insufficient**
- **Status**: ⏳ Test in Phase 5

#### 3. Audio buffering strategy
**Question**: Streaming/chunked processing vs. fixed-size buffers?
- **Decision needed by**: Phase 3
- **Impact**: Latency, accuracy
- **Recommendation**: **✅ DECIDED: Streaming/chunked (reduces latency, accepted in updated spec)**

#### 4. Concurrent processing architecture
**Question**: Web Workers, Worker Threads, or single-threaded with async?
- **Decision needed by**: Phase 3
- **Impact**: Performance, complexity
- **Recommendation**: **Worker Threads for Whisper, main thread for detection (keeps UI responsive)**
- **Status**: ⏳ Implement in Phase 3

#### 5. Context window algorithm
**Question**: Time-based (2 min) vs. reference-count-based (last 5 refs) vs. hybrid?
- **Decision needed by**: Phase 4
- **Impact**: Detection accuracy for contextual references
- **Recommendation**: **Hybrid: time-based default (adjustable), but reset on book change regardless of time**
- **Status**: ⏳ Implement and test in Phase 4

### 13.3 User Experience Questions

#### 1. Auto-display mode
**Question**: Should there be an option for AI to auto-push high-confidence matches (>98%) without operator confirmation?
- **Decision needed by**: Phase 7
- **Impact**: User trust, error handling
- **Recommendation**: **❌ NO - Never in v1.0. Risk too high (1-in-50 errors unacceptable during worship). Keep human-in-loop always.**
- **Status**: ✅ DECIDED: No auto-display

#### 2. Notification sounds
**Question**: Should the app play subtle sounds for detected references?
- **Decision needed by**: Phase 4
- **Impact**: User experience, distraction risk
- **Recommendation**: **Optional, disabled by default. Subtle chime option for operators who want audio feedback.**
- **Status**: ⏳ Add to Phase 6 settings

#### 3. Dark mode
**Question**: Should operator interface support dark theme?
- **Decision needed by**: Phase 6
- **Impact**: Development time (2-3 days)
- **Recommendation**: **Defer to v1.1. Focus on core functionality first.**
- **Status**: ✅ DECIDED: v1.1 feature

#### 4. Mobile preview/control
**Question**: Should there be a companion mobile app for pastors to preview verses?
- **Decision needed by**: Post-v1.0
- **Impact**: Scope expansion, separate app development
- **Recommendation**: **v2.0 feature. Significant value but requires separate development effort.**
- **Status**: ✅ DECIDED: v2.0

#### 5. Confidence threshold customization
**Question**: Should operators be able to adjust confidence thresholds (e.g., only show >85%)?
- **Decision needed by**: Phase 5
- **Impact**: Flexibility vs. complexity
- **Recommendation**: **Yes, add to Phase 6 settings. Default: show all >75%, let advanced users adjust.**
- **Status**: ⏳ Add to settings

---

## 14. Appendices

### Appendix A: Bible Book Names & Abbreviations

**Old Testament (39 books)**:
- Genesis (Gen, Ge, Gn)
- Exodus (Exod, Ex, Exo)
- Leviticus (Lev, Le, Lv)
- Numbers (Num, Nu, Nm, Nb)
- Deuteronomy (Deut, Dt, De)
- Joshua (Josh, Jos, Jsh)
- Judges (Judg, Jdg, Jg, Jdgs)
- Ruth (Rth, Ru)
- 1 Samuel (1 Sam, 1 Sa, 1Samuel, 1S, I Sa, 1 Sm, 1Sa, I Sam, 1Sam, First Samuel)
- 2 Samuel (2 Sam, 2 Sa, 2S, II Sa, 2 Sm, 2Sa, II Sam, 2Sam, Second Samuel)
- 1 Kings (1 Kgs, 1 Ki, 1K, I Kgs, 1Kgs, I Ki, 1Ki, First Kings)
- 2 Kings (2 Kgs, 2 Ki, 2K, II Kgs, 2Kgs, II Ki, 2Ki, Second Kings)
- 1 Chronicles (1 Chron, 1 Chr, 1Ch, I Chr, 1Chr, First Chronicles)
- 2 Chronicles (2 Chron, 2 Chr, 2Ch, II Chr, 2Chr, Second Chronicles)
- Ezra (Ezr, Ez)
- Nehemiah (Neh, Ne)
- Esther (Est, Esth, Es)
- Job (Jb)
- Psalms (Ps, Psalm, Pslm, Psa, Psm, Pss)
- Proverbs (Prov, Pro, Prv, Pr)
- Ecclesiastes (Eccles, Eccle, Ecc, Ec, Qoh)
- Song of Solomon (Song, Song of Songs, SOS, Canticles, Cant)
- Isaiah (Isa, Is)
- Jeremiah (Jer, Je, Jr)
- Lamentations (Lam, La)
- Ezekiel (Ezek, Eze, Ezk)
- Daniel (Dan, Da, Dn)
- Hosea (Hos, Ho)
- Joel (Joe, Jl)
- Amos (Am)
- Obadiah (Obad, Ob)
- Jonah (Jon, Jnh)
- Micah (Mic, Mc)
- Nahum (Nah, Na)
- Habakkuk (Hab, Hb)
- Zephaniah (Zeph, Zep, Zp)
- Haggai (Hag, Hg)
- Zechariah (Zech, Zec, Zc)
- Malachi (Mal, Ml)

**New Testament (27 books)**:
- Matthew (Matt, Mt)
- Mark (Mrk, Mk, Mr)
- Luke (Luk, Lk)
- John (Jn, Jhn)
- Acts (Act, Ac)
- Romans (Rom, Ro, Rm)
- 1 Corinthians (1 Cor, 1 Co, I Cor, 1Cor, First Corinthians)
- 2 Corinthians (2 Cor, 2 Co, II Cor, 2Cor, Second Corinthians)
- Galatians (Gal, Ga)
- Ephesians (Eph, Ephes)
- Philippians (Phil, Php, Pp)
- Colossians (Col, Co)
- 1 Thessalonians (1 Thess, 1 Thes, 1 Th, I Thess, 1Thess, First Thessalonians)
- 2 Thessalonians (2 Thess, 2 Thes, 2 Th, II Thess, 2Thess, Second Thessalonians)
- 1 Timothy (1 Tim, 1 Ti, I Tim, 1Tim, First Timothy)
- 2 Timothy (2 Tim, 2 Ti, II Tim, 2Tim, Second Timothy)
- Titus (Tit, Ti)
- Philemon (Philem, Phm, Pm)
- Hebrews (Heb, He)
- James (Jas, Jm, Jam)
- 1 Peter (1 Pet, 1 Pe, 1 Pt, 1P, I Pet, 1Pet, First Peter)
- 2 Peter (2 Pet, 2 Pe, 2 Pt, 2P, II Pet, 2Pet, Second Peter)
- 1 John (1 Jn, 1 Jhn, 1J, I Jn, 1Jn, First John)
- 2 John (2 Jn, 2 Jhn, 2J, II Jn, 2Jn, Second John)
- 3 John (3 Jn, 3 Jhn, 3J, III Jn, 3Jn, Third John)
- Jude (Jud, Jd)
- Revelation (Rev, Re, Rv, Apocalypse)

### Appendix B: Reference Pattern Examples

**Explicit Patterns**:
```
John 3:16
John 3:16-18
John 3:16-17, 19
John chapter 3 verse 16
John ch. 3 v. 16
John three sixteen
John 3:16-17 and 20-22
First Corinthians 13
1 Corinthians 13:4-7
Romans chapter 8
Psalm 23
The book of John, chapter 3, verse 16
```

**Implicit Patterns** (semantic matching):
```
"God so loved the world" → John 3:16
"Love is patient" → 1 Corinthians 13:4
"The Lord is my shepherd" → Psalm 23:1
"Nothing can separate us from God's love" → Romans 8:38-39
"Fear not" → Isaiah 41:10 (and many others - context helps)
"Greater is He that is in you" → 1 John 4:4
"I can do all things through Christ" → Philippians 4:13
"Be still and know" → Psalm 46:10
```

**Contextual Patterns**:
```
Context: Recently mentioned John 3
"And verse 17 says..." → John 3:17
"In verse 5..." → John 3:5
"Going back to verse 16..." → John 3:16

Context: Preaching through Romans
"In chapter 8, verse 28..." → Romans 8:28
(Without context, would need book name)
```

### Appendix C: Glossary

- **STT**: Speech-to-Text (transcription)
- **Whisper**: OpenAI's speech recognition model
- **Embedding**: Numerical representation of text for semantic similarity
- **Semantic Search**: Finding similar meanings, not just exact words
- **VAD**: Voice Activity Detection (distinguishing speech from silence/music)
- **IPC**: Inter-Process Communication (Electron main ↔️ renderer)
- **SQLite**: Lightweight file-based database
- **FTS**: Full-Text Search (fast text searching in database)
- **Projector Window**: Fullscreen display output for congregation
- **Operator Window**: Control interface for media operator
- **Confidence Score**: Percentage indicating detection certainty (0-100%)
- **Confidence Tier**: Category of confidence (High >90%, Good 85-90%, Possible 75-85%)
- **Session**: Single sermon/service instance with logging
- **Reference**: Bible citation (e.g., "John 3:16")
- **Verse Range**: Multiple consecutive verses (e.g., "John 3:16-18")
- **Explicit Detection**: Pattern-based detection (regex) of stated references
- **Semantic Detection**: AI-based detection of paraphrases/quotes
- **Context Window**: Time period or reference count used for contextual detection
- **Calibration**: Audio setup process to optimize for specific environment
- **Attribution**: Copyright notice displayed with verses
- **Auto-scaling**: Automatic font size adjustment for long verses

### Appendix D: Sample User Stories

**As a media operator**:
- I want to see live transcription so I can follow along with the sermon
- I want the system to suggest verses automatically so I don't miss references
- **I want high-confidence matches clearly indicated so I can trust quick selections**
- I want to review multiple translations before displaying so I choose the right version
- I want manual search readily available so I can find verses the AI misses
- **I want keyboard shortcuts so I can work faster without reaching for the mouse**
- I want to customize display fonts so text is readable on our projector
- **I want to know audio quality in real-time so I can fix issues quickly**
- **I want a tutorial when I first start so I can learn the system quickly**

**As a pastor**:
- I want verses to appear quickly when I reference them so my sermon flow isn't disrupted
- I want freedom to paraphrase scripture without exact citations
- I want multiple translations available to serve diverse congregation preferences
- I want a log of references used so I can review my sermon afterward
- **I want copyright notices displayed so we remain legally compliant**
- **I don't want incorrect verses displayed so I need operator oversight**

**As an IT administrator**:
- I want simple installation so I can deploy quickly
- I want reliable offline operation so services aren't disrupted by internet outages
- I want detailed logs for troubleshooting if issues occur
- I want minimal ongoing maintenance so I can focus on other priorities
- **I want audio calibration tools so I can optimize for our specific setup**
- **I want diagnostic exports so I can troubleshoot problems effectively**

**As a church leadership**:
- **I want clear licensing information so we remain legally compliant with Bible copyrights**
- I want session logs so we can track sermon series progress
- I want reliable technology so our worship services run smoothly
- **I want training materials so new volunteers can learn quickly**

### Appendix E: Competitive Analysis

**ProPresenter** (Renewed Vision):
- **Strengths**: Mature, widely adopted, full presentation features, robust slide management
- **Weaknesses**: No AI detection, manual verse lookup only, expensive ($399-$799), steep learning curve
- **Market Position**: Industry standard for large churches
- **Differentiation**: Our AI detection, offline-first, focused on scripture, lower cost, easier to learn

**EasyWorship** (Softouch):
- **Strengths**: Affordable ($18/mo or $348 one-time), good Bible integration, user-friendly
- **Weaknesses**: No AI features, limited customization, outdated UI, Windows-only
- **Market Position**: Popular with small-medium churches
- **Differentiation**: Modern UI, AI assistance, better typography control, keyboard-first workflow

**OpenLP** (Open Source):
- **Strengths**: Free, open source, cross-platform, active community
- **Weaknesses**: Complex setup, no AI, dated interface, limited support
- **Market Position**: Budget-conscious churches, tech-savvy admins
- **Differentiation**: AI features, better UX, professional polish, Windows-optimized

**Proclaim** (Faithlife):
- **Strengths**: Cloud-based, integrates with Logos Bible software, modern interface
- **Weaknesses**: Requires internet, subscription only ($12.99/mo), no AI detection
- **Market Position**: Churches already using Logos ecosystem
- **Differentiation**: Offline operation, AI detection, no subscription lock-in

**Key Insight**: **None offer AI-powered reference detection** - this is our unique value proposition and primary competitive advantage.

### Appendix F: Bible Version Licensing Summary

**Public Domain** (freely usable):
- King James Version (KJV, 1769)
- Douay-Rheims Bible (DR)
- Young's Literal Translation (YLT)
- Webster's Bible (WBT)
- American Standard Version (ASV, 1901)

**Restricted/Licensed** (requires permission for public display):
- **New International Version (NIV)** - Biblica/Zondervan
  - Attribution: "Scripture quotations taken from The Holy Bible, New International Version® NIV®"
  - License Required: Yes, for public display
  - URL: https://www.biblica.com/permissions/
  
- **English Standard Version (ESV)** - Crossway
  - Attribution: "Scripture quotations are from the ESV® Bible"
  - License Required: Yes, restrictions on length/frequency
  - URL: https://www.crossway.org/permissions/
  
- **New Living Translation (NLT)** - Tyndale House
  - Attribution: "Scripture quotations are taken from the Holy Bible, New Living Translation"
  - License Required: Yes
  - URL: https://www.tyndale.com/permissions
  
- **The Message (MSG)** - NavPress/Eugene Peterson
  - Attribution: "Scripture taken from THE MESSAGE"
  - License Required: Yes, strict limits
  - URL: https://www.navpress.com/permissions
  
- **Amplified Bible Classic (AMPC)** - Lockman Foundation
  - Attribution: "Scripture taken from the Amplified® Bible"
  - License Required: Yes
  - URL: https://www.lockman.org/permissions/
  
- **The Passion Translation (TPT)** - BroadStreet Publishing
  - Attribution: "Scripture quotations marked TPT are from The Passion Translation®"
  - License Required: Yes
  - URL: https://www.thepassiontranslation.com/permissions

**CRITICAL**: 
- **User Responsibility**: Application EULA must clearly state that users are responsible for obtaining appropriate licenses for public display of copyrighted translations.
- **Installation Warning**: Setup wizard displays prominent warning about licensing before version selection.
- **In-App Indicators**: Each version shows licensing status (✓ Public Domain, ⚠️ License Required).
- **Attribution Display**: Copyright notices automatically displayed on projector per version requirements.
- **Legal Review**: All included versions must undergo legal review before bundling with installer.

### Appendix G: System Requirements Detail

**Minimum Specifications**:
- **OS**: Windows 10 (64-bit, version 1903 or later)
- **CPU**: Intel Core i5-6500 @ 3.2 GHz (or AMD Ryzen 5 1600)
- **RAM**: 4GB DDR4
- **Storage**: 2GB free space (HDD acceptable)
- **GPU**: Integrated graphics (Intel HD 530 or equivalent)
- **Display**: 1920x1080 (operator) + 1920x1080 (projector)
- **Audio**: USB audio interface or 3.5mm line-in
- **Ports**: 1x USB-A (audio), 1x HDMI/DisplayPort (projector)
- **Expected Performance**: 4-5 second latency, may struggle with poor audio

**Recommended Specifications**:
- **OS**: Windows 11 (64-bit)
- **CPU**: Intel Core i5-10400 @ 2.9 GHz (or AMD Ryzen 5 3600)
- **RAM**: 8GB DDR4
- **Storage**: 5GB free space (SSD strongly recommended for faster loading)
- **GPU**: Dedicated graphics optional (NVIDIA GTX 1050 or equivalent for 4K)
- **Display**: 1920x1080 (operator) + 3840x2160 (projector)
- **Audio**: USB audio interface (class-compliant, 48kHz preferred)
- **Ports**: 1x USB-A (audio), 1x HDMI 2.0/DisplayPort 1.4 (projector)
- **Expected Performance**: 3-4 second latency, handles poor audio better

**Optimal Specifications** (for large churches with 4K projectors):
- **OS**: Windows 11 (64-bit)
- **CPU**: Intel Core i7-11700 @ 2.5 GHz (or AMD Ryzen 7 5800X)
- **RAM**: 16GB DDR4
- **Storage**: 10GB free space (NVMe SSD)
- **GPU**: NVIDIA RTX 3050 or equivalent (for future GPU acceleration)
- **Display**: 2560x1440 or 4K (operator) + 3840x2160 (projector)
- **Audio**: Professional USB audio interface (Focusrite, PreSonus, etc.)
- **Expected Performance**: 2-3 second latency, excellent with all audio conditions

**Network Requirements**:
- **Installation**: Internet connection for initial model downloads (~500MB one-time)
- **Operation**: No internet required (fully offline)
- **Updates**: Offline installer packages distributed separately via USB/download
- **Telemetry** (optional): Internet for anonymous usage statistics (opt-in only)

**Tested Audio Interfaces**:
- Behringer U-Phoria UMC202HD
- Focusrite Scarlett Solo
- PreSonus AudioBox USB 96
- Native Instruments Komplete Audio 1
- Generic USB audio adapters
- Standard 3.5mm line-in (varies by motherboard)

### Appendix H: Keyboard Shortcuts Reference Card

**QUICK REFERENCE - AI Sermon Assistant**

**Match Navigation**:
- `↑` / `↓` - Select match suggestion
- `←` / `→` - Switch version tabs
- `Enter` - Preview selected match
- `Space` - Push to projector screen

**Display Control**:
- `Esc` - Clear projector (blank screen)
- `Ctrl+Z` - Show previous verse
- `F11` - Toggle projector fullscreen

**Manual Search**:
- `Ctrl+F` - Focus search bar
- Type book name, chapter, verse
- `Enter` - Load verse

**Version Quick-Select**:
- `Ctrl+1` - Display Version 1 (e.g., KJV)
- `Ctrl+2` - Display Version 2 (e.g., NIV)
- `Ctrl+3` - Display Version 3 (e.g., AMPC)
- `Ctrl+4` - Display Version 4 (e.g., TPT)

**Panels**:
- `Ctrl+H` - Toggle history panel
- `Ctrl+,` - Open settings

**TIP**: All shortcuts customizable in Settings!

*(Printable PDF available in Help menu)*

---

## 15. Approval & Sign-off

### Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | Nov 16, 2025 | Product Team | Initial draft |
| 1.0 | Nov 16, 2025 | Product Team | Complete PRD for development |
| 1.1 | Nov 16, 2025 | Product Team | **Integrated feedback: tiered confidence, audio calibration, keyboard shortcuts, extended testing, copyright handling, streaming audio, tutorial system** |

### Stakeholder Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Product Owner | [TBD] | _________ | _____ |
| Technical Lead | [TBD] | _________ | _____ |
| UX Designer | [TBD] | _________ | _____ |
| Church Representative | [TBD] | _________ | _____ |
| Legal Counsel | [TBD] | _________ | _____ |

### Change Management

Changes to this PRD after approval require:
1. Written change request with justification
2. Impact assessment (scope, timeline, budget)
3. Approval from Product Owner and Technical Lead
4. Version update and distribution to stakeholders

**Major Changes in v1.1**:
- ✅ Added tiered confidence display (High/Good/Possible)
- ✅ Added audio calibration wizard and quality monitoring
- ✅ Moved keyboard shortcuts to P0 (v1.0) with customization
- ✅ Extended testing phase from 2 to 3 weeks
- ✅ Added copyright attribution display requirements
- ✅ Specified streaming audio processing to reduce latency
- ✅ Added in-app tutorial and video walkthrough
- ✅ Added operator feedback loop
- ✅ Clarified semantic matching expectations (75% top-100, 60% all verses)
- ✅ Added context window adjustability and improved logic
- ✅ Specified auto-scaling for long verses
- ✅ Decided against auto-display mode (always require confirmation)
- ✅ Added IPC heartbeat monitoring for reliability
- ✅ Added diagnostic export tools
- ✅ Clarified Bible licensing requirements and in-app indicators
- ✅ Added voice fingerprinting as optional enhancement
- ✅ Specified database schema optimizations
- ✅ Added performance targets differentiated by hardware tier

---

## 16. Next Steps

### Immediate Actions (Week 0)
1. ✅ PRD review and approval by stakeholders
2. ☐ **CRITICAL: Legal review of Bible version licensing** (before Phase 1)
3. ☐ Finalize technology stack decisions
4. ☐ Set up development environment (Kilocode workspace)
5. ☐ Acquire licensed Bible text files (KJV, NIV, AMPC, TPT)
6. ☐ Download and test Whisper small.en and all-MiniLM-L6-v2 models
7. ☐ Test alternative embedding models (mpnet-base-v2) for comparison
8. ☐ Recruit beta testers (3-5 churches with diverse AV setups)
9. ☐ Create project GitHub repository with initial structure
10. ☐ Set up issue tracking and project board

### Phase 1 Kickoff (Week 1)
1. ☐ Initialize Git repository and branching strategy
2. ☐ Set up Electron boilerplate with React + Tailwind
3. ☐ Create database schema with FTS5 and copyright metadata
4. ☐ Write Bible import scripts for all 4 versions
5. ☐ Implement basic IPC with contextBridge
6. ☐ First team meeting to align on development approach
7. ☐ Begin Phase 1 development (Foundation & Data)
8. ☐ Set up CI/CD pipeline for automated testing

### Pre-Development Preparation
**Legal & Licensing**:
- [ ] Obtain written permission for NIV inclusion (Biblica)
- [ ] Obtain written permission for AMPC inclusion (Lockman)
- [ ] Obtain written permission for TPT inclusion (BroadStreet)
- [ ] Draft EULA with clear licensing responsibility clauses
- [ ] Create installation wizard licensing warning text
- [ ] Prepare in-app licensing information pages

**Technical Setup**:
- [ ] Set up Windows 10 and Windows 11 test machines
- [ ] Acquire test audio interfaces (3-5 different models)
- [ ] Set up test projector (1080p + 4K if possible)
- [ ] Download pre-trained models and verify checksums
- [ ] Archive models to secure backup location
- [ ] Test Whisper performance on minimum spec hardware

**Design & UX**:
- [ ] Create high-fidelity mockups for operator window
- [ ] Create high-fidelity mockups for projector window
- [ ] Design tutorial overlay steps (5 steps)
- [ ] Record video walkthrough script (3-5 minutes)
- [ ] Design keyboard shortcuts quick reference card
- [ ] Create icon set for UI elements

**Beta Testing**:
- [ ] Identify 3-5 beta churches (varied sizes, AV setups)
- [ ] Create beta testing feedback form
- [ ] Schedule weekly check-ins with beta testers
- [ ] Prepare pre-beta survey (current workflow, pain points)
- [ ] Prepare post-beta survey (satisfaction, suggestions)

### Communication Plan
- **Daily**: Development team standup (15 min, async or sync)
  - What did you complete yesterday?
  - What will you work on today?
  - Any blockers?
  
- **Weekly**: 
  - Internal team sync (1 hour, review progress, plan next week)
  - Beta tester check-in (30 min, gather feedback)
  
- **Bi-weekly**: 
  - Stakeholder demo (working features, 30 min)
  - Design review (UI/UX feedback, 30 min)
  
- **Monthly**: 
  - User testing sessions with beta churches (2 hours)
  - Retrospective (what's working, what needs improvement)
  
- **Ad-hoc**: 
  - Slack/Discord for daily coordination
  - GitHub issues for bug tracking
  - Pull request reviews

### Success Criteria for MVP Launch
- [ ] All P0 (Critical) features implemented and tested
- [ ] All P1 (High) features implemented
- [ ] Performance targets met:
  - 3-4 second latency on recommended hardware
  - 4-5 second latency acceptable on minimum hardware
  - >95% explicit detection accuracy
  - >75% semantic detection accuracy (top-100 verses)
- [ ] Zero critical bugs in production testing
- [ ] Successful deployment at 3+ beta churches
- [ ] 10+ real sermons tested without major issues
- [ ] Positive feedback from operators (>4.5/5) and pastors (>4.0/5)
- [ ] Complete documentation:
  - User manual (installation, setup, operation)
  - Troubleshooting guide
  - Video tutorials
  - Keyboard shortcuts reference
  - Licensing information
- [ ] Windows installer package tested and ready for distribution
- [ ] Legal approval of all included Bible versions
- [ ] Marketing materials prepared (website, screenshots, demo video)

### Risk Monitoring Checklist
**Technical Risks** (check monthly):
- [ ] Whisper transcription accuracy meeting targets?
- [ ] Semantic matching false positive rate <5%?
- [ ] Latency targets achievable on minimum hardware?
- [ ] Audio device compatibility issues discovered?
- [ ] Memory leaks or performance degradation?

**User Experience Risks** (check after each beta test):
- [ ] Operators comfortable with interface?
- [ ] Tutorial effective for new users?
- [ ] Keyboard shortcuts intuitive?
- [ ] Display formatting working across projector types?
- [ ] Any confusion about AI suggestions vs. manual mode?

**Business Risks** (check monthly):
- [ ] Bible licensing on track?
- [ ] Beta churches satisfied with progress?
- [ ] Any competitive threats emerging?
- [ ] Timeline slipping significantly?

### Post-Launch Plan (Weeks 14+)
**Week 14-16: Initial Release & Support**
- Release v1.0 to beta churches
- Monitor for critical bugs (hotfix releases if needed)
- Gather detailed feedback for v1.1 planning
- Create user community (forum, Discord, or similar)
- Respond to support requests within 24 hours

**Week 17-20: v1.1 Planning & Development**
- Prioritize v1.1 features based on user feedback
- Begin development on most-requested enhancements
- Continue supporting v1.0 users
- Expand beta testing to 10+ churches

**Week 21+: Ongoing Development**
- Regular feature releases (v1.x every 6-8 weeks)
- Begin v2.0 planning (multi-language, advanced features)
- Build user community and gather testimonials
- Explore partnerships with church software vendors
- Consider commercial licensing model

---

## 17. Training & Documentation Plan

### 17.1 User Documentation

**Installation Guide** (10 pages):
- System requirements checklist
- Step-by-step installation with screenshots
- Audio device setup instructions
- Projector configuration
- Initial calibration walkthrough
- Troubleshooting common installation issues

**User Manual** (40 pages):
- Quick start guide (2 pages)
- Interface overview with labeled screenshots
- Audio calibration detailed guide
- Manual search tutorial
- AI detection explanation (how it works, what to expect)
- Display customization guide
- Session management
- Keyboard shortcuts reference
- Tips for optimal performance
- FAQ (20+ common questions)

**Troubleshooting Guide** (15 pages):
- Audio issues (device not detected, poor quality, etc.)
- Detection issues (AI not finding verses, false positives)
- Display issues (wrong screen, formatting problems)
- Performance issues (slow, laggy, crashes)
- Diagnostic data export instructions
- When to contact support

**Licensing Guide** (8 pages):
- Overview of Bible translation copyrights
- Public domain vs. licensed versions
- How to obtain public display licenses
- Attribution requirements
- Legal disclaimer
- Links to copyright holders

### 17.2 Video Tutorials

**1. Getting Started (3 minutes)**
- Installation overview
- First launch and setup wizard
- Audio calibration quick test
- Displaying your first verse

**2. Using AI Detection (4 minutes)**
- How AI detection works
- Understanding confidence levels
- Reviewing and selecting matches
- Keyboard shortcuts for speed

**3. Manual Search Mode (2 minutes)**
- When to use manual search
- Finding any verse quickly
- Keyboard navigation tips

**4. Customizing Display (3 minutes)**
- Changing fonts and colors
- Auto-scaling for long verses
- Adding text shadows for readability
- Saving preset configurations

**5. Troubleshooting Audio (2 minutes)**
- Audio quality indicators
- Running calibration wizard
- Common audio problems and solutions

**Total: 14 minutes of video content**

### 17.3 In-App Help

**Contextual Help**:
- Tooltip on every UI element (on hover)
- "?" icon next to complex features (opens help panel)
- Tutorial overlay (first launch, can replay)
- Quick tips displayed during idle time

**Help Menu Items**:
- View Tutorial Again
- Keyboard Shortcuts Reference
- Video Tutorials (opens browser)
- Documentation (opens PDF)
- Check for Updates
- About / System Information
- Report a Bug
- Contact Support

### 17.4 Training Program for Beta Churches

**Pre-Installation** (1 week before):
- Email with system requirements
- Pre-survey to understand current workflow
- Schedule training session

**Installation Day** (1 hour on-site or remote):
- Install software on church computer
- Configure audio input from mixing board
- Run audio calibration with pastor's voice
- Set up projector display
- Test with sample verses
- Customize display formatting to church's preference

**Training Session** (1 hour):
- Walk through interface
- Practice AI detection with recorded sermon clips
- Practice manual search
- Review keyboard shortcuts
- Simulate common scenarios (poor audio, rapid references, etc.)
- Answer questions

**Follow-Up**:
- First Sunday: On-call support during service (phone/text)
- Week 2: Check-in call to gather feedback
- Week 4: Follow-up survey
- Ongoing: Email/Discord support

### 17.5 Support Resources

**Support Channels**:
- Email: support@sermonassistant.com (response within 24 hours)
- Discord Community: Real-time chat, user-to-user help
- GitHub Issues: Bug reports, feature requests
- Documentation Site: Searchable knowledge base
- Video Library: YouTube channel with tutorials

**Support SLA** (Service Level Agreement):
- **Critical bugs** (app crashes, can't start): Response within 4 hours, fix within 48 hours
- **High priority** (feature broken, major inconvenience): Response within 24 hours, fix within 1 week
- **Medium priority** (minor bugs, questions): Response within 48 hours
- **Low priority** (feature requests, suggestions): Acknowledged within 1 week

**Community Building**:
- Monthly user webinars (What's New, Tips & Tricks)
- User spotlight stories (how churches use the app)
- Feature voting (let users prioritize v1.1 features)
- Beta program for early access to new features

---

## 18. Metrics & Analytics Implementation

### 18.1 Telemetry Collection (Opt-In Only)

**CRITICAL**: All telemetry must be:
- Opt-in during setup (NOT opt-out)
- Completely anonymous (no identifiable information)
- Transparent (show exactly what's collected)
- User-controlled (can disable anytime)
- Privacy policy compliant

**What to Collect** (if user opts in):
```javascript
// Session-level metrics (aggregated)
{
  "session_id": "uuid-v4", // Random, no user identification
  "app_version": "1.0.0",
  "os_version": "Windows 11",
  "hardware_tier": "minimum|recommended|optimal",
  "duration_minutes": 45,
  "total_references": 15,
  "explicit_detections": 10,
  "semantic_detections": 3,
  "manual_searches": 2,
  "average_confidence": 91.2,
  "average_latency_ms": 3200,
  "operator_feedbacks": {
    "correct": 12,
    "incorrect": 1
  },
  "audio_quality_distribution": {
    "good": 80,
    "fair": 15,
    "poor": 5
  }
}

// What NOT to collect
❌ Sermon transcripts
❌ Preacher names
❌ Church names
❌ Specific verse selections (privacy)
❌ IP addresses
❌ User identifiable information
```

**Privacy Dashboard** (in Settings):
- Toggle telemetry on/off
- View exactly what's been sent (JSON preview)
- Clear all collected data
- Link to privacy policy

### 18.2 Local Analytics (Always Enabled)

**Operator Dashboard** (in app):
- Personal stats (not shared):
  - Total sessions run
  - Most-referenced books
  - Detection accuracy over time
  - Average latency
  - Keyboard shortcut usage
  - Time saved vs. manual mode estimate

**Session Reports**:
- References displayed per session
- Detection method breakdown
- Confidence score distribution
- Timeline of references (visual graph)
- Export as PDF for pastor review

### 18.3 A/B Testing Framework (Future)

For testing improvements to detection algorithms:
- Serve different confidence thresholds to different users
- Measure which settings produce best operator satisfaction
- Implement winner as new default
- Always user-controlled (can opt out)

---

## 19. Monetization & Distribution Strategy

### 19.1 Pricing Model (TBD)

**Option 1: Free + Premium**
- **Free**: All core features, 4 Bible versions, community support
- **Premium** ($99/year or $299 lifetime):
  - Unlimited Bible versions
  - Priority support
  - Early access to new features
  - Custom branding (church logo on display)
  - Advanced analytics

**Option 2: One-Time Purchase**
- $199 one-time purchase
- All features included
- Free updates for 1 year
- Optional support plan ($49/year)

**Option 3: Freemium with Church Tiers**
- **Free**: Basic features, 2 Bible versions
- **Small Church** ($9.99/month): All features, 4 versions, up to 500 attendees
- **Medium Church** ($19.99/month): All features, unlimited versions, up to 1500 attendees
- **Large Church** ($39.99/month): All features, priority support, unlimited attendees, custom branding

**Recommendation**: Start with **Option 2** (one-time purchase) for simplicity, minimal friction for churches, and to build user base. Consider subscription in v2.0 if ongoing costs justify it.

### 19.2 Distribution Channels

**Direct Download**:
- Official website with download link
- Windows installer (.exe or .msi)
- Automatic update checks (manual installation)

**Church Software Directories**:
- Submit to Church Tech Today
- List on Ministry Tech
- Partner with church IT service providers

**Partnerships**:
- Integrate with existing church management software
- Partner with AV equipment vendors
- Collaborate with Bible software companies (Logos, Olive Tree)

**App Stores** (Future):
- Microsoft Store (requires UWP conversion or packaging)
- Consideration for v1.1+

### 19.3 Marketing Strategy

**Launch Prep**:
- Professional website with demo video
- Case studies from beta churches
- Press kit (screenshots, logo, description)
- Social media presence (Twitter, Facebook, Instagram)

**Launch Campaign**:
- Blog post: "Introducing AI Sermon Assistant"
- Submit to Product Hunt, Hacker News
- Reach out to church tech blogs/podcasts
- Free trial for first 100 churches (if applicable)
- Testimonials from pastors and operators

**Ongoing**:
- Content marketing (how-to guides, best practices)
- YouTube channel with tutorials
- Email newsletter (monthly updates)
- User testimonials and case studies
- Conference presence (church tech conferences)

---

## 20. Ethical Considerations

### 20.1 Responsible AI

**Transparency**:
- Clearly communicate that AI is assistive, not authoritative
- Always show confidence scores
- Never hide when AI makes suggestions vs. human actions
- Explain how detection works (in documentation)

**Human Oversight**:
- NEVER auto-display without operator confirmation
- Human always has final say
- Easy to override or dismiss AI suggestions
- Manual mode always available

**Accuracy & Safety**:
- Extensive testing to minimize false positives
- Clear visual indicators for low-confidence matches
- Operator feedback loop to improve over time
- Fail gracefully (manual mode as fallback)

**Privacy**:
- All processing local (no cloud)
- Sermon transcripts never transmitted
- Telemetry opt-in only and anonymous
- Clear privacy policy

### 20.2 Copyright Respect

**Bible Translations**:
- Comply with all copyright requirements
- Display required attributions
- Make licensing information transparent
- User responsible for obtaining licenses (clear EULA)
- Provide links and guidance for license acquisition

**Respecting Scripture**:
- Accurate representation of verses
- No alterations or editorializing
- Clear indication of version differences
- Respect for theological significance

### 20.3 Accessibility & Inclusion

**Design for All**:
- Keyboard-first interface (motor impairments)
- High contrast mode (visual impairments)
- Clear error messages (cognitive accessibility)
- Adjustable font sizes
- WCAG 2.0 AA compliance

**Affordability**:
- Pricing accessible to small churches
- Consider free tier or discounts for under-resourced churches
- Open-source components give back to community

**Language & Culture**:
- v2.0: Spanish support (large Spanish-speaking church population)
- Future: More languages based on demand
- Respect for different Christian traditions and worship styles

### 20.4 Theological Neutrality

**No Bias**:
- Include versions from different theological traditions
- Don't favor one translation over others (alphabetical ordering)
- No commentary or interpretation (just scripture)
- Respect denominational differences

**User Control**:
- Churches choose their preferred versions
- No forced defaults based on theology
- Respect for local church authority

---

## 21. Long-Term Vision (v3.0 and Beyond)

### 21.1 Platform Expansion

**Multi-Platform**:
- macOS version (many churches use Mac)
- Linux version (open-source churches)
- Web version (cloud-based, no installation)
- iPad app (portable control for worship leaders)

**Integration Ecosystem**:
- Plugin API for church software integration
- Streaming platform integration (YouTube, Facebook Live)
- Presentation software plugins (PowerPoint, Keynote)
- Church management system sync (Planning Center, CCB)

### 21.2 Advanced AI Features

**Sermon Analysis**:
- Automatic sermon outline generation
- Key themes and topics identification
- Cross-reference suggestions based on context
- Illustration database integration

**Predictive Display**:
- Anticipate next verse based on sermon flow
- Pre-load likely references in background
- Context-aware suggestions (e.g., "Often followed by...")

**Multi-Language NLP**:
- Real-time translation for multilingual churches
- Detect references in Spanish, French, Portuguese, etc.
- Cross-language semantic matching

**Voice Cloning** (with permission):
- Calibrate to specific pastor's voice
- Improved accuracy for multi-speaker services
- Filter out guest speakers vs. lead pastor

### 21.3 Community & Collaboration

**Sermon Sharing**:
- Anonymous sermon log sharing (opt-in)
- "Top 10 verses this month" across all users
- Trending topics in churches
- Sermon series templates

**Collaborative Features**:
- Multiple operators (backup control)
- Remote preview (pastor sees what's displayed)
- Real-time notes and annotations
- Sermon review and feedback tools

**Open Source Components**:
- Core detection engine as open-source library
- Community contributions for improvements
- Plugin ecosystem for custom features
- Educational resource for AI/NLP students

### 21.4 Expanded Content Types

**Beyond Scripture**:
- Hymn and worship lyrics display
- Creeds and liturgical texts
- Prayer guides and responsive readings
- Announcements and bulletin integration

**Multimedia**:
- Image overlays on verses (background photos)
- Video clip integration
- Animated text transitions
- 3D rendering for special occasions

**Accessibility Features**:
- Sign language interpretation overlay
- Audio description for visually impaired
- Closed captioning generation
- Multi-language subtitles

---

## 22. Conclusion

This Product Requirements Document outlines a comprehensive plan for building an AI-powered sermon assistant that respects the sacred nature of worship while leveraging modern technology to reduce operator burden and improve congregation experience.

### Key Differentiators:
1. **AI-Powered Detection**: Unique in the market, saves significant time
2. **Human-in-the-Loop**: Maintains operator control and accuracy
3. **Offline-First**: Reliable operation without internet dependency
4. **Respectful Licensing**: Clear guidance and compliance tools
5. **Keyboard-First UX**: Efficient workflow for experienced operators
6. **Comprehensive Training**: Documentation, videos, and in-app tutorials

### Core Values:
- **Accuracy**: Scripture deserves perfect representation
- **Transparency**: Users understand how AI works
- **Privacy**: Sermon content stays local
- **Accessibility**: Available to churches of all sizes
- **Respect**: For copyright, theology, and user intelligence

### Success Factors:
- **Technical Excellence**: Meeting performance targets consistently
- **User-Centered Design**: Solving real operator pain points
- **Legal Compliance**: Proper licensing and attribution
- **Community Building**: Engaged users who provide feedback
- **Continuous Improvement**: Regular updates based on real-world usage

This PRD provides a solid foundation for a 13-week development cycle to deliver v1.0, with a clear roadmap for future enhancements. The product addresses a genuine need in churches worldwide, using AI responsibly to enhance—not replace—human judgment in the service of worship.

**Let's build something meaningful.**

---

**End of Product Requirements Document v1.1**

*This is a living document. Updates will be made as the project progresses based on user feedback, technical discoveries, and evolving requirements. All changes tracked in version control.*

**Next Review Date**: End of Phase 2 (Week 3) - Assess initial development progress and adjust timeline if needed.