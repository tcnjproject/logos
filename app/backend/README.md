# Python implementation of some PRD features

## Dependencies
- Python 3.9 and later
- Run: `pip install -r backend/requirements.txt`.
  - TODO: Create a script for automatic environment setup 

### transcribe.py
- `transcribe.py` implements FR 1.3 (Real-time Transcription).
  It uses whisper large-v3 model to achieve >90% transcription accuracy.
   - **TODO**: Improve latency. On my PC, latency is > 2 seconds probably because I am not using a GPU. I will add GPU support and test for latency improvement.
- Run: `python .\transcribe.py --model large-v3`
 - Note: the script will download the `large-v3` model to your PC (about 2GB in size)
- This script should not be started manually. It is used by the `server.py` script for automatic speech to text transcription.
  
### server.py
- Based on FastAPI. This script implements the backend API server that communicates with the frontend.
- Run:python backend/server.py
- Several API endpoints will be available to the frontend once this server is started.