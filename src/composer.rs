//! Input method state machine: turns key events into a preedit buffer and candidates.

use crate::tailo_index::{Candidate, TailoIndex};
use crate::tone::{Tone, apply_tone};

/// Formatting mode applied to committed candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum OutputMode {
    /// Pure Hanzi (default), e.g. "水" / "啉水".
    #[default]
    HanziOnly,
    /// Hanzi followed by Tailo with tone numbers, e.g. "水 (tsui2)" / "啉水 (lim1-tsui2)".
    HanziWithNumber,
    /// Hanzi followed by Tailo with diacritical marks, e.g. "水 (tsuí)" / "啉水 (lim-tsuí)".
    HanziWithMarks,
}

/// Formats a candidate for insertion according to the selected [`OutputMode`].
///
/// Falls back to pure Hanzi if romanization or tones are missing or if the number
/// of syllables does not match the number of tones.
pub(crate) fn format_commit(candidate: &Candidate, mode: OutputMode) -> String {
    if mode == OutputMode::HanziOnly {
        return candidate.hanzi.clone();
    }

    let syllables: Vec<&str> = candidate
        .tailo
        .split(|c: char| c == '-' || c.is_ascii_whitespace())
        .filter(|s| !s.is_empty())
        .collect();

    if syllables.is_empty() || syllables.len() != candidate.tones.len() {
        return candidate.hanzi.clone();
    }

    let formatted_tailo = match mode {
        OutputMode::HanziOnly => unreachable!(),
        OutputMode::HanziWithNumber => syllables
            .iter()
            .zip(&candidate.tones)
            .map(|(syl, tone)| format!("{syl}{}", tone.number()))
            .collect::<Vec<_>>()
            .join("-"),
        OutputMode::HanziWithMarks => syllables
            .iter()
            .zip(&candidate.tones)
            .map(|(syl, tone)| apply_tone(syl, *tone))
            .collect::<Vec<_>>()
            .join("-"),
    };

    format!("{} ({})", candidate.hanzi, formatted_tailo)
}

/// Result of a composition or selection editing operation on [`Composer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditOutcome {
    /// State changed or the key was swallowed; the UI should re-render.
    Handled,
    /// Not ours; let the page handle the key.
    PassThrough,
}

/// Input state machine over a [`TailoIndex`].
pub(crate) struct Composer {
    index: TailoIndex,
    buffer: String,
    candidates: Vec<Candidate>,
    selected: usize,
    mode: OutputMode,
}

impl Composer {
    /// Creates a composer loaded with the embedded MOE lexicon and hints.
    pub(crate) fn built_in() -> Self {
        Self::new(TailoIndex::built_in())
    }

    pub(crate) fn new(index: TailoIndex) -> Self {
        Self {
            index,
            buffer: String::new(),
            candidates: Vec::new(),
            selected: 0,
            mode: OutputMode::default(),
        }
    }

    pub(crate) fn output_mode(&self) -> OutputMode {
        self.mode
    }

    pub(crate) fn set_output_mode(&mut self, mode: OutputMode) {
        self.mode = mode;
    }

    /// Feeds a typed character: a letter, a hyphen, or a tone digit.
    ///
    /// - A tone digit is accepted only when the buffer is non-empty and does
    ///   not already end with a digit; otherwise `PassThrough`.
    /// - A hyphen is accepted when the buffer is non-empty and does not already
    ///   end with a digit; otherwise `PassThrough` on empty or swallowed after digit.
    /// - With an empty buffer, anything but a letter is `PassThrough`.
    /// - With a non-empty buffer, unsupported characters, and characters typed
    ///   after a tone digit, are swallowed without changing state.
    pub(crate) fn push_char(&mut self, c: char) -> EditOutcome {
        let ends_with_digit = self
            .buffer
            .chars()
            .next_back()
            .is_some_and(|last| last.is_ascii_digit());

        if Tone::from_char(c).is_some() {
            if self.buffer.is_empty() || ends_with_digit {
                return EditOutcome::PassThrough;
            }
            self.buffer.push(c);
        } else if c.is_ascii_alphabetic() {
            if ends_with_digit {
                return EditOutcome::Handled;
            }
            self.buffer.push(c.to_ascii_lowercase());
        } else if c == '-' {
            if self.buffer.is_empty() {
                return EditOutcome::PassThrough;
            }
            if ends_with_digit {
                return EditOutcome::Handled;
            }
            self.buffer.push('-');
        } else if self.buffer.is_empty() {
            return EditOutcome::PassThrough;
        } else {
            return EditOutcome::Handled;
        }

        self.refresh();
        EditOutcome::Handled
    }

