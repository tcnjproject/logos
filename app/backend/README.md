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
- `uvicorn server:app --reload`
- When done, deactiavte the virtual environment with `deactivate`

### transcribe.py
- `transcribe.py` implements FR 1.3 (Real-time Transcription).
  It uses whisper large-v3 model to achieve >90% transcription accuracy.
   - **TODO**: Improve latency. On my PC, latency is > 2 seconds probably because I am not using a GPU. I will add GPU support and test for latency improvement.
- Run: `python .\transcribe.py --model large-v3`
 - Note: the script will download the `large-v3` model to your PC (about 2GB in size)
- This script should not be started manually. It is used by the `server.py` script for automatic speech to text transcription.
  
### server.py
- Based on FastAPI. This script implements the backend API server that communicates with the frontend.
- Run with `uvicorn server:app --reload`
- Several API endpoints will be available to the frontend once this server is started.