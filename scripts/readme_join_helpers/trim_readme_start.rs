use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    // Require path as first argument (no fallback). Print usage and exit if missing.
    let arg = match env::args().nth(1) {
        Some(a) => a,
        None => {
            eprintln!("Usage: trim_readme_start <path-to-Readme_Start.md>");
            std::process::exit(2);
        }
    };
    let path = Path::new(&arg);
    if !path.exists() {
        eprintln!("Path does not exist: {}", path.display());
        std::process::exit(3);
    }

    // If REPO_ROOT is provided via env, strip it from logs to keep output concise
    let repo_root = env::var("REPO_ROOT").ok();
    let display_path = match &repo_root {
        Some(root) if path.starts_with(root) => {
            // make path relative to repo root
            match path.strip_prefix(root) {
                Ok(p) => p.display().to_string().trim_start_matches('/').to_string(),
                Err(_) => path.display().to_string(),
            }
        }
        _ => path.display().to_string(),
    };

    let s = fs::read_to_string(path)?;
    // Trim trailing whitespace and write the trimmed content exactly
    let trimmed = s.trim_end_matches(|c: char| c.is_whitespace());
    fs::write(path, trimmed.as_bytes())?;
    println!("Trimmed: {} ({} -> {} bytes)", display_path, s.len(), trimmed.len());
    Ok(())
}
