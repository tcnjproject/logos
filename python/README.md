# Python implementation of some PRD features

### FR 1.3
- `fr_1_3.py` implements FR 1.3 (Real-time Transcription).
  It uses whisper large-v3 model to achieve >90% transcription accuracy.
   - **TODO**: improve latency. On my PC, latency is > 2seconds probably because I am not using a GPU. I will test with add GPU support and test for latency improvement

## Dependencies
- Python 3.9 
- Create a python virtual environment yourself and run `pip install -r requirements`.
  - TODO: Create a script for automatic environment setup 

## Run script
`python .\fr_1_3.py --model large-v3 --non_english`
 - Note: the script will download the `large-v3` model to your PC (about 2GB in size)
  
