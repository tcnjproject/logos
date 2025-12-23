## Data Models (Baseline From PRD)

These are the minimum data structures referenced in the PRD. Some fields are fully specified there; anything not explicit is marked **TBD**.

### Detected reference (operator-visible event)
- `DetectedReference`
  - `id: string`
  - `timestamp: Date`
  - `reference: { book: string; chapter: number; verseStart: number; verseEnd?: number }`
  - `confidence: number`
  - `confidenceTier: 'high' | 'good' | 'possible'`
  - `method: 'explicit' | 'semantic' | 'manual'`
  - `transcriptSnippet: string`
  - `matches: VerseMatch[]`
  - `operatorFeedback?: 'correct' | 'incorrect'`

### Verse match (candidate / selected)
- `VerseMatch`
  - `verseId: number`
  - `book: string`
  - `chapter: number`
  - `verse: number`
  - `versions: Record<string, string>` (e.g. `{ KJV: "..." }`)
  - `copyright: Record<string, string>`

### Display settings (projector)
- `DisplaySettings` (selected fields from PRD)
  - `font: { family; size; weight; color; autoScale; shadow? }`
  - `background: { color; image? }`
  - `reference: { size; position; visible }`
  - `copyright: { size; position; visible }`
  - `alignment: 'left' | 'center' | 'right'`
  - `lineSpacing: number`

### Sessions + logs
- `SermonSession`
  - `id: number`
  - `date: Date`
  - `title?: string`
  - `preacher?: string`
  - `logs: SessionLog[]`
  - `stats: { totalReferences; explicitDetections; semanticDetections; manualSearches; averageConfidence }`
- `SessionLog`
  - `id: number`
  - `timestamp: Date`
  - `reference: string` (e.g., `"John 3:16"`)
  - `versionDisplayed: string`
  - `detectionMethod: string`
  - `confidence: number`
  - `transcriptSnippet: string`
  - `operatorFeedback?: 'correct' | 'incorrect'`

### Bible version metadata
- `BibleVersion`
  - `id: number`
  - `code: string` (e.g., `"NIV"`)
  - `name: string`
  - `copyrightYear: number`
  - `copyrightHolder: string`
  - `licenseType: 'public_domain' | 'restricted' | 'licensed'`
  - `licenseUrl?: string`
  - `displayAttribution: string`
  - `requiresLicense: boolean`

### Audio calibration
- `AudioProfile`
  - `deviceId: string`
  - `deviceName: string`
  - `sensitivity: number`
  - `noiseThreshold: number`
  - `vadEnabled: boolean`
  - `qualityScore: 'good' | 'fair' | 'poor'`
  - `calibrationDate: Date`

### Keyboard shortcuts
- `KeyboardShortcuts` (names from PRD; exact parsing/serialization is **TBD**)
  - `pushToScreen`, `clearScreen`, `navigateUp`, `navigateDown`, `navigateLeft`, `navigateRight`
  - `focusSearch`, `toggleHistory`
  - `quickVersion1`, `quickVersion2`, `quickVersion3`, `quickVersion4`
  - `previousVerse`, `toggleFullscreen`

### Database (SQLite + FTS5)
- PRD references:
  - FTS5 indexing on verse text.
  - Fast lookup indexes by `book_id` + `chapter`.
  - `versions` table holds licensing/attribution metadata.
- **TBD**: full normalized schema for books/chapters/verses and how multiple translations map (shared verse IDs vs per-version rows).

