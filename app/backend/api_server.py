from fastapi import FastAPI

# from pydantic import BaseModel
# from typing import List, Optional, Any
import uuid

app = FastAPI()


# WSS /v1/speech-to-text/realtime
# Realtime speech-to-text transcription service. 
# This WebSocket API enables streaming audio input and receiving transcription results
#
# URL	wss://127.0.0.1:8000/v1/speech-to-text/realtime
# Method	GET
# Status	101 Switching Protocols
#
# Event Flow
#   Audio chunks are sent as input_audio_chunk messages
#   Transcription results are streamed back in various formats (partial, committed, with timestamps)
#   Supports manual commit or VAD-based automatic commit strategies
#   Authentication is done either by providing a valid API key in the xi-api-key header or by providing a valid token in the token query parameter. 
#   Tokens can be generated from the single use token endpoint. Use tokens if you want to transcribe audio from the client side.
@app.websocket("/v1/speech-to-text/realtime")
async def realtime_transcription(websocket):
    await websocket.accept()
    while True:
        data = await websocket.receive_text()
        # Placeholder for processing the received audio data and generating a transcription
        transcript = "This is a sample real-time transcript."
        await websocket.send_text(transcript)

# POST /v1/speech-to-text
# Transcribe an audio file sent in the request body and return the transcript.
# curl -X POST "http://127.0.0.1:8000/v1/speech-to-text?enable_logging=true" \
    #  -H "Content-Type: multipart/form-data" \
    #  -F model_id="parakeet-tdt-0.6b-v3" \
    #  -F file=@<file1>
@app.post("/v1/speech-to-text")
async def transcribe_audio(enable_logging: bool = False):
    # Placeholder static transcription result for transformation
    return {
        "language_code": "en",
        "text": "",
        "transcription_id": await generate_unique_id(),
    }

async def generate_unique_id() -> str:
    # Placeholder unique ID generation logic
    return str(uuid.uuid4())

# GET /v1/speech-to-text/transcripts/:transcription_id
# Retrieve a previously generated transcript by its ID.
# curl http://127.0.0.1:8000/v1/speech-to-text/transcripts/transcription_id
# @app.get("/v1/speech-to-text/transcripts/{transcription_id}")
# async def get_transcript(transcription_id: str):
#     return {"transcription_id": transcription_id, "transcript": "This is a sample transcript."}


# DELETE /v1/speech-to-text/transcripts/:transcription_id
# Delete a previously generated transcript by its ID.
# curl -X DELETE http://127.0.0.1:8000/v1/speech-to-text/transcripts/transcription_id
# @app.delete("/v1/speech-to-text/transcripts/{transcription_id}")
# async def delete_transcript(transcription_id: str):
#     return {"transcription_id": transcription_id, "status": "deleted"}