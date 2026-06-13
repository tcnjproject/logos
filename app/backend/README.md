# Backend services

## Dependencies
- At least Python 3.10

## Setup
Setup the backend server in a virtual environment with the following steps:
- `cd .\app\backend`
- `python -m venv .venv`
- On Windows (powershell): `.venv\Scripts\Activate.ps1`. On Linux/MacOS: `source .venv/bin/activate`
- `python -m pip install --upgrade pip`
- `pip install -r requirements.txt`
- `uvicorn api_server:app --reload`
- When done, deactiavte the virtual environment with `deactivate`

### transcribe.py
- `transcribe.py` implements FR 1.3 (Real-time Transcription).
  It uses `parakeet-tdt-0.6b-v3` model with [onnxruntime](https://onnxruntime.ai/) to achieve >90% transcription accuracy and near real-time latency.
- Run: `python .\transcribe.py`
- Note: the script will download the `parakeet-tdt-0.6b-v3` model to your PC and use it offline.

TODO:  
### api_server.py
- Based on FastAPI. This script implements the backend API server that communicates with the frontend.
- Run with `uvicorn api_server:app --reload`
- Several API endpoints will be available to the frontend once this server is started.
  - `POST /v1/speech-to-text` to transcribe an audio
    * `curl -X POST "http://127.0.0.1:8000/v1/speech-to-text" -F "file=@C:\Users\Test\output2.mp3" `
  - `POST /v1/speech-to-text/realtime` for transcribing realtime with audio inputs from the microphone.
    * `curl -N -X POST "http://127.0.0.1:8000/v1/speech-to-text/realtime"`

### bible_search.py
- High-performance scripture search engine
- Example:
  ```python
    >>> from bible_search import *
    >>> engine = ScriptureSearchEngine(use_cache=True)
    >>> engine.load_scripture("bible/nlt.json")
    >>> verse = engine.search_by_reference("Deuteronomy 10:9")
    >>> verse.text
    'That is why the Levites have no share of property or possession of land among the other Israelite tribes. The LORD himself is their special possession, as the LORD your God told them.)'
    >>>
  ```

# Logos — Dear PyGui

Live Bible verse display application built with Dear PyGui.

## Install & run

```bash
pip install dearpygui
python main.py
```

## File structure

```
main.py       — UI, layout, callbacks, theme
data.py       — Enums (Translation, SearchMode, TourStep) + sample verses
state.py      — AppState dataclass (search, queue, live verse, timer)
```

## Features

- **Loading screen** — animated orange progress bar
- **Live Transcript** — start/stop mic, scrollable transcript area, audio level bars
- **Program Preview** — black screen showing the staged verse
- **Bible Search** — Book search (reference) and Context search (theme) tabs
- **Translation picker** — NIV / KJV / ESV / NLT / NASB dropdown popup
- **Verse results** — Present or Queue each result
- **Live Display** — Go Live toggle (green/grey), broadcasts verse to output screen
- **Verse Queue** — add, present, and remove queued verses
- **Recent Detections** — AI-detected verses from speech
- **Onboarding tour** — 8-step tooltip overlay, Next / Back / Skip
- **Update banner** — bottom-right notification
- **Countdown timer** — REMAINING: 1:00:00, ticks down live
- **Menu bar** — File / Edit / View / Audio / Window / Help

## Wiring in Parakeet STT

`state.py` has `is_transcribing` and `transcript_text`. From your WebSocket
receiver thread, update `state.transcript_text` and call:

```python
if dpg.does_item_exist("transcript_live_text"):
    dpg.configure_item("transcript_live_text",
                       default_value=state.transcript_text)
```

Detected verse references should be appended to `state.recent_detections`
and trigger `_refresh_detections_ui()`.