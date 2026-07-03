//! Vocabulary loading and SentencePiece-style detokenization.
//!
//! `vocab.txt` maps token id -> token text, one `"<token> <id>"` pair per line, with the
//! SentencePiece word-start marker `▁` (U+2581) already meaning "this token starts a new word".
//! We replace it with a literal space on load (matching `onnx_asr`'s NeMo loader) and then, when
//! decoding, join adjacent token pieces and collapse the resulting whitespace with the same regex
//! `onnx_asr` uses so that punctuation directly follows the preceding word instead of getting its
//! own leading space (e.g. `"Hello , world !"` -> `"Hello, world!"`).
use std::fs;
use std::path::Path;

use regex::Regex;

use crate::error::{Result, TranscribeError};

const WORD_START_MARKER: char = '\u{2581}';

pub struct Vocab {
    tokens: Vec<String>,
    /// Id of the `<blk>` (CTC/RNNT blank) token, if present.
    pub blank_idx: usize,
    detokenize_pattern: Regex,
}

impl Vocab {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path).map_err(|source| TranscribeError::ModelFile {
            path: path.to_path_buf(),
            source,
        })?;

        let mut tokens: Vec<Option<String>> = Vec::new();
        for (line_no, line) in contents.lines().enumerate() {
            let (token, id) = line.rsplit_once(' ').ok_or_else(|| TranscribeError::Vocab {
                line: line_no + 1,
                text: line.to_string(),
            })?;
            let id: usize = id.parse().map_err(|_| TranscribeError::Vocab {
                line: line_no + 1,
                text: line.to_string(),
            })?;
            let token = token.replace(WORD_START_MARKER, " ");

            if id >= tokens.len() {
                tokens.resize(id + 1, None);
            }
            tokens[id] = Some(token);
        }

        let tokens: Vec<String> = tokens
            .into_iter()
            .enumerate()
            .map(|(id, t)| {
                t.ok_or_else(|| TranscribeError::Vocab {
                    line: 0,
                    text: format!("missing vocab entry for id {id}"),
                })
            })
            .collect::<Result<_>>()?;

        let blank_idx = tokens
            .iter()
            .position(|t| t == "<blk>")
            .ok_or_else(|| TranscribeError::Vocab {
                line: 0,
                text: "vocabulary has no <blk> token".to_string(),
            })?;

        // Mirrors onnx_asr's `_AsrWithDecoding.DECODE_SPACE_PATTERN`:
        //   \A\s     - drop a leading space at the very start of the text
        //   \s\B     - drop a space that isn't at a word boundary (glued to the next piece)
        //   (\s)\b   - collapse a space at a word boundary down to a single space
        let detokenize_pattern = Regex::new(r"\A\s|\s\B|(\s)\b").expect("static regex is valid");

        Ok(Self {
            tokens,
            blank_idx,
            detokenize_pattern,
        })
    }

    pub fn vocab_size(&self) -> usize {
        self.tokens.len()
    }

    pub fn token(&self, id: usize) -> &str {
        &self.tokens[id]
    }

    /// Join emitted token ids into human-readable text.
    pub fn decode(&self, ids: &[i32]) -> String {
        let joined: String = ids.iter().map(|&id| self.token(id as usize)).collect();
        self.detokenize_pattern
            .replace_all(&joined, |caps: &regex::Captures| {
                if caps.get(1).is_some() {
                    " "
                } else {
                    ""
                }
            })
            .into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocab_from_pairs(pairs: &[(&str, usize)]) -> Vocab {
        let max_id = pairs.iter().map(|(_, id)| *id).max().unwrap_or(0);
        let mut tokens = vec![String::new(); max_id + 1];
        for (tok, id) in pairs {
            tokens[*id] = tok.replace(WORD_START_MARKER, " ");
        }
        Vocab {
            tokens,
            blank_idx: max_id,
            detokenize_pattern: Regex::new(r"\A\s|\s\B|(\s)\b").unwrap(),
        }
    }

    #[test]
    fn detokenizes_like_sentencepiece() {
        let vocab = vocab_from_pairs(&[
            ("\u{2581}Hello", 0),
            ("\u{2581}world", 1),
            ("!", 2),
        ]);
        assert_eq!(vocab.decode(&[0, 1, 2]), "Hello world!");
    }

    #[test]
    fn keeps_single_space_between_words() {
        let vocab = vocab_from_pairs(&[
            ("\u{2581}The", 0),
            ("\u{2581}quick", 1),
            ("\u{2581}brown", 2),
            ("\u{2581}fox", 3),
            (".", 4),
        ]);
        assert_eq!(vocab.decode(&[0, 1, 2, 3, 4]), "The quick brown fox.");
    }

    #[test]
    fn joins_apostrophes_without_extra_space() {
        let vocab = vocab_from_pairs(&[
            ("\u{2581}It", 0),
            ("'", 1),
            ("s", 2),
            ("\u{2581}a", 3),
            ("\u{2581}test", 4),
            (",", 5),
            ("\u{2581}really", 6),
            (".", 7),
        ]);
        assert_eq!(vocab.decode(&[0, 1, 2, 3, 4, 5, 6, 7]), "It's a test, really.");
    }
}
