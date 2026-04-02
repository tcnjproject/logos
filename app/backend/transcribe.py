# TCNJ AI/ML Group
# Source: https://github.com/davabase/whisper_real_time
#
#F R-1.3: Real-time Transcription
# Priority: P0 (Critical)
# Description: System shall transcribe spoken words to text with <2 second latency
#     Acceptance Criteria:
#     Uses Whisper small.en model for English speech-to-text
#     Processes audio in streaming/chunked mode to reduce buffering delay
#     Displays live transcript in operator window
#     Maintains transcription accuracy >90% for clear speech
#     Handles religious terminology correctly (Bible names, theological terms)


import argparse
import os, sys
import numpy as np
import speech_recognition as sr
# import whisper
import torch

from datetime import datetime, timedelta
from queue import Queue
from time import sleep
from sys import platform
from pathlib import Path

# from faster_whisper import WhisperModel

# for distil-whisper
# from transformers import AutoModelForSpeechSeq2Seq, AutoProcessor, pipeline

ROOT_DIR = Path(os.getcwd()).as_posix()
os.environ["HF_HOME"] = ROOT_DIR + "/models"
os.environ["HF_HUB_CACHE"] = ROOT_DIR + "/models"
os.environ["HF_HUB_DISABLE_SYMLINKS_WARNING"] = "true"
if sys.platform == "win32":
    os.environ["PATH"] = ROOT_DIR + f";{ROOT_DIR}/ffmpeg;" + os.environ["PATH"]

MODEL_CONFIG = {
        "parakeet-tdt-0.6b-v3": {
            "hf_id": "nemo-parakeet-tdt-0.6b-v3",
            "quantization": "int8",
            "description": "INT8 (fastest)"
        },
    }

# Model cache for lazy loading
model_cache = {}

def get_parakeet_model():
    """
    Get or load the parakeet model with lazy loading and caching.
        
    Returns:
        Loaded ASR model instance
    """
    model_name = "parakeet-tdt-0.6b-v3"
    
    # Return cached model if available
    if model_name in model_cache:
        print(f"Using cached model: {model_name}")
        return model_cache[model_name]
    
    # Load new model
    print(f"Loading model: {model_name}")
    config = MODEL_CONFIG[model_name]
    
    try:
        import onnxruntime as ort
        
        # Reuse providers from startup
        available_providers = ort.get_available_providers()
        providers_to_try = []
        if "TensorrtExecutionProvider" in available_providers:
            providers_to_try.append("TensorrtExecutionProvider")
        if "CUDAExecutionProvider" in available_providers:
            providers_to_try.append("CUDAExecutionProvider")
        providers_to_try.append("CPUExecutionProvider")
        
        # Configure session options
        sess_options = ort.SessionOptions()
        sess_options.intra_op_num_threads = 4
        sess_options.inter_op_num_threads = 1
        sess_options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        
        model = onnx_asr.load_model(
            config["hf_id"],
            quantization=config["quantization"],
            providers=providers_to_try,
            sess_options=sess_options,
        ).with_timestamps()
        
        # Cache the loaded model
        model_cache[model_name] = model
        print(f"Model {model_name} loaded successfully")
        
        return model
    except Exception as e:
        print(f"❌ Failed to load model {model_name}: {e}")
        import traceback
        traceback.print_exc()
        # Try to return the default cached model if available
        if "parakeet-tdt-0.6b-v3" in model_cache:
            print(f"⚠️ Falling back to cached default model")
            return model_cache["parakeet-tdt-0.6b-v3"]
        else:
            # If we can't even get the default, we have a serious problem
            raise RuntimeError(f"Failed to load model {model_name} and no fallback available")

