//! Prefix index keyed by Tâi-lô (TL) romanization, mapping typed letters to Taiwanese hanzi.

use crate::tone::{self, Tone};
use std::collections::{HashMap, HashSet};

/// One Taiwanese lexicon item: a Tâi-lô romanization and its hanzi form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconEntry {
    /// Tâi-lô romanization, e.g. `"lim-tsui"`. Multi-syllable entries separate
    /// syllables with `-` or whitespace; typed input does not need a separator.
    pub tailo: String,
    /// Taiwanese hanzi output, e.g. `"啉水"`.
    pub hanzi: String,
    /// Frequency weight; higher values rank first.
    pub weight: u32,
    /// Per-syllable tones. May be empty if tones are unknown.
    pub tones: Vec<Tone>,
}

/// A candidate shown in the selection window (hanzi only).
///
/// `tailo` and `tones` are not displayed; they let the composer format the
/// committed text according to the output mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub hanzi: String,
    pub tailo: String,
    pub tones: Vec<Tone>,
}

impl From<&LexiconEntry> for Candidate {
    fn from(entry: &LexiconEntry) -> Self {
        Self {
            hanzi: entry.hanzi.clone(),
            tailo: entry.tailo.clone(),
            tones: entry.tones.clone(),
        }
    }
}

/// Maximum number of orthographic hints returned by a prefix lookup.
const MAX_HINTS: usize = 3;

struct TrieNode<T> {
    children: HashMap<char, TrieNode<T>>,
    entries: Vec<T>,
}

impl<T> Default for TrieNode<T> {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            entries: Vec::new(),
        }
    }
}

impl<T> TrieNode<T> {
    fn collect_into<'a>(&'a self, out: &mut Vec<&'a T>) {
        out.extend(self.entries.iter());
        for child in self.children.values() {
            child.collect_into(out);
        }
    }
}

static COMPACT_DICT: &[u8] = include_bytes!("../assets/dict.compact");
static COMPACT_HINTS: &[u8] = include_bytes!("../assets/hints.compact");

/// Prefix index over Tâi-lô keys and orthographic hints for common colloquial
/// misspellings, both backed by a HashMap trie.
#[derive(Default)]
pub(crate) struct TailoIndex {
    lexicon_root: TrieNode<LexiconEntry>,
    hints_root: TrieNode<Candidate>,
}