    /// Removes the last buffered character. `PassThrough` if the buffer is empty.
    pub(crate) fn backspace(&mut self) -> EditOutcome {
        if self.buffer.pop().is_none() {
            return EditOutcome::PassThrough;
        }
        self.refresh();
        EditOutcome::Handled
    }

    /// Moves the highlighted candidate by `delta`, clamped to the list bounds (no wrap).
    pub(crate) fn move_selection(&mut self, delta: i32) -> EditOutcome {
        if self.candidates.is_empty() {
            return EditOutcome::PassThrough;
        }
        let last = self.candidates.len() as i64 - 1;
        let next = (self.selected as i64 + delta as i64).clamp(0, last);
        self.selected = next as usize;
        EditOutcome::Handled
    }

    /// Commits the highlighted candidate (Space / Enter / Click).
    /// With no candidates the action is ignored and the buffer is left unchanged.
    pub(crate) fn commit_selected(&mut self) -> Option<String> {
        let candidate = self.candidates.get(self.selected)?;
        let text = format_commit(candidate, self.mode);
        self.reset();
        Some(text)
    }

    /// Abandons the current input (Esc). `PassThrough` if nothing is being composed.
    pub(crate) fn cancel(&mut self) -> EditOutcome {
        if self.buffer.is_empty() {
            return EditOutcome::PassThrough;
        }
        self.reset();
        EditOutcome::Handled
    }

    /// The raw typed buffer (default preedit mode).
    pub(crate) fn preedit(&self) -> &str {
        &self.buffer
    }

    pub(crate) fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    pub(crate) fn hints(&self) -> &[Candidate] {
        self.index.lookup_hints(&self.buffer)
    }

    /// Highlighted candidate index, 0-based.
    pub(crate) fn selected(&self) -> usize {
        self.selected
    }

