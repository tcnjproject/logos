# Python implementation of some PRD features

## Dependencies
- Python 3.9 and later

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
  It uses whisper large-v3 model to achieve >90% transcription accuracy.
  Improved latency with `parakeet-tdt-0.6b-v3` model running on onnx.
- Run: `python .\transcribe.py`
 - Note: the script will download the `parakeet-tdt-0.6b-v3` model to your PC

TODO:  
### api_server.py
- Based on FastAPI. This script implements the backend API server that communicates with the frontend.
- Run with `uvicorn api_server:app --reload`
- Several API endpoints will be available to the frontend once this server is started.
  - `POST /v1/speech-to-text` to transcribe an audio
    * `curl -X POST "http://127.0.0.1:8000/v1/speech-to-text" -F "file=@C:\Users\Test\output2.mp3" `
  - `POST /v1/speech-to-text/realtime` for transcribing realtime with audio inputs from the microphone.
    * `curl -N -X POST "http://127.0.0.1:8000/v1/speech-to-text/realtime"`