/// Normalizes a romanization into a trie key: lowercase letters only,
/// with hyphens and tone digits removed.
fn normalize(tailo: &str) -> String {
    tailo
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

impl TailoIndex {
    /// Loads the embedded MOE Taiwanese lexicon and orthographic hints.
    pub(crate) fn built_in() -> Self {
        let mut index = Self::load_compact(COMPACT_DICT);
        index.load_hints_compact(COMPACT_HINTS);
        index
    }

    /// Inserts an entry. Entries whose normalized key is empty are ignored.
    pub(crate) fn insert(&mut self, entry: LexiconEntry) {
        let key = normalize(&entry.tailo);
        if key.is_empty() {
            return;
        }
        let mut node = &mut self.lexicon_root;
        for c in key.chars() {
            node = node.children.entry(c).or_default();
        }
        node.entries.push(entry);
    }

    /// Inserts a read-only hint mapping `typo` to `candidate`, deduplicating identical `(hanzi, tailo)` pairs.
    pub(crate) fn insert_hint(&mut self, typo: &str, candidate: Candidate) {
        let key = normalize(typo);
        if key.is_empty() {
            return;
        }
        let mut node = &mut self.hints_root;
        for c in key.chars() {
            node = node.children.entry(c).or_default();
        }
        if !node
            .entries
            .iter()
            .any(|c| c.hanzi == candidate.hanzi && c.tailo == candidate.tailo)
        {
            node.entries.push(candidate);
        }
    }

    /// Looks up up to [`MAX_HINTS`] read-only hint candidates by prefix match,
    /// prioritizing exact matches before shorter prefix extensions.
    pub(crate) fn lookup_hints(&self, input: &str) -> Vec<Candidate> {
        if input.is_empty() || !input.chars().all(|c| c.is_ascii_alphabetic() || c == '-') {
            return Vec::new();
        }
        let key = normalize(input);
        if key.is_empty() {
            return Vec::new();
        }

        let mut node = &self.hints_root;
        for c in key.chars() {
            match node.children.get(&c) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }

        let mut prefix_entries = Vec::new();
        for child in node.children.values() {
            child.collect_into(&mut prefix_entries);
        }
        prefix_entries.sort_by(|a, b| {
            a.tones
                .len()
                .cmp(&b.tones.len())
                .then_with(|| a.hanzi.len().cmp(&b.hanzi.len()))
                .then_with(|| a.hanzi.cmp(&b.hanzi))
                .then_with(|| a.tailo.cmp(&b.tailo))
        });

        let mut seen = HashSet::new();
        node.entries
            .iter()
            .chain(prefix_entries)
            .filter(|c| seen.insert((c.hanzi.as_str(), c.tailo.as_str())))
            .take(MAX_HINTS)
            .cloned()
            .collect()
    }

    /// Looks up candidates by typed input, prioritizing exact matches over prefix
    /// extensions, sorted by weight (descending), and deduplicated by hanzi.
    ///
    /// - Stored hyphens are stripped on insert, so `lim-tsui` is found by `limtsui` or `lim-tsui`.
    /// - Without a trailing tone digit, exact key matches appear first, followed by
    ///   longer prefix matches (enabling compound-word suggestions).
    /// - With a trailing tone digit (e.g. `tsui2` or `lim-tsui2`), only entries whose key
    ///   equals the input and whose last syllable carries that tone are returned.
    /// - Input containing anything other than letters, hyphens, and one optional
    ///   trailing tone digit yields no candidates.
    pub(crate) fn lookup(&self, input: &str) -> Vec<Candidate> {
        let (base, tone) = tone::split_syllable_and_tone(input);
        if !base.chars().all(|c| c.is_ascii_alphabetic() || c == '-') {
            return Vec::new();
        }
        let key = normalize(base);
        if key.is_empty() {
            return Vec::new();
        }

        let mut node = &self.lexicon_root;
        for c in key.chars() {
            match node.children.get(&c) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }

        let mut matched: Vec<(&LexiconEntry, bool)> = Vec::new();
        match tone {
            Some(t) => matched.extend(
                node.entries
                    .iter()
                    .filter(|e| e.tones.last() == Some(&t))
                    .map(|e| (e, true)),
            ),
            None => {
                matched.extend(node.entries.iter().map(|e| (e, true)));
                let mut prefix_entries = Vec::new();
                for child in node.children.values() {
                    child.collect_into(&mut prefix_entries);
                }
                matched.extend(prefix_entries.into_iter().map(|e| (e, false)));
            }
        }

        // Exact key matches rank before prefix extensions; tie-break by weight (desc),
        // then fewer syllables, shorter hanzi, and finally hanzi codepoint order.
        matched.sort_by(|(a, a_exact), (b, b_exact)| {
            b_exact
                .cmp(a_exact)
                .then_with(|| b.weight.cmp(&a.weight))
                .then_with(|| a.tones.len().cmp(&b.tones.len()))
                .then_with(|| a.hanzi.len().cmp(&b.hanzi.len()))
                .then_with(|| a.hanzi.cmp(&b.hanzi))
        });

        let mut seen_hanzi = HashSet::new();
        matched
            .into_iter()
            .filter(|(e, _)| seen_hanzi.insert(e.hanzi.as_str()))
            .map(|(e, _)| Candidate::from(e))
            .collect()
    }

    /// Loads entries directly from a compact binary byte slice.
    fn load_compact(bytes: &[u8]) -> Self {
        let mut index = Self::default();
        for entry in decode_compact(bytes) {
            index.insert(entry);
        }
        index
    }

    /// Loads orthographic hints from a compact binary byte slice into this index.
    fn load_hints_compact(&mut self, bytes: &[u8]) {
        for (typo, candidate) in decode_hints_compact(bytes) {
            self.insert_hint(&typo, candidate);
        }
    }
}

/// Encodes a slice of [`LexiconEntry`] into a compact binary format.
pub fn encode_compact(entries: &[LexiconEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    for entry in entries {
        let tailo_bytes = entry.tailo.as_bytes();
        let hanzi_bytes = entry.hanzi.as_bytes();
        if tailo_bytes.len() > 255 || hanzi_bytes.len() > 255 || entry.tones.len() > 255 {
            continue;
        }

        out.push(tailo_bytes.len() as u8);
        out.extend_from_slice(tailo_bytes);

        out.push(hanzi_bytes.len() as u8);
        out.extend_from_slice(hanzi_bytes);

        let weight = (entry.weight.min(u16::MAX as u32) as u16).to_le_bytes();
        out.extend_from_slice(&weight);

        out.push(entry.tones.len() as u8);
        for &t in &entry.tones {
            out.push(t.number());
        }
    }
    out
}

/// Reads a 1-byte length-prefixed UTF-8 string from `bytes` starting at `*i`,
/// advancing `*i` past the string bytes on success.
fn read_len_str(bytes: &[u8], i: &mut usize) -> Option<String> {
    if *i >= bytes.len() {
        return None;
    }
    let len = bytes[*i] as usize;
    *i += 1;
    if *i + len > bytes.len() {
        return None;
    }
    let s = std::str::from_utf8(&bytes[*i..*i + len]).ok()?.to_string();
    *i += len;
    Some(s)
}

/// Reads a 1-byte length-prefixed sequence of tone numbers from `bytes` starting at `*i`,
/// advancing `*i` past the tone bytes on success.
fn read_tones(bytes: &[u8], i: &mut usize) -> Option<Vec<Tone>> {
    if *i >= bytes.len() {
        return None;
    }
    let len = bytes[*i] as usize;
    *i += 1;
    if *i + len > bytes.len() {
        return None;
    }
    let mut tones = Vec::with_capacity(len);
    for &b in &bytes[*i..*i + len] {
        if let Some(t) = Tone::from_u8(b) {
            tones.push(t);
        }
    }
    *i += len;
    Some(tones)
}

/// Decodes entries from a compact binary format into a `Vec<LexiconEntry>`.
fn decode_compact(bytes: &[u8]) -> Vec<LexiconEntry> {
    let mut entries = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let Some(tailo) = read_len_str(bytes, &mut i) else {
            break;
        };
        let Some(hanzi) = read_len_str(bytes, &mut i) else {
            break;
        };
        if i + 2 > bytes.len() {
            break;
        }
        let weight = u16::from_le_bytes([bytes[i], bytes[i + 1]]) as u32;
        i += 2;
        let Some(tones) = read_tones(bytes, &mut i) else {
            break;
        };

        entries.push(LexiconEntry {
            tailo,
            hanzi,
            weight,
            tones,
        });
    }
    entries
}

/// Encodes a slice of `(typo, Candidate)` hint pairs into a deterministic compact binary format.
pub fn encode_hints_compact(hints: &[(String, Candidate)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (typo, cand) in hints {
        let key = normalize(typo);
        let typo_bytes = key.as_bytes();
        let tailo_bytes = cand.tailo.as_bytes();
        let hanzi_bytes = cand.hanzi.as_bytes();
        if typo_bytes.is_empty()
            || typo_bytes.len() > 255
            || tailo_bytes.len() > 255
            || hanzi_bytes.len() > 255
            || cand.tones.len() > 255
        {
            continue;
        }

        out.push(typo_bytes.len() as u8);
        out.extend_from_slice(typo_bytes);

        out.push(tailo_bytes.len() as u8);
        out.extend_from_slice(tailo_bytes);

        out.push(hanzi_bytes.len() as u8);
        out.extend_from_slice(hanzi_bytes);

        out.push(cand.tones.len() as u8);
        for &t in &cand.tones {
            out.push(t.number());
        }
    }
    out
}

/// Decodes orthographic hints from a compact binary format into `Vec<(String, Candidate)>`.
fn decode_hints_compact(bytes: &[u8]) -> Vec<(String, Candidate)> {
    let mut pairs = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let Some(typo) = read_len_str(bytes, &mut i) else {
            break;
        };
        let Some(tailo) = read_len_str(bytes, &mut i) else {
            break;
        };
        let Some(hanzi) = read_len_str(bytes, &mut i) else {
            break;
        };
        let Some(tones) = read_tones(bytes, &mut i) else {
            break;
        };

        pairs.push((
            typo,
            Candidate {
                hanzi,
                tailo,
                tones,
            },
        ));
    }

    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(tailo: &str, hanzi: &str, weight: u32, tones: &[Tone]) -> LexiconEntry {
        LexiconEntry {
            tailo: tailo.to_string(),
            hanzi: hanzi.to_string(),
            weight,
            tones: tones.to_vec(),
        }
    }

    fn hanzi_of(cands: &[Candidate]) -> Vec<&str> {
        cands.iter().map(|c| c.hanzi.as_str()).collect()
    }

    #[test]
    fn test_hyphenated_data_is_found_by_plain_letters_and_hyphenated_input() {
        let mut idx = TailoIndex::default();
        idx.insert(entry("lim-tsui", "啉水", 10, &[Tone::Tone1, Tone::Tone2]));
        assert_eq!(hanzi_of(&idx.lookup("limtsui")), vec!["啉水"]);
        assert_eq!(hanzi_of(&idx.lookup("lim-tsui")), vec!["啉水"]);
    }

    #[test]
    fn test_lookup_prioritizes_exact_match_before_prefix_extensions() {
        let mut idx = TailoIndex::default();
        idx.insert(entry("lim-tsui", "啉水", 10, &[Tone::Tone1, Tone::Tone2]));
        idx.insert(entry("lim", "啉", 5, &[Tone::Tone1]));
        assert_eq!(hanzi_of(&idx.lookup("lim")), vec!["啉", "啉水"]);
    }

    #[test]
    fn test_lookup_deduplicates_identical_hanzi_keeping_highest_ranked() {
        let mut idx = TailoIndex::default();
        idx.insert(entry("tsui-tsinn", "水井", 4, &[Tone::Tone2, Tone::Tone2]));
        idx.insert(entry("tsui-tsenn", "水井", 8, &[Tone::Tone2, Tone::Tone2]));
        idx.insert(entry("tsui", "水", 5, &[Tone::Tone2]));

        let cands = idx.lookup("tsui");
        assert_eq!(hanzi_of(&cands), vec!["水", "水井"]);
        assert_eq!(cands[1].tailo, "tsui-tsenn");
    }

    #[test]
    fn test_lookup_sorts_by_weight_desc() {
        let mut idx = TailoIndex::default();
        idx.insert(entry("tsui", "嘴", 3, &[Tone::Tone3]));
        idx.insert(entry("tsui", "水", 9, &[Tone::Tone2]));
        idx.insert(entry("tsui", "媠", 5, &[Tone::Tone2]));
        assert_eq!(hanzi_of(&idx.lookup("tsui")), vec!["水", "媠", "嘴"]);
    }

    #[test]
    fn test_lookup_tie_breaks_equal_weight_by_fewer_syllables_first() {
        let mut idx = TailoIndex::default();
        // '主' (U+4E3B) < '水' (U+6C34) in Unicode, but "水" has 1 syllable while "主意" has 2.
        idx.insert(entry("tsu-i", "主意", 100, &[Tone::Tone2, Tone::Tone3]));
        idx.insert(entry("tsui", "水", 100, &[Tone::Tone2]));
        assert_eq!(hanzi_of(&idx.lookup("tsui")), vec!["水", "主意"]);
    }

    #[test]
    fn test_lookup_with_tone_digit_filters_by_last_tone() {
        let mut idx = TailoIndex::default();
        idx.insert(entry("tsui", "嘴", 3, &[Tone::Tone3]));
        idx.insert(entry("tsui", "水", 9, &[Tone::Tone2]));
        idx.insert(entry("tsui-tsun", "水準", 4, &[Tone::Tone2, Tone::Tone2]));
        assert_eq!(hanzi_of(&idx.lookup("tsui2")), vec!["水"]);
        assert!(idx.lookup("tsui5").is_empty());
    }

    #[test]
    fn test_lookup_unknown_or_malformed_returns_empty() {
        let mut idx = TailoIndex::default();
        idx.insert(entry("tsui", "水", 9, &[Tone::Tone2]));
        assert!(idx.lookup("xyz").is_empty());
        assert!(idx.lookup("").is_empty());
        assert!(idx.lookup("2").is_empty());
        assert!(idx.lookup("--").is_empty());
        assert!(idx.lookup("ts1ui").is_empty());
        assert!(idx.lookup("水").is_empty());
    }

    #[test]
    fn test_insert_ignores_entry_with_empty_key() {
        let mut idx = TailoIndex::default();
        idx.insert(entry("-1", "空", 1, &[]));
        assert!(idx.lookup("a").is_empty());
    }

    #[test]
    fn test_compact_binary_roundtrip() {
        let entries = vec![
            entry("tsui", "水", 100, &[Tone::Tone2]),
            entry("tsui", "媠", 80, &[Tone::Tone2]),
            entry("lim-tsui", "啉水", 95, &[Tone::Tone1, Tone::Tone2]),
        ];

        let bytes = encode_compact(&entries);
        let loaded_index = TailoIndex::load_compact(&bytes);

        let cands = loaded_index.lookup("tsui");
        assert_eq!(cands.len(), 2);
        assert_eq!(cands[0].hanzi, "水");
        assert_eq!(cands[1].hanzi, "媠");

        let cands_with_tone = loaded_index.lookup("tsui2");
        assert_eq!(cands_with_tone.len(), 2);

        let lim_cands = loaded_index.lookup("lim");
        assert_eq!(lim_cands.len(), 1);
        assert_eq!(lim_cands[0].hanzi, "啉水");
    }

    #[test]
    fn test_insert_and_prefix_lookup_hints() {
        let mut idx = TailoIndex::default();
        let cand = |hanzi: &str, tailo: &str, tones: &[Tone]| Candidate {
            hanzi: hanzi.to_string(),
            tailo: tailo.to_string(),
            tones: tones.to_vec(),
        };
        idx.insert_hint("jire", cand("這个", "tsit-e", &[Tone::Tone4, Tone::Tone5]));
        idx.insert_hint("jia", cand("食", "tsiah", &[Tone::Tone8]));
        idx.insert_hint("jia", cand("遮", "tsia", &[Tone::Tone1]));
        idx.insert_hint("jia", cand("食", "tsiah", &[Tone::Tone8]));
        idx.insert_hint(
            "jiababue",
            cand(
                "食飽未",
                "tsiah-pa-bue",
                &[Tone::Tone8, Tone::Tone2, Tone::Tone7],
            ),
        );
        idx.insert_hint(
            "jiahuann",
            cand(
                "食飯配滷蛋",
                "tsiah-png phue lo-tan",
                &[
                    Tone::Tone8,
                    Tone::Tone7,
                    Tone::Tone3,
                    Tone::Tone2,
                    Tone::Tone7,
                ],
            ),
        );

        assert_eq!(hanzi_of(&idx.lookup_hints("jir")), vec!["這个"]);
        assert_eq!(hanzi_of(&idx.lookup_hints("jire")), vec!["這个"]);
        // Exact matches ("食", "遮") rank first, followed by shorter prefix match ("食飽未"), capped at 3
        assert_eq!(
            hanzi_of(&idx.lookup_hints("jia")),
            vec!["食", "遮", "食飽未"]
        );
    }

    #[test]
    fn test_hints_compact_binary_roundtrip() {
        let cand = Candidate {
            hanzi: "這个".to_string(),
            tailo: "tsit-e".to_string(),
            tones: vec![Tone::Tone4, Tone::Tone5],
        };
        let bytes = encode_hints_compact(&[
            ("jire".to_string(), cand.clone()),
            ("jile".to_string(), cand.clone()),
        ]);
        let mut idx = TailoIndex::default();
        idx.load_hints_compact(&bytes);
        assert_eq!(idx.lookup_hints("jire"), vec![cand.clone()]);
        assert_eq!(idx.lookup_hints("jile"), vec![cand]);
    }

    #[test]
    fn test_load_actual_compact_dict() {
        let index = TailoIndex::built_in();

        // Verify basic lookup
        let cands = index.lookup("tsui");
        assert!(!cands.is_empty());
        assert!(cands.iter().any(|c| c.hanzi == "水"));

        // Verify Option A Kana conversion: nagasi -> 那卡西
        let nagasi_cands = index.lookup("nagasi");
        assert!(!nagasi_cands.is_empty());
        assert!(nagasi_cands.iter().any(|c| c.hanzi == "那卡西"));

        // Verify demonstratives: tsite -> 這个, hite -> 彼个
        let tsite_cands = index.lookup("tsite");
        assert!(tsite_cands.iter().any(|c| c.hanzi == "這个"));
        let hite_cands = index.lookup("hite");
        assert!(hite_cands.iter().any(|c| c.hanzi == "彼个"));

        // Verify built-in orthographic hints: jire / jile -> 這个
        let jire_hints = index.lookup_hints("jire");
        assert_eq!(jire_hints.len(), 1);
        assert_eq!(jire_hints[0].hanzi, "這个");
        let jile_hints = index.lookup_hints("jile");
        assert_eq!(jile_hints.len(), 1);
        assert_eq!(jile_hints[0].hanzi, "這个");

        // Verify supplement additions: suio -> 媠喔 (and reject vulgar/Mandarin loan 水喔)
        let suio_cands = index.lookup("suio");
        assert!(suio_cands.iter().any(|c| c.hanzi == "媠喔"));
        assert!(!suio_cands.iter().any(|c| c.hanzi == "水喔"));

        // Verify colloquial hint mappings: ganane -> 敢按呢, jiababue -> 食飽未, mutan -> 毋通
        let ganane_hints = index.lookup_hints("ganane");
        assert!(ganane_hints.iter().any(|c| c.hanzi == "敢按呢"));
        let jiababue_hints = index.lookup_hints("jiababue");
        assert!(jiababue_hints.iter().any(|c| c.hanzi == "食飽未"));
        let mutan_hints = index.lookup_hints("mutan");
        assert!(mutan_hints.iter().any(|c| c.hanzi == "毋通"));
        let dongbeitiao_hints = index.lookup_hints("dongbeitiao");
        assert!(dongbeitiao_hints.iter().any(|c| c.hanzi == "擋袂牢"));

        // Verify slang and lyrics hints (prefix and exact match)
        let risi_hints = index.lookup_hints("risihinkang");
        assert!(risi_hints.iter().any(|c| c.hanzi == "你是耳空生包皮"));
        let wagari_hints = index.lookup_hints("wagari");
        assert!(wagari_hints.iter().any(|c| c.hanzi == "我合你上好就到遮"));
        let jimin_hints = index.lookup_hints("jimin");
        assert!(jimin_hints.iter().any(|c| c.hanzi == "志明真正毋知欲按怎"));
        let maigo_hints = index.lookup_hints("maigo");
        assert!(maigo_hints.iter().any(|c| c.hanzi == "莫閣傷心"));

        // Verify lyrics prefix lookup on lexicon trie
        let guakapli_cands = index.lookup("guakapli");
        assert!(guakapli_cands.iter().any(|c| c.hanzi == "我合你上好就到遮"));
        let kiannkau_cands = index.lookup("kiannkau");
        assert!(kiannkau_cands.iter().any(|c| c.hanzi == "行到淡水的海岸"));
    }

    #[test]
    fn test_lookup_hyphen_and_trailing_tone() {
        let mut idx = TailoIndex::default();
        idx.insert(LexiconEntry {
            tailo: "lim-tsui".to_string(),
            hanzi: "啉水".to_string(),
            weight: 10,
            tones: vec![Tone::Tone1, Tone::Tone2],
        });
        idx.insert(LexiconEntry {
            tailo: "lim".to_string(),
            hanzi: "啉".to_string(),
            weight: 20,
            tones: vec![Tone::Tone1],
        });

        // Exact match with hyphen
        let cands_hyphen = idx.lookup("lim-tsui");
        assert_eq!(cands_hyphen.len(), 1);
        assert_eq!(cands_hyphen[0].hanzi, "啉水");

        // Hyphen with trailing tone
        let cands_tone = idx.lookup("lim-tsui2");
        assert_eq!(cands_tone.len(), 1);
        assert_eq!(cands_tone[0].hanzi, "啉水");

        // Wrong tone returns empty
        assert!(idx.lookup("lim-tsui3").is_empty());

        // Trailing hyphen matches prefix
        let cands_prefix = idx.lookup("lim-");
        assert_eq!(cands_prefix.len(), 2);
        assert_eq!(cands_prefix[0].hanzi, "啉");
        assert_eq!(cands_prefix[1].hanzi, "啉水");

        // Consecutive hyphens or lone hyphens return empty
        assert!(idx.lookup("---").is_empty());
        assert!(idx.lookup("-").is_empty());
    }
}
