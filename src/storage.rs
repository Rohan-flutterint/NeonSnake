use std::{env, fs, path::PathBuf};

use crate::game::HighScoreEntry;

fn score_file_path() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neonsnake_scores")
}

pub fn load_high_scores() -> Vec<HighScoreEntry> {
    let Ok(contents) = fs::read_to_string(score_file_path()) else {
        return Vec::new();
    };

    let mut entries = contents
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(',');
            let score = parts.next()?.trim().parse().ok()?;
            let length = parts.next()?.trim().parse().ok()?;
            Some(HighScoreEntry { score, length })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.length.cmp(&a.length)));
    entries.truncate(5);
    entries
}

pub fn save_high_scores(entries: &[HighScoreEntry]) -> std::io::Result<()> {
    let body = entries
        .iter()
        .map(|entry| format!("{},{}", entry.score, entry.length))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(score_file_path(), format!("{body}\n"))
}

pub fn register_high_score(entries: &mut Vec<HighScoreEntry>, new_entry: HighScoreEntry) {
    entries.push(new_entry);
    entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.length.cmp(&a.length)));
    entries.truncate(5);
}
