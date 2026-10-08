// TCNJ AI/ML Group
//! Path resolution helpers for assets, the SQLite Bible database, and models.
//!
//! Resolves resources gracefully across development (`cargo run`), release
//! binaries, and packaged application bundles (e.g. macOS `.app`).

use std::path::{Path, PathBuf};

/// Get the project root in development, or fallback to current directory.
pub fn dev_project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Candidate directories to look for resources.
fn search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    // 1. Explicit environment override
    if let Ok(dir) = std::env::var("LOGOS_ROOT") {
        roots.push(PathBuf::from(dir));
    }

    // 2. Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }

    // 3. Next to the executing binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            roots.push(exe_dir.to_path_buf());

            // 4. macOS bundle: Contents/MacOS/logos -> Contents/Resources
            if let Some(contents) = exe_dir.parent() {
                let resources = contents.join("Resources");
                if resources.is_dir() {
                    roots.push(resources);
                }
            }
        }
    }

    // 5. Cargo manifest dir in development
    roots.push(dev_project_root());

    roots
}

/// Find a resource file by relative subpath (e.g. "assets/logos.png").
pub fn find_resource<P: AsRef<Path>>(subpath: P) -> Option<PathBuf> {
    let subpath = subpath.as_ref();

    for root in search_roots() {
        let candidate = root.join(subpath);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Resolve an asset file path (e.g. "logos.png", "help.svg").
pub fn asset_path(filename: &str) -> PathBuf {
    if let Ok(dir) = std::env::var("LOGOS_ASSETS_DIR") {
        let path = PathBuf::from(dir).join(filename);
        if path.exists() {
            return path;
        }
    }

    find_resource(Path::new("assets").join(filename))
        .unwrap_or_else(|| PathBuf::from("assets").join(filename))
}

/// Resolve the path to the `rhema.db` SQLite database.
pub fn bible_db_path() -> PathBuf {
    if let Ok(custom) = std::env::var("LOGOS_BIBLE_DB") {
        let path = PathBuf::from(custom);
        if path.exists() {
            return path;
        }
    }

    find_resource("data/rhema.db")
        .unwrap_or_else(|| dev_project_root().join("data").join("rhema.db"))
}

/// Resolve the path to the Whisper GGML model.
pub fn whisper_model_path(model_name: &str) -> PathBuf {
    if let Ok(custom) = std::env::var("LOGOS_WHISPER_MODEL") {
        let path = PathBuf::from(custom);
        if path.exists() {
            return path;
        }
    }

    let fname = if model_name == "large-v3-turbo" {
        "ggml-large-v3-turbo-q8_0.bin".to_string()
    } else {
        format!("ggml-{model_name}.bin")
    };

    find_resource(Path::new("models").join("whisper").join(&fname))
        .unwrap_or_else(|| dev_project_root().join("models").join("whisper").join(&fname))
}

/// Candidate system fonts for text rasterization in NDI broadcasting.
pub fn system_font_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &[
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\calibri.ttf",
            r"C:\Windows\Fonts\arial.ttf",
        ]
    }

    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/SFPro.ttf",
        ]
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        ]
    }
}
