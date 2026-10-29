//! WebAssembly bindings for [`Composer`].

use crate::composer::{self, Composer, EditOutcome, OutputMode};
use wasm_bindgen::prelude::wasm_bindgen;

/// WebAssembly-exported state machine for Taiwanese Romanization input.
#[wasm_bindgen]
pub struct WasmComposer {
    inner: Composer,
}

#[wasm_bindgen]
impl WasmComposer {
    /// Creates a new composer loaded with the embedded MOE lexicon and hints.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Composer::built_in(),
        }
    }

    /// Feeds a typed character (letter or tone digit).
    /// Returns `true` if handled by the composer, `false` if passed through.
    pub fn push_char(&mut self, c: char) -> bool {
        self.inner.push_char(c) == EditOutcome::Handled
    }

    /// Handles Backspace.
    /// Returns `true` if handled by the composer, `false` if passed through.
    pub fn backspace(&mut self) -> bool {
        self.inner.backspace() == EditOutcome::Handled
    }

    /// Handles Escape / Cancel.
    /// Returns `true` if handled by the composer, `false` if passed through.
    pub fn cancel(&mut self) -> bool {
        self.inner.cancel() == EditOutcome::Handled
    }

    /// Moves the highlighted candidate selection by `delta`.
    /// Returns `true` if handled by the composer, `false` if passed through.
    pub fn move_selection(&mut self, delta: i32) -> bool {
        self.inner.move_selection(delta) == EditOutcome::Handled
    }

    /// Commits the candidate at the specified 0-based `index` (Space / Enter / Click).
    /// Returns the committed string if the index is valid, or `None` otherwise.
    pub fn commit_index(&mut self, index: usize) -> Option<String> {
        self.inner.commit_index(index)
    }

    /// Returns the raw preedit string currently being composed.
    pub fn preedit(&self) -> String {
        self.inner.preedit().to_string()
    }

    /// Returns candidates as a native JavaScript Array of Hanzi strings (e.g. `['水', '媠', '嘴']`).
    pub fn candidates(&self) -> Vec<String> {
        self.inner
            .candidates()
            .iter()
            .map(|c| c.hanzi.clone())
            .collect()
    }

    /// Returns read-only orthographic hints formatted with diacritics (e.g. `['這个 (tsit-ê)']`).
    pub fn hints(&self) -> Vec<String> {
        self.inner
            .hints()
            .iter()
            .map(|c| composer::format_commit(c, OutputMode::HanziWithMarks))
            .collect()
    }

    /// Returns the 0-based index of the currently highlighted candidate.
    pub fn selected(&self) -> usize {
        self.inner.selected()
    }

    /// Sets the output mode:
    /// - 0: HanziOnly
    /// - 1: HanziWithNumber
    /// - 2: HanziWithMarks
    pub fn set_output_mode(&mut self, mode: u8) {
        let mode = match mode {
            0 => OutputMode::HanziOnly,
            1 => OutputMode::HanziWithNumber,
            2 => OutputMode::HanziWithMarks,
            _ => OutputMode::default(),
        };
        self.inner.set_output_mode(mode);
    }

    /// Returns the current output mode (0, 1, or 2).
    pub fn output_mode(&self) -> u8 {
        match self.inner.output_mode() {
            OutputMode::HanziOnly => 0,
            OutputMode::HanziWithNumber => 1,
            OutputMode::HanziWithMarks => 2,
        }
    }
}

impl Default for WasmComposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_composer_workflow() {
        let mut wc = WasmComposer::new();

        // Push letters
        assert!(wc.push_char('t'));
        assert!(wc.push_char('s'));
        assert!(wc.push_char('u'));
        assert!(wc.push_char('i'));
        assert_eq!(wc.preedit(), "tsui");

        // Candidates check (direct Vec<String>)
        let cands = wc.candidates();
        assert!(!cands.is_empty());
        assert!(cands.contains(&"水".to_string()));
        assert_eq!(wc.selected(), 0);

        // Move selection
        assert!(wc.move_selection(1));
        assert_eq!(wc.selected(), 1);

        // Commit candidate (HanziWithMarks default)
        assert_eq!(
            wc.commit_index(wc.selected()),
            Some(format!("{} (tsui\u{0302})", cands[1]))
        );
        assert_eq!(wc.preedit(), "");
        assert!(wc.candidates().is_empty());
    }

    #[test]
    fn test_wasm_composer_output_modes() {
        let mut wc = WasmComposer::new();
        wc.set_output_mode(1); // HanziWithNumber
        assert_eq!(wc.output_mode(), 1);

        for c in "tsui2".chars() {
            wc.push_char(c);
        }
        assert_eq!(wc.commit_index(0), Some("水 (tsui2)".to_string()));

        wc.set_output_mode(2); // HanziWithMarks
        assert_eq!(wc.output_mode(), 2);
        for c in "limtsui2".chars() {
            wc.push_char(c);
        }
        assert_eq!(
            wc.commit_index(0),
            Some("啉水 (lim-tsui\u{0301})".to_string())
        );
    }

    #[test]
    fn test_wasm_composer_backspace_and_cancel() {
        let mut wc = WasmComposer::new();
        assert!(!wc.backspace());

        wc.push_char('b');
        wc.push_char('o');
        assert_eq!(wc.preedit(), "bo");

        assert!(wc.backspace());
        assert_eq!(wc.preedit(), "b");

        assert!(wc.cancel());
        assert_eq!(wc.preedit(), "");
        assert!(!wc.cancel());
    }

    #[test]
    fn test_wasm_composer_hints() {
        let mut wc = WasmComposer::new();
        for c in "jire".chars() {
            wc.push_char(c);
        }
        assert!(wc.candidates().is_empty());
        assert_eq!(wc.hints(), vec!["這个 (tsit-e\u{0302})".to_string()]);
    }
}
