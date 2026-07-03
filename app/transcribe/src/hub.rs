//! Downloads Parakeet ONNX model files directly from the Hugging Face Hub via the
//! [`hf-hub`](https://docs.rs/hf-hub) crate, reusing the same on-disk cache layout
//! (`models--<org>--<name>/snapshots/<hash>/...` under `$HF_HOME`, or
//! `~/.cache/huggingface/hub` by default) that Python's `huggingface_hub` uses. Files already
//! present in the cache are reused rather than re-downloaded.

use std::path::PathBuf;

use hf_hub::api::sync::ApiBuilder;

use crate::error::Result;

/// The Hugging Face model repo this crate defaults to when `--model-id` isn't overridden.
pub const DEFAULT_MODEL_ID: &str = "istupakov/parakeet-tdt-0.6b-v3-onnx";

/// Files required by [`crate::Transcriber::from_model_dir`].
const REQUIRED_FILES: [&str; 4] = [
    "encoder-model.int8.onnx",
    "decoder_joint-model.int8.onnx",
    "vocab.txt",
    "config.json",
];

/// Downloads (or reuses the cached copy of) `model_id` from the Hugging Face Hub, returning the
/// local directory containing its files. Only files missing from the cache are fetched. Honors
/// the `HF_HOME` and `HF_ENDPOINT` environment variables, matching Python's `huggingface_hub`.
pub fn download_model(model_id: &str) -> Result<PathBuf> {
    let api = ApiBuilder::from_env().build()?;
    let repo = api.model(model_id.to_string());

    let mut dir = None;
    for file in REQUIRED_FILES {
        let path = repo.get(file)?;
        dir.get_or_insert_with(|| {
            path.parent()
                .expect("cached file path always has a parent directory")
                .to_path_buf()
        });
    }

    Ok(dir.expect("REQUIRED_FILES is non-empty"))
}
