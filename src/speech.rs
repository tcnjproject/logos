//! Audio conversion and lifecycle boundary for interchangeable Rhema STT providers.
use rhema_stt::{SttProvider, TranscriptEvent, WhisperProvider};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::{Mutex, mpsc};

pub type EventReceiver = Arc<Mutex<mpsc::Receiver<TranscriptEvent>>>;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

pub struct SpeechSession {
    pub id: u64,
    pub events: EventReceiver,
    provider: Arc<dyn SttProvider>,
}

impl SpeechSession {
    pub fn local() -> Result<(Self, crossbeam_channel::Sender<Vec<i16>>), String> {
        let path = std::env::var_os("LOGOS_WHISPER_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("models/whisper/ggml-base.en.bin"));
        if !path.is_file() {
            return Err(format!(
                "Whisper model missing: {}. Set LOGOS_WHISPER_MODEL.",
                path.display()
            ));
        }
        let threads = std::thread::available_parallelism()
            .map(|n| n.get().min(4) as i32)
            .unwrap_or(2);
        Self::with_provider(Arc::new(WhisperProvider::new(
            path,
            Some("en".into()),
            threads,
        )))
    }

    pub fn with_provider(
        provider: Arc<dyn SttProvider>,
    ) -> Result<(Self, crossbeam_channel::Sender<Vec<i16>>), String> {
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(64);
        let (event_tx, event_rx) = mpsc::channel(64);
        let worker = provider.clone();
        std::thread::Builder::new()
            .name("logos-stt".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let _ = event_tx.blocking_send(TranscriptEvent::Error(error.to_string()));
                        return;
                    }
                };
                runtime.block_on(async move {
                    if let Err(error) = worker.start(audio_rx, event_tx.clone()).await {
                        let _ = event_tx
                            .send(TranscriptEvent::Error(error.to_string()))
                            .await;
                    }
                    let _ = event_tx.send(TranscriptEvent::Disconnected).await;
                });
            })
            .map_err(|e| format!("Could not start STT worker: {e}"))?;
        Ok((
            Self {
                id: NEXT_SESSION.fetch_add(1, Ordering::Relaxed),
                events: Arc::new(Mutex::new(event_rx)),
                provider,
            },
            audio_tx,
        ))
    }
}

impl Drop for SpeechSession {
    fn drop(&mut self) {
        self.provider.stop();
    }
}

/// Stateful averaging resampler: interleaved device audio → mono 16kHz PCM.
/// Keeps channel/phase state across callbacks, emits the 1024-sample frames VAD expects.
pub struct PcmConverter {
    sample_rate: u32,
    channels: usize,
    channel_samples: Vec<f32>,
    phase: u32,
    sum: f32,
    count: u32,
    frame: Vec<i16>,
}

impl PcmConverter {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            channels: usize::from(channels.max(1)),
            channel_samples: Vec::new(),
            phase: 0,
            sum: 0.0,
            count: 0,
            frame: Vec::with_capacity(1024),
        }
    }
    pub fn push(&mut self, input: &[f32], mut emit: impl FnMut(Vec<i16>)) {
        for &sample in input {
            self.channel_samples.push(sample);
            if self.channel_samples.len() != self.channels {
                continue;
            }
            self.sum += self.channel_samples.iter().sum::<f32>() / self.channels as f32;
            self.count += 1;
            self.channel_samples.clear();
            self.phase += 16_000;
            let value = (self.sum / self.count as f32).clamp(-1.0, 1.0);
            while self.phase >= self.sample_rate {
                self.phase -= self.sample_rate;
                self.frame.push((value * 32767.0).round() as i16);
                if self.frame.len() == 1024 {
                    emit(std::mem::replace(&mut self.frame, Vec::with_capacity(1024)));
                }
            }
            if self.count > 0 && self.phase < 16_000 {
                self.sum = 0.0;
                self.count = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    #[tokio::test]
    #[ignore = "requires the runtime Whisper model"]
    async fn local_whisper_model_loads() {
        let (session, _audio) = SpeechSession::local().unwrap();
        let event = tokio::time::timeout(std::time::Duration::from_secs(30), async {
            session.events.lock().await.recv().await
        })
        .await
        .unwrap()
        .unwrap();
        assert!(
            matches!(event, TranscriptEvent::Connected),
            "Whisper startup failed: {event:?}"
        );
    }
    struct FakeProvider {
        stopped: AtomicBool,
    }
    #[async_trait::async_trait]
    impl SttProvider for FakeProvider {
        async fn start(
            &self,
            audio: crossbeam_channel::Receiver<Vec<i16>>,
            events: mpsc::Sender<TranscriptEvent>,
        ) -> Result<(), rhema_stt::SttError> {
            let samples = audio
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap();
            assert_eq!(samples.len(), 1024);
            events
                .send(TranscriptEvent::Final {
                    transcript: "John 3:16".into(),
                    words: vec![],
                    confidence: 1.0,
                    speech_final: true,
                })
                .await
                .unwrap();
            Ok(())
        }
        fn stop(&self) {
            self.stopped.store(true, Ordering::SeqCst);
        }
        fn name(&self) -> &'static str {
            "test"
        }
    }
    #[tokio::test]
    async fn provider_events_and_cancellation_cross_the_session_boundary() {
        let provider = Arc::new(FakeProvider {
            stopped: AtomicBool::new(false),
        });
        let (session, audio) = SpeechSession::with_provider(provider.clone()).unwrap();
        audio.send(vec![0; 1024]).unwrap();
        let event = tokio::time::timeout(std::time::Duration::from_secs(3), async {
            session.events.lock().await.recv().await
        })
        .await
        .unwrap()
        .unwrap();
        assert!(
            matches!(event, TranscriptEvent::Final { transcript, .. } if transcript == "John 3:16")
        );
        drop(session);
        assert!(provider.stopped.load(Ordering::SeqCst));
    }
    #[test]
    fn resamples_stereo_across_callback_boundaries() {
        for rate in [8_000, 16_000, 44_100, 48_000] {
            let mut converter = PcmConverter::new(rate, 2);
            let input = vec![0.5; rate as usize * 2];
            let mut output = Vec::new();
            for chunk in input.chunks(333) {
                converter.push(chunk, |frame| output.extend(frame));
            }
            assert_eq!(output.len(), 15 * 1024);
            assert!(output.iter().all(|&v| (v - 16384).abs() <= 1));
        }
    }
    #[test]
    fn downmixes_opposite_channels() {
        let mut converter = PcmConverter::new(16_000, 2);
        let input: Vec<f32> = (0..1024).flat_map(|_| [0.8, -0.8]).collect();
        converter.push(&input, |frame| assert!(frame.iter().all(|&v| v == 0)));
    }
}
