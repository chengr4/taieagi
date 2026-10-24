mod composer;
mod tailo_index;
mod tone;
pub mod wasm;

pub use tailo_index::{Candidate, LexiconEntry, encode_compact, encode_hints_compact};
pub use tone::{Tone, parse_tailo_word};