def load_parakeet_model():
    try:
        print("\nInitializing ONNX Runtime...")
        import onnx_asr
        import onnxruntime as ort
        
        # Detect available providers
        available_providers = ort.get_available_providers()
        print(f"Available providers: {available_providers}")
        
        # Priority: Tensorrt, CUDA, CPU
        providers_to_try = []
        if "TensorrtExecutionProvider" in available_providers:
            providers_to_try.append("TensorrtExecutionProvider")
        if "CUDAExecutionProvider" in available_providers:
            providers_to_try.append("CUDAExecutionProvider")
        providers_to_try.append("CPUExecutionProvider")
        
        print(f"Using providers: {providers_to_try}")

        # Load default INT8 model at startup
        print("\nLoading default Parakeet TDT 0.6B V3 ONNX model with INT8 quantization...")
        
        # Configure session options for optimal CPU performance
        sess_options = ort.SessionOptions()
        sess_options.intra_op_num_threads = 4  # Match Waitress threads
        sess_options.inter_op_num_threads = 1
        sess_options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL

        default_config = MODEL_CONFIG["parakeet-tdt-0.6b-v3"]
        asr_model = onnx_asr.load_model(
            default_config["hf_id"],
            quantization=default_config["quantization"],
            providers=providers_to_try,
            sess_options=sess_options,
        ).with_timestamps()
        
        # Cache the default model
        model_cache["parakeet-tdt-0.6b-v3"] = asr_model
        
        print("Default model loaded successfully with CPU optimization!")
    except Exception as e:
        print(f"❌ Model loading failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit()

    print("=" * 50)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", default="parakeet-tdt-0.6b-v3", help="Model to use",
                        choices=["parakeet-tdt-0.6b-v3", "large-v3", "faster-whisper", "distil-whisper"])
    # parser.add_argument("--non_english", action='store_true',
    #                     help="Don't use the english model.")
    parser.add_argument("--energy_threshold", default=1000,
                        help="Energy level for mic to detect.", type=int)
    parser.add_argument("--record_timeout", default=2,
                        help="How real time the recording is in seconds.", type=float)
    parser.add_argument("--phrase_timeout", default=3,
                        help="How much empty space between recordings before we "
                             "consider it a new line in the transcription.", type=float)
    if 'linux' in platform:
        parser.add_argument("--default_microphone", default='pulse',
                            help="Default microphone name for SpeechRecognition. "
                                 "Run this with 'list' to view available Microphones.", type=str)
    args = parser.parse_args()

    # The last time a recording was retrieved from the queue.
    phrase_time = None
    # Thread safe Queue for passing data from the threaded recording callback.
    data_queue = Queue()
    # Bytes object which holds audio data for the current phrase
    phrase_bytes = bytes()
    # We use SpeechRecognizer to record our audio because it has a nice feature where it can detect when speech ends.
    recorder = sr.Recognizer()
    recorder.energy_threshold = args.energy_threshold
    # Definitely do this, dynamic energy compensation lowers the energy threshold dramatically to a point where the SpeechRecognizer never stops recording.
    recorder.dynamic_energy_threshold = False

    # Important for linux users.
    # Prevents permanent application hang and crash by using the wrong Microphone
    if 'linux' in platform:
        mic_name = args.default_microphone
        if not mic_name or mic_name == 'list':
            print("Available microphone devices are: ")
            for index, name in enumerate(sr.Microphone.list_microphone_names()):
                print(f"Microphone with name \"{name}\" found")
            return
        else:
            for index, name in enumerate(sr.Microphone.list_microphone_names()):
                if mic_name in name:
                    source = sr.Microphone(sample_rate=16000, device_index=index)
                    break
    else:
        source = sr.Microphone(sample_rate=16000)

    # Check for GPU
    device = "cuda:0" if torch.cuda.is_available() else "cpu"
    torch_dtype = torch.float16 if torch.cuda.is_available() else torch.float32
    print(f"Using device: {device}")
    
    # TODO: Try this model on a GPU-enabled PC
    if args.model == "faster-whisper":
        model = "large-v3"
        # run on CPU with INT8
        audio_model = WhisperModel(model, device=device, compute_type="int8")
    # TODO: Try this model on a GPU-enabled PC
    elif args.model == "distil-whisper":
        model = "distil-whisper/distil-large-v3"
        audio_model = AutoModelForSpeechSeq2Seq.from_pretrained(model, torch_dtype=torch_dtype, low_cpu_mem_usage=True, use_safetensors=True)
        audio_model.to(device)
        processor = AutoProcessor.from_pretrained(model)
    elif args.model == "parakeet-tdt-0.6b-v3":
        load_parakeet_model()
        audio_model = get_parakeet_model()
    else:
        model = args.model
        audio_model = whisper.load_model(model)

    record_timeout = args.record_timeout
    phrase_timeout = args.phrase_timeout

    transcription = ['']

    with source:
        recorder.adjust_for_ambient_noise(source)

    def record_callback(_, audio:sr.AudioData) -> None:
        """
        Threaded callback function to receive audio data when recordings finish.
        audio: An AudioData containing the recorded bytes.
        """
        # Grab the raw bytes and push it into the thread safe queue.
        data = audio.get_raw_data()
        data_queue.put(data)

    # Create a background thread that will pass us raw audio bytes.
    # We could do this manually but SpeechRecognizer provides a nice helper.
    recorder.listen_in_background(source, record_callback, phrase_time_limit=record_timeout)

    # Cue the user that we're ready to go.
    print("Model loaded.\n")

    while True:
        try:
            now = datetime.utcnow()
            # Pull raw recorded audio from the queue.
            if not data_queue.empty():
                phrase_complete = False
                # If enough time has passed between recordings, consider the phrase complete.
                # Clear the current working audio buffer to start over with the new data.
                if phrase_time and now - phrase_time > timedelta(seconds=phrase_timeout):
                    phrase_bytes = bytes()
                    phrase_complete = True
                # This is the last time we received new audio data from the queue.
                phrase_time = now
                
                # Combine audio data from queue
                audio_data = b''.join(data_queue.queue)
                data_queue.queue.clear()

                # Add the new audio data to the accumulated data for this phrase
                phrase_bytes += audio_data

                # Convert in-ram buffer to something the model can use directly without needing a temp file.
                # Convert data from 16 bit wide integers to floating point with a width of 32 bits.
                # Clamp the audio stream frequency to a PCM wavelength compatible default of 32768hz max.
                audio_np = np.frombuffer(phrase_bytes, dtype=np.int16).astype(np.float32) / 32768.0

                # Read the transcription.
                if args.model == "faster-whisper":
                    segments, _ = audio_model.transcribe(audio_np, beam_size=5)
                    text = " ".join([segment.text for segment in segments]).strip()
                elif args.model == "distil-whisper":
                    inputs = processor(audio_np, sampling_rate=16000, return_tensors="pt")
                    inputs = {k: v.to(device) for k, v in inputs.items()}
                    generated_ids = audio_model.generate(**inputs)
                    text = processor.batch_decode(generated_ids, skip_special_tokens=True)[0].strip()
                elif args.model == "parakeet-tdt-0.6b-v3":
                    result = audio_model.recognize(audio_np)
                    text = result.text.strip()
                else:
                    result = audio_model.transcribe(audio_np, fp16=torch.cuda.is_available())
                    text = result['text'].strip()

                # If we detected a pause between recordings, add a new item to our transcription.
                # Otherwise edit the existing one.
                if phrase_complete:
                    transcription.append(text)
                else:
                    transcription[-1] = text

                # Clear the console to reprint the updated transcription.
                os.system('cls' if os.name=='nt' else 'clear')
                for line in transcription:
                    print(line)
                # Flush stdout.
                print('', end='', flush=True)
            else:
                # Infinite loops are bad for processors, must sleep.
                sleep(0.25)
        except KeyboardInterrupt:
            break

    print("\n\nTranscription:")
    for line in transcription:
        print(line)


if __name__ == "__main__":
    main()