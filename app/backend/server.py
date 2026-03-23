from fastapi import FastAPI

app = FastAPI()

@app.post("/speech-to-text")
async def transcribe(data: dict):
    return {"message": "Data received", "received_data": data}

@app.get("/about")
async def about_server():
    return {"Name": "W2M Server", "Version": "1.0.0", "Description": "Backend server for W2M application"}