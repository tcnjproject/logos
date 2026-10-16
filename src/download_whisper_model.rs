// Copyright (c) 2026 The Covenant Nation Jesutedo (TCNJ) <AI/ML Group>
//
// This file is part of the logos crate, which is licensed under the MIT or Apache-2.0 license. 
// See the LICENSE file in the root of this repository for details.
//
// Ported from https://github.com/openbezal/rhema/blob/main/data/download-whisper-model.ts

use futures_util::StreamExt;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
const MODEL_FILENAME: &str = "ggml-base.en.bin";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadOutcome {
    AlreadyPresent,
    Downloaded,
}

pub async fn download_model(
    mut report_progress: impl FnMut(u64, Option<u64>),
) -> Result<DownloadOutcome, Box<dyn Error + Send + Sync>> {
    let model_dir = PathBuf::from("models/whisper");
    let model_path = model_dir.join(MODEL_FILENAME);
    if model_path.is_file() {
        let size = fs::metadata(&model_path)?.len();
        report_progress(size, Some(size));
        return Ok(DownloadOutcome::AlreadyPresent);
    }

    fs::create_dir_all(&model_dir)?;
    let temporary_path = model_dir.join(format!("{MODEL_FILENAME}.part.bin"));
    let response = reqwest::get(MODEL_URL).await?.error_for_status()?;
    let total_size = response.content_length();
    report_progress(0, total_size);

    let mut file = tokio::fs::File::create(&temporary_path).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        report_progress(downloaded, total_size);
    }

    file.flush().await?;
    drop(file);
    if total_size.is_some_and(|total| total != downloaded) {
        return Err(format!(
            "Incomplete model download: received {downloaded} of {} bytes",
            total_size.unwrap()
        )
        .into());
    }
    fs::rename(&temporary_path, &model_path)?;
    report_progress(downloaded, Some(total_size.unwrap_or(downloaded)));

    Ok(DownloadOutcome::Downloaded)
}