    fn refresh(&mut self) {
        self.candidates = self.index.lookup(&self.buffer);
        self.selected = 0;
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.candidates.clear();
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tailo_index::LexiconEntry;

    fn composer() -> Composer {
        let mut index = TailoIndex::default();
        let mut add = |tailo: &str, hanzi: &str, weight: u32, tones: &[Tone]| {
            index.insert(LexiconEntry {
                tailo: tailo.to_string(),
                hanzi: hanzi.to_string(),
                weight,
                tones: tones.to_vec(),
            });
        };
        add("tsui", "水", 9, &[Tone::Tone2]);
        add("tsui", "媠", 5, &[Tone::Tone2]);
        add("tsui", "嘴", 3, &[Tone::Tone3]);
        add("lim-tsui", "啉水", 10, &[Tone::Tone1, Tone::Tone2]);
        Composer::new(index)
    }

    fn type_str(e: &mut Composer, s: &str) {
        for c in s.chars() {
            e.push_char(c);
        }
    }

    fn hanzi_of(e: &Composer) -> Vec<&str> {
        e.candidates().iter().map(|c| c.hanzi.as_str()).collect()
    }

    #[test]
    fn test_push_letters_updates_candidates_and_resets_selection() {
        let mut e = composer();
        type_str(&mut e, "tsui");
        assert_eq!(e.preedit(), "tsui");
        assert_eq!(hanzi_of(&e), vec!["水", "媠", "嘴"]);
        e.move_selection(2);
        e.push_char('x');
        assert_eq!(e.selected(), 0);
    }

    #[test]
    fn test_tone_digit_rejected_on_empty_or_after_digit() {
        let mut e = composer();
        assert_eq!(e.push_char('2'), EditOutcome::PassThrough);
        assert_eq!(e.preedit(), "");

        type_str(&mut e, "tsui2");
        assert_eq!(e.push_char('3'), EditOutcome::PassThrough);
        assert_eq!(e.preedit(), "tsui2");
        assert_eq!(hanzi_of(&e), vec!["水", "媠"]);
    }

    #[test]
    fn test_letters_after_tone_digit_are_swallowed() {
        let mut e = composer();
        type_str(&mut e, "tsui2");
        assert_eq!(e.push_char('a'), EditOutcome::Handled);
        assert_eq!(e.preedit(), "tsui2");
    }

    #[test]
    fn test_unsupported_chars() {
        let mut e = composer();
        assert_eq!(e.push_char('!'), EditOutcome::PassThrough);
        assert_eq!(e.push_char('-'), EditOutcome::PassThrough);
        type_str(&mut e, "ts");
        assert_eq!(e.push_char('!'), EditOutcome::Handled);
        assert_eq!(e.preedit(), "ts");
    }

    #[test]
    fn test_backspace_recomputes_and_passes_through_when_empty() {
        let mut e = composer();
        assert_eq!(e.backspace(), EditOutcome::PassThrough);
        type_str(&mut e, "tsuix");
        assert!(e.candidates().is_empty());
        assert_eq!(e.backspace(), EditOutcome::Handled);
        assert_eq!(e.preedit(), "tsui");
        assert_eq!(hanzi_of(&e), vec!["水", "媠", "嘴"]);
    }

    #[test]
    fn test_move_selection_clamps_without_wrapping() {
        let mut e = composer();
        type_str(&mut e, "tsui");
        e.move_selection(-1);
        assert_eq!(e.selected(), 0);
        e.move_selection(10);
        assert_eq!(e.selected(), 2);
        e.move_selection(-1);
        assert_eq!(e.selected(), 1);
    }

    #[test]
    fn test_commit_selected_returns_highlighted_and_clears() {
        let mut e = composer();
        type_str(&mut e, "tsui");
        e.move_selection(1);
        assert_eq!(e.commit_selected(), Some("媠".to_string()));
        assert_eq!(e.preedit(), "");
        assert!(e.candidates().is_empty());
        assert_eq!(e.selected(), 0);
    }

    #[test]
    fn test_commit_with_no_candidates_keeps_buffer() {
        let mut e = composer();
        type_str(&mut e, "xyz");
        assert_eq!(e.commit_selected(), None);
        assert_eq!(e.preedit(), "xyz");
    }

    #[test]
    fn test_cancel_clears_buffer_and_candidates() {
        let mut e = composer();
        assert_eq!(e.cancel(), EditOutcome::PassThrough);
        type_str(&mut e, "tsui");
        assert_eq!(e.cancel(), EditOutcome::Handled);
        assert_eq!(e.preedit(), "");
        assert!(e.candidates().is_empty());
    }

    #[test]
    fn test_compound_word_typed_without_hyphen_and_with_tone() {
        let mut e = composer();
        type_str(&mut e, "limtsui2");
        assert_eq!(hanzi_of(&e), vec!["啉水"]);
    }

    #[test]
    fn test_format_commit_hanzi_only() {
        let single = Candidate {
            hanzi: "水".to_string(),
            weight: 9,
            tailo: "tsui".to_string(),
            tones: vec![Tone::Tone2],
        };
        assert_eq!(format_commit(&single, OutputMode::HanziOnly), "水");

        let compound = Candidate {
            hanzi: "啉水".to_string(),
            weight: 10,
            tailo: "lim-tsui".to_string(),
            tones: vec![Tone::Tone1, Tone::Tone2],
        };
        assert_eq!(format_commit(&compound, OutputMode::HanziOnly), "啉水");
    }

    #[test]
    fn test_format_commit_with_number() {
        let single = Candidate {
            hanzi: "水".to_string(),
            weight: 9,
            tailo: "tsui".to_string(),
            tones: vec![Tone::Tone2],
        };
        assert_eq!(
            format_commit(&single, OutputMode::HanziWithNumber),
            "水 (tsui2)"
        );

        let compound = Candidate {
            hanzi: "啉水".to_string(),
            weight: 10,
            tailo: "lim-tsui".to_string(),
            tones: vec![Tone::Tone1, Tone::Tone2],
        };
        // Preserves Tone 1 digit
        assert_eq!(
            format_commit(&compound, OutputMode::HanziWithNumber),
            "啉水 (lim1-tsui2)"
        );

        // Also supports whitespace-separated lexicon data
        let space_compound = Candidate {
            hanzi: "啉水".to_string(),
            weight: 10,
            tailo: "lim tsui".to_string(),
            tones: vec![Tone::Tone1, Tone::Tone2],
        };
        assert_eq!(
            format_commit(&space_compound, OutputMode::HanziWithNumber),
            "啉水 (lim1-tsui2)"
        );
    }

    #[test]
    fn test_format_commit_with_marks() {
        let single = Candidate {
            hanzi: "水".to_string(),
            weight: 9,
            tailo: "tsui".to_string(),
            tones: vec![Tone::Tone2],
        };
        // Tone 2 places acute combining mark \u{0301} on 'i' -> "tsui\u{0301}"
        assert_eq!(
            format_commit(&single, OutputMode::HanziWithMarks),
            "水 (tsui\u{0301})"
        );

        let compound = Candidate {
            hanzi: "啉水".to_string(),
            weight: 10,
            tailo: "lim-tsui".to_string(),
            tones: vec![Tone::Tone1, Tone::Tone2],
        };
        // Tone 1 has no combining mark in Tâi-lô; Tone 2 receives acute
        assert_eq!(
            format_commit(&compound, OutputMode::HanziWithMarks),
            "啉水 (lim-tsui\u{0301})"
        );
    }

    #[test]
    fn test_format_commit_fallbacks_to_pure_hanzi() {
        let empty_tones = Candidate {
            hanzi: "水".to_string(),
            weight: 9,
            tailo: "tsui".to_string(),
            tones: vec![],
        };
        assert_eq!(
            format_commit(&empty_tones, OutputMode::HanziWithNumber),
            "水"
        );
        assert_eq!(
            format_commit(&empty_tones, OutputMode::HanziWithMarks),
            "水"
        );

        let mismatch = Candidate {
            hanzi: "啉水".to_string(),
            weight: 10,
            tailo: "lim-tsui".to_string(),
            tones: vec![Tone::Tone1], // Expected 2, got 1
        };
        assert_eq!(
            format_commit(&mismatch, OutputMode::HanziWithNumber),
            "啉水"
        );

        let empty_tailo = Candidate {
            hanzi: "水".to_string(),
            weight: 9,
            tailo: "".to_string(),
            tones: vec![Tone::Tone2],
        };
        assert_eq!(
            format_commit(&empty_tailo, OutputMode::HanziWithMarks),
            "水"
        );
    }

    #[test]
    fn test_composer_commit_with_configured_mode() {
        let mut e = composer();
        type_str(&mut e, "limtsui2");

        // HanziWithNumber mode
        e.set_output_mode(OutputMode::HanziWithNumber);
        assert_eq!(e.output_mode(), OutputMode::HanziWithNumber);
        assert_eq!(e.commit_selected(), Some("啉水 (lim1-tsui2)".to_string()));

        // HanziWithMarks mode
        type_str(&mut e, "tsui");
        e.set_output_mode(OutputMode::HanziWithMarks);
        assert_eq!(e.commit_selected(), Some("水 (tsui\u{0301})".to_string()));
    }

    #[test]
    fn test_composer_hints_delegation() {
        let mut index = TailoIndex::default();
        index.insert_hint(
            "jire",
            Candidate {
                hanzi: "這个".to_string(),
                weight: 0,
                tailo: "tsit-e".to_string(),
                tones: vec![Tone::Tone4, Tone::Tone5],
            },
        );
        let mut e = Composer::new(index);

        type_str(&mut e, "jir");
        assert!(e.hints().is_empty());

        e.push_char('e');
        assert_eq!(e.hints().len(), 1);
        assert_eq!(e.hints()[0].hanzi, "這个");
        // Cannot commit read-only hints when candidates are empty
        assert_eq!(e.commit_selected(), None);
        assert_eq!(e.preedit(), "jire");
    }

    #[test]
    fn test_composer_handles_hyphen_composition() {
        let mut e = composer();

        // Initial hyphen with empty buffer is PassThrough
        assert_eq!(e.push_char('-'), EditOutcome::PassThrough);
        assert_eq!(e.preedit(), "");

        // Typing with hyphens: "lim-tsui"
        type_str(&mut e, "lim-tsui");
        assert_eq!(e.preedit(), "lim-tsui");
        assert_eq!(e.candidates().len(), 1);
        assert_eq!(e.candidates()[0].hanzi, "啉水");

        // Adding tone digit after hyphenated word: "lim-tsui2"
        assert_eq!(e.push_char('2'), EditOutcome::Handled);
        assert_eq!(e.preedit(), "lim-tsui2");
        assert_eq!(e.candidates().len(), 1);
        assert_eq!(e.candidates()[0].hanzi, "啉水");

        // Typing hyphen after tone digit is swallowed
        assert_eq!(e.push_char('-'), EditOutcome::Handled);
        assert_eq!(e.preedit(), "lim-tsui2");

        // Commit candidate clears buffer
        assert_eq!(e.commit_selected(), Some("啉水".to_string()));
        assert_eq!(e.preedit(), "");
    }
}
