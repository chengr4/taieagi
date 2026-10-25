//! Tool for preprocessing the MOE Taiwanese dictionary CSV dataset into a compact binary index.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use taieagi::{Candidate, LexiconEntry};

/// Parses a CSV line respecting quoted fields.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Checks if a string contains any Japanese Kana characters.
fn contains_kana(s: &str) -> bool {
    s.chars().any(|c| ('\u{3040}'..='\u{30FF}').contains(&c))
}

/// Converts Japanese Hiragana and Katakana characters to standard Romaji.
/// Non-kana characters are preserved as-is.
fn kana_to_romaji(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        let romaji = match c {
            // Vowels
            'あ' | 'ア' | 'ぁ' | 'ァ' => "a",
            'い' | 'イ' | 'ぃ' | 'ィ' => "i",
            'う' | 'ウ' | 'ぅ' | 'ゥ' => "u",
            'え' | 'エ' | 'ぇ' | 'ェ' => "e",
            'お' | 'オ' | 'ぉ' | 'ォ' => "o",
            // K
            'か' | 'カ' => "ka",
            'き' | 'キ' => "ki",
            'く' | 'ク' => "ku",
            'け' | 'ケ' => "ke",
            'こ' | 'コ' => "ko",
            // G
            'が' | 'ガ' => "ga",
            'ぎ' | 'ギ' => "gi",
            'ぐ' | 'グ' => "gu",
            'げ' | 'ゲ' => "ge",
            'ご' | 'ゴ' => "go",
            // S
            'さ' | 'サ' => "sa",
            'し' | 'シ' => "si",
            'す' | 'ス' => "su",
            'せ' | 'セ' => "se",
            'そ' | 'ソ' => "so",
            // Z / J
            'ざ' | 'ザ' => "za",
            'じ' | 'ジ' => "zi",
            'ず' | 'ズ' => "zu",
            'ぜ' | 'ゼ' => "ze",
            'ぞ' | 'ゾ' => "zo",
            // T
            'た' | 'タ' => "ta",
            'ち' | 'チ' => "ti",
            'つ' | 'ツ' => "tu",
            'て' | 'テ' => "te",
            'と' | 'ト' => "to",
            // D
            'だ' | 'ダ' => "da",
            'ぢ' | 'ヂ' => "di",
            'づ' | 'ヅ' => "du",
            'で' | 'デ' => "de",
            'ど' | 'ド' => "do",
            // N
            'な' | 'ナ' => "na",
            'に' | 'ニ' => "ni",
            'ぬ' | 'ヌ' => "nu",
            'ね' | 'ネ' => "ne",
            'の' | 'ノ' => "no",
            // H
            'は' | 'ハ' => "ha",
            'ひ' | 'ヒ' => "hi",
            'ふ' | 'フ' => "hu",
            'へ' | 'ヘ' => "he",
            'ほ' | 'ホ' => "ho",
            // B
            'ば' | 'バ' => "ba",
            'び' | 'ビ' => "bi",
            'ぶ' | 'ブ' => "bu",
            'べ' | 'ベ' => "be",
            'ぼ' | 'ボ' => "bo",
            // P
            'ぱ' | 'パ' => "pa",
            'ぴ' | 'ピ' => "pi",
            'ぷ' | 'プ' => "pu",
            'ぺ' | 'ペ' => "pe",
            'ぽ' | 'ポ' => "po",
            // M
            'ま' | 'マ' => "ma",
            'み' | 'ミ' => "mi",
            'む' | 'ム' => "mu",
            'め' | 'メ' => "me",
            'も' | 'モ' => "mo",
            // Y
            'や' | 'ヤ' | 'ゃ' | 'ャ' => "ya",
            'ゆ' | 'ユ' | 'ゅ' | 'ュ' => "yu",
            'よ' | 'ヨ' | 'ょ' | 'ョ' => "yo",
            // R
            'ら' | 'ラ' => "ra",
            'り' | 'リ' => "ri",
            'る' | 'ル' => "ru",
            'れ' | 'レ' => "re",
            'ろ' | 'ロ' => "ro",
            // W
            'わ' | 'ワ' | 'ゎ' | 'ヮ' => "wa",
            'を' | 'ヲ' => "wo",
            // N / Coda
            'ん' | 'ン' => "n",
            'っ' | 'ッ' => "t",
            'ー' => "",
            _ => {
                result.push(c);
                continue;
            }
        };
        result.push_str(romaji);
    }
    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let input_path = args.get(1).map_or("assets/moe_raw.csv", String::as_str);
    let output_path = args.get(2).map_or("assets/dict.compact", String::as_str);

    println!("📖 Reading source dataset from: {}", input_path);
    let file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(file);

    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut total_lines = 0;
    let mut skipped_empty = 0;

    for (idx, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        total_lines += 1;
        if idx == 0 && line.starts_with("主編碼") {
            continue; // Skip CSV header
        }

        let fields = parse_csv_line(&line);
        if fields.len() < 4 {
            continue;
        }

        let attr = &fields[1];
        let hanzi = &fields[2];
        let raw_tailo_col = &fields[3];

        if hanzi.is_empty() || raw_tailo_col.is_empty() {
            skipped_empty += 1;
            continue;
        }

        // Base weight by entry attribute
        let base_weight: u32 = match attr.as_str() {
            "1" => 100, // Main primary lemma
            "2" => 90,  // Secondary lemma / single character
            "5" => 85,  // Lexical dialect variants
            "22" => 80, // Surnames
            _ => 70,    // Names, landmarks, appendices, proverbs
        };

        // Split multiple pronunciations separated by '/'
        for reading in raw_tailo_col.split('/') {
            let reading_trimmed = reading.trim();
            if reading_trimmed.is_empty() {
                continue;
            }

            // Convert Japanese Kana if present (Option A), otherwise borrow slice directly
            let (plain_tailo, tones) = if contains_kana(reading_trimmed) {
                let romaji = kana_to_romaji(reading_trimmed);
                taieagi::parse_tailo_word(&romaji)
            } else {
                taieagi::parse_tailo_word(reading_trimmed)
            };
            if plain_tailo.is_empty() {
                continue;
            }

            // Deduplicate (plain_tailo, hanzi)
            let dedup_key = format!("{}\t{}", plain_tailo, hanzi);
            if !seen.insert(dedup_key) {
                continue;
            }

            entries.push(LexiconEntry {
                tailo: plain_tailo,
                hanzi: hanzi.to_string(),
                weight: base_weight,
                tones,
            });
        }
    }

    let extra_vocab_paths = ["assets/supplement.csv", "assets/lyrics.csv"];
    for path_str in &extra_vocab_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }
        let file = File::open(path)?;
        for (idx, line_res) in BufReader::new(file).lines().enumerate() {
            let line = line_res?;
            if idx == 0 && line.starts_with("詞目") {
                continue;
            }
            let fields = parse_csv_line(&line);
            if fields.len() < 2 || fields[0].is_empty() || fields[1].is_empty() {
                continue;
            }
            let hanzi = &fields[0];
            for reading in fields[1].split('/') {
                let (plain_tailo, tones) = taieagi::parse_tailo_word(reading.trim());
                if plain_tailo.is_empty() {
                    continue;
                }
                let dedup_key = format!("{}\t{}", plain_tailo, hanzi);
                if seen.insert(dedup_key) {
                    entries.push(LexiconEntry {
                        tailo: plain_tailo,
                        hanzi: hanzi.to_string(),
                        weight: 100,
                        tones,
                    });
                }
            }
        }
    }

    println!(
        "📊 Processed {} rows (skipped {} empty rows)",
        total_lines, skipped_empty
    );
    println!(
        "📦 Collected {} unique pronunciation entries",
        entries.len()
    );

    println!("💾 Encoding into compact binary format...");
    let compact_bytes = taieagi::encode_compact(&entries);
    let size_kb = compact_bytes.len() as f64 / 1024.0;
    println!(
        "📏 Total binary size: {:.2} KB ({} bytes)",
        size_kb,
        compact_bytes.len()
    );

    let mut out_file = File::create(Path::new(output_path))?;
    out_file.write_all(&compact_bytes)?;
    println!(
        "✨ Successfully wrote compact dictionary to: {}",
        output_path
    );

    let hints_paths = ["assets/hints.csv", "assets/lyrics_hints.csv"];
    let mut hint_pairs = Vec::new();
    for hints_path_str in &hints_paths {
        let hints_path = Path::new(hints_path_str);
        if !hints_path.exists() {
            continue;
        }
        let hints_file = File::open(hints_path)?;
        for (idx, line_res) in BufReader::new(hints_file).lines().enumerate() {
            let line = line_res?;
            if idx == 0 && line.starts_with("詞目") {
                continue;
            }
            let fields = parse_csv_line(&line);
            if fields.len() < 3
                || fields[0].is_empty()
                || fields[1].is_empty()
                || fields[2].is_empty()
            {
                continue;
            }
            let hanzi = &fields[0];
            let (plain_tailo, tones) = taieagi::parse_tailo_word(fields[1].trim());
            if plain_tailo.is_empty() {
                continue;
            }
            let candidate = Candidate {
                hanzi: hanzi.to_string(),
                tailo: plain_tailo,
                tones,
            };
            for typo in fields[2].split('/') {
                let typo_trimmed = typo.trim();
                if !typo_trimmed.is_empty() {
                    hint_pairs.push((typo_trimmed.to_string(), candidate.clone()));
                }
            }
        }
    }
    if !hint_pairs.is_empty() {
        let hints_bytes = taieagi::encode_hints_compact(&hint_pairs);
        let hints_out_path = "assets/hints.compact";
        let mut hints_out = File::create(Path::new(hints_out_path))?;
        hints_out.write_all(&hints_bytes)?;
        println!(
            "✨ Successfully wrote compact hints ({} bytes) to: {}",
            hints_bytes.len(),
            hints_out_path
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kana_to_romaji() {
        assert_eq!(kana_to_romaji("ながし"), "nagasi");
        assert_eq!(kana_to_romaji("ナガシ"), "nagasi");
        assert_eq!(kana_to_romaji("あたま"), "atama");
        assert_eq!(kana_to_romaji("karaoke"), "karaoke");
    }
}
