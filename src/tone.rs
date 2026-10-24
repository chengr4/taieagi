//! Tone definitions and tone mark conversions for Taiwanese Romanization.

/// Tone enumeration for Taiwanese Romanization (Tâi-lô / POJ tones 1 to 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    Tone1,
    Tone2,
    Tone3,
    Tone4,
    Tone5,
    Tone6,
    Tone7,
    Tone8,
    Tone9,
}

impl Tone {
    /// Parse tone from an ASCII digit character ('1'..='9').
    pub(crate) fn from_char(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::Tone1),
            '2' => Some(Self::Tone2),
            '3' => Some(Self::Tone3),
            '4' => Some(Self::Tone4),
            '5' => Some(Self::Tone5),
            '6' => Some(Self::Tone6),
            '7' => Some(Self::Tone7),
            '8' => Some(Self::Tone8),
            '9' => Some(Self::Tone9),
            _ => None,
        }
    }

    /// Parse tone from an integer (1..=9).
    pub(crate) fn from_u8(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::Tone1),
            2 => Some(Self::Tone2),
            3 => Some(Self::Tone3),
            4 => Some(Self::Tone4),
            5 => Some(Self::Tone5),
            6 => Some(Self::Tone6),
            7 => Some(Self::Tone7),
            8 => Some(Self::Tone8),
            9 => Some(Self::Tone9),
            _ => None,
        }
    }

    /// Returns the corresponding Unicode Combining Diacritical Mark.
    /// Tone 1 and Tone 4 have no diacritical marks and return None.
    fn combining_mark(&self) -> Option<char> {
        match self {
            Self::Tone1 | Self::Tone4 => None,
            Self::Tone2 => Some('\u{0301}'), // Acute: ́
            Self::Tone3 => Some('\u{0300}'), // Grave: ̀
            Self::Tone5 => Some('\u{0302}'), // Circumflex: ̂
            Self::Tone6 => Some('\u{030C}'), // Caron: ̌
            Self::Tone7 => Some('\u{0304}'), // Macron: ̄
            Self::Tone8 => Some('\u{030D}'), // Vertical line above: ̍
            Self::Tone9 => Some('\u{030B}'), // Double acute: ̋
        }
    }

    /// Returns the numeric representation of the tone (1 to 9).
    pub(crate) fn number(&self) -> u8 {
        match self {
            Self::Tone1 => 1,
            Self::Tone2 => 2,
            Self::Tone3 => 3,
            Self::Tone4 => 4,
            Self::Tone5 => 5,
            Self::Tone6 => 6,
            Self::Tone7 => 7,
            Self::Tone8 => 8,
            Self::Tone9 => 9,
        }
    }
}

/// Finds the byte index within `syllable` of the character that should receive the tone mark,
/// following the official Ministry of Education Taiwanese Romanization (Tâi-lô) rules:
/// 1. `a > oo > e, o > i, u`
/// 2. In `iu` and `ui`, the tone mark is placed on the second vowel (`u` and `i` respectively).
/// 3. In `oo`, the tone mark is placed on the first `o`.
/// 4. In syllabic nasals without vowels (`ng`, `m`), `ng` marks on `n`, `m` marks on `m`.
/// 5. Returns `None` if no valid tone-bearing vowel or syllabic nasal is found.
fn find_tone_target_index(syllable: &str) -> Option<usize> {
    let lower = syllable.to_ascii_lowercase();

    // Exception: dialect triple vowel "ere" marks on the second 'e'
    if let Some(pos) = lower.find("ere") {
        return Some(pos + 2);
    }

    // Priority 1: 'a' has the highest priority over all other vowels
    if let Some(pos) = lower.find('a') {
        return Some(pos);
    }

    // Priority 2: 'oo' marks on the first 'o'
    if let Some(pos) = lower.find("oo") {
        return Some(pos);
    }

    // Special diphthongs: 'iu' marks on 'u', 'ui' marks on 'i'
    if let Some(pos) = lower.find("iu") {
        return Some(pos + 1);
    }
    if let Some(pos) = lower.find("ui") {
        return Some(pos + 1);
    }

    // Legacy POJ 'oe' marks on 'e'
    if let Some(pos) = lower.find("oe") {
        return Some(pos + 1);
    }

    // Priority 3: 'e' or 'o'
    if let Some(pos) = lower.find('e') {
        return Some(pos);
    }
    if let Some(pos) = lower.find('o') {
        return Some(pos);
    }

    // Priority 4: 'i' or 'u'
    if let Some(pos) = lower.find('i') {
        return Some(pos);
    }
    if let Some(pos) = lower.find('u') {
        return Some(pos);
    }

    // Priority 5: Syllabic nasals (when no vowels are present)
    // Syllabic 'ng' (e.g., "ng", "sng", "mng") marks on 'n'
    if let Some(pos) = lower.find("ng") {
        return Some(pos);
    }
    // Syllabic 'm' (e.g., "m", "hm") marks on 'm'
    if let Some(pos) = lower.find('m') {
        return Some(pos);
    }

    None
}

/// Applies the specified tone to a syllable string, placing the Unicode Combining
/// Diacritical Mark directly after the target character determined by `find_tone_target_index`.
/// If the tone has no diacritical mark (Tone 1, Tone 4) or no valid target character
/// is found, the original syllable is returned as a new `String`.
pub(crate) fn apply_tone(syllable: &str, tone: Tone) -> String {
    let Some(mark) = tone.combining_mark() else {
        return syllable.to_string();
    };

    let Some(target_idx) = find_tone_target_index(syllable) else {
        return syllable.to_string();
    };

    let target_char_len = syllable[target_idx..]
        .chars()
        .next()
        .map(|c| c.len_utf8())
        .unwrap_or(1);
    let insert_pos = target_idx + target_char_len;

    let mut result = String::with_capacity(syllable.len() + mark.len_utf8());
    result.push_str(&syllable[..insert_pos]);
    result.push(mark);
    result.push_str(&syllable[insert_pos..]);
    result
}

/// Splits an input string into the base syllable slice and an optional tone,
/// if the string ends with an ASCII tone digit ('1'..='9').
/// If the trailing character is not a valid tone digit, `(input, None)` is returned.
/// This operation is zero-copy and borrows directly from the input string slice.
pub(crate) fn split_syllable_and_tone(input: &str) -> (&str, Option<Tone>) {
    if let Some(last_char) = input.chars().next_back() {
        if let Some(tone) = Tone::from_char(last_char) {
            let prefix_len = input.len() - last_char.len_utf8();
            return (&input[..prefix_len], Some(tone));
        }
    }
    (input, None)
}

/// Parses a single Tâi-lô syllable with Unicode diacritics into plain ASCII lowercase letters
/// and its corresponding [`Tone`].
///
/// If no explicit tone mark is found:
/// - Syllables ending with stop codas ('p', 't', 'k', 'h') are assigned [`Tone::Tone4`].
/// - All other syllables are assigned [`Tone::Tone1`].
fn parse_tailo_syllable(syllable: &str) -> (String, Tone) {
    let mut plain = String::with_capacity(syllable.len());
    let mut detected_tone: Option<Tone> = None;

    for c in syllable.chars() {
        match c {
            // Tone 2 (Acute)
            'á' | 'Á' => {
                plain.push('a');
                detected_tone = Some(Tone::Tone2);
            }
            'é' | 'É' => {
                plain.push('e');
                detected_tone = Some(Tone::Tone2);
            }
            'í' | 'Í' => {
                plain.push('i');
                detected_tone = Some(Tone::Tone2);
            }
            'ó' | 'Ó' => {
                plain.push('o');
                detected_tone = Some(Tone::Tone2);
            }
            'ú' | 'Ú' => {
                plain.push('u');
                detected_tone = Some(Tone::Tone2);
            }
            'ń' | 'Ń' => {
                plain.push('n');
                detected_tone = Some(Tone::Tone2);
            }
            'ḿ' | 'Ḿ' => {
                plain.push('m');
                detected_tone = Some(Tone::Tone2);
            }
            '\u{0301}' => {
                detected_tone = Some(Tone::Tone2);
            }

            // Tone 3 (Grave)
            'à' | 'À' => {
                plain.push('a');
                detected_tone = Some(Tone::Tone3);
            }
            'è' | 'È' => {
                plain.push('e');
                detected_tone = Some(Tone::Tone3);
            }
            'ì' | 'Ì' => {
                plain.push('i');
                detected_tone = Some(Tone::Tone3);
            }
            'ò' | 'Ò' => {
                plain.push('o');
                detected_tone = Some(Tone::Tone3);
            }
            'ù' | 'Ù' => {
                plain.push('u');
                detected_tone = Some(Tone::Tone3);
            }
            'ǹ' | 'Ǹ' => {
                plain.push('n');
                detected_tone = Some(Tone::Tone3);
            }
            '\u{0300}' => {
                detected_tone = Some(Tone::Tone3);
            }

            // Tone 5 (Circumflex)
            'â' | 'Â' => {
                plain.push('a');
                detected_tone = Some(Tone::Tone5);
            }
            'ê' | 'Ê' => {
                plain.push('e');
                detected_tone = Some(Tone::Tone5);
            }
            'î' | 'Î' => {
                plain.push('i');
                detected_tone = Some(Tone::Tone5);
            }
            'ô' | 'Ô' => {
                plain.push('o');
                detected_tone = Some(Tone::Tone5);
            }
            'û' | 'Û' => {
                plain.push('u');
                detected_tone = Some(Tone::Tone5);
            }
            '\u{0302}' => {
                detected_tone = Some(Tone::Tone5);
            }

            // Tone 6 (Caron)
            'ǎ' | 'Ǎ' => {
                plain.push('a');
                detected_tone = Some(Tone::Tone6);
            }
            'ě' | 'Ě' => {
                plain.push('e');
                detected_tone = Some(Tone::Tone6);
            }
            'ǐ' | 'Ǐ' => {
                plain.push('i');
                detected_tone = Some(Tone::Tone6);
            }
            'ǒ' | 'Ǒ' => {
                plain.push('o');
                detected_tone = Some(Tone::Tone6);
            }
            'ǔ' | 'Ǔ' => {
                plain.push('u');
                detected_tone = Some(Tone::Tone6);
            }
            '\u{030C}' => {
                detected_tone = Some(Tone::Tone6);
            }

            // Tone 7 (Macron)
            'ā' | 'Ā' => {
                plain.push('a');
                detected_tone = Some(Tone::Tone7);
            }
            'ē' | 'Ē' => {
                plain.push('e');
                detected_tone = Some(Tone::Tone7);
            }
            'ī' | 'Ī' => {
                plain.push('i');
                detected_tone = Some(Tone::Tone7);
            }
            'ō' | 'Ō' => {
                plain.push('o');
                detected_tone = Some(Tone::Tone7);
            }
            'ū' | 'Ū' => {
                plain.push('u');
                detected_tone = Some(Tone::Tone7);
            }
            '\u{0304}' => {
                detected_tone = Some(Tone::Tone7);
            }

            // Tone 8 (Vertical line above \u{030D})
            '\u{030D}' => {
                detected_tone = Some(Tone::Tone8);
            }

            // Tone 9 (Double acute)
            'ő' | 'Ő' => {
                plain.push('o');
                detected_tone = Some(Tone::Tone9);
            }
            'ű' | 'Ű' => {
                plain.push('u');
                detected_tone = Some(Tone::Tone9);
            }
            '\u{030B}' => {
                detected_tone = Some(Tone::Tone9);
            }

            // Optional POJ combining dot above right
            '\u{0358}' => {}

            // ASCII tone digits (fallback for numeric input e.g. "tsui2")
            '1'..='9' => {
                if detected_tone.is_none() {
                    detected_tone = Tone::from_char(c);
                }
            }

            // Regular alphabetic characters
            c if c.is_ascii_alphabetic() => {
                plain.push(c.to_ascii_lowercase());
            }

            // Skip unknown symbols, hyphens, and whitespace
            _ => {}
        }
    }

    let final_tone = match detected_tone {
        Some(t) => t,
        None => {
            // Unaccented syllable: check for stop coda ('p', 't', 'k', 'h')
            if plain.ends_with('p')
                || plain.ends_with('t')
                || plain.ends_with('k')
                || plain.ends_with('h')
            {
                Tone::Tone4
            } else {
                Tone::Tone1
            }
        }
    };

    (plain, final_tone)
}

/// Parses a full hyphenated, spaced, or compound Tâi-lô word/phrase into standardized
/// hyphenated plain letters and a sequence of [`Tone`]s.
pub fn parse_tailo_word(word: &str) -> (String, Vec<Tone>) {
    let mut plain_syllables = Vec::new();
    let mut tones = Vec::new();

    // Normalize neutral tone marker '--' to standard hyphen '-'
    let normalized = word.replace("--", "-");

    // Split on hyphens or spaces
    for part in normalized.split(|c| c == '-' || c == ' ') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (plain, tone) = parse_tailo_syllable(trimmed);
        if !plain.is_empty() {
            plain_syllables.push(plain);
            tones.push(tone);
        }
    }

    (plain_syllables.join("-"), tones)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_parsing_from_char() {
        assert_eq!(Tone::from_char('1'), Some(Tone::Tone1));
        assert_eq!(Tone::from_char('2'), Some(Tone::Tone2));
        assert_eq!(Tone::from_char('8'), Some(Tone::Tone8));
        assert_eq!(Tone::from_char('0'), None);
        assert_eq!(Tone::from_char('a'), None);
    }

    #[test]
    fn test_tone_parsing_from_u8() {
        assert_eq!(Tone::from_u8(1), Some(Tone::Tone1));
        assert_eq!(Tone::from_u8(7), Some(Tone::Tone7));
        assert_eq!(Tone::from_u8(0), None);
        assert_eq!(Tone::from_u8(10), None);
    }

    #[test]
    fn test_tone_combining_marks() {
        assert_eq!(Tone::Tone1.combining_mark(), None);
        assert_eq!(Tone::Tone4.combining_mark(), None);
        assert_eq!(Tone::Tone2.combining_mark(), Some('\u{0301}'));
        assert_eq!(Tone::Tone3.combining_mark(), Some('\u{0300}'));
        assert_eq!(Tone::Tone5.combining_mark(), Some('\u{0302}'));
        assert_eq!(Tone::Tone6.combining_mark(), Some('\u{030C}'));
        assert_eq!(Tone::Tone7.combining_mark(), Some('\u{0304}'));
        assert_eq!(Tone::Tone8.combining_mark(), Some('\u{030D}'));
    }

    #[test]
    fn test_find_tone_target_index_vowel_priority() {
        // 'a' has highest priority
        assert_eq!(find_tone_target_index("a"), Some(0));
        assert_eq!(find_tone_target_index("tai"), Some(1));
        assert_eq!(find_tone_target_index("piau"), Some(2));
        assert_eq!(find_tone_target_index("toan"), Some(2));

        // 'oo' marked on first 'o'
        assert_eq!(find_tone_target_index("koo"), Some(1));
        assert_eq!(find_tone_target_index("thoo"), Some(2));

        // 'iu' and 'ui' marked on the second vowel
        assert_eq!(find_tone_target_index("tsui"), Some(3));
        assert_eq!(find_tone_target_index("kiu"), Some(2));

        // 'e' and 'o'
        assert_eq!(find_tone_target_index("kue"), Some(2));
        assert_eq!(find_tone_target_index("sio"), Some(2));
        assert_eq!(find_tone_target_index("te"), Some(1));
        assert_eq!(find_tone_target_index("to"), Some(1));

        // 'i' and 'u'
        assert_eq!(find_tone_target_index("si"), Some(1));
        assert_eq!(find_tone_target_index("tu"), Some(1));
    }

    #[test]
    fn test_find_tone_target_index_syllabic_nasals() {
        // 'ng' marked on 'n'
        assert_eq!(find_tone_target_index("ng"), Some(0));
        assert_eq!(find_tone_target_index("sng"), Some(1));
        assert_eq!(find_tone_target_index("tshng"), Some(3));
        assert_eq!(find_tone_target_index("mng"), Some(1));

        // 'm' marked on 'm'
        assert_eq!(find_tone_target_index("m"), Some(0));
        assert_eq!(find_tone_target_index("hm"), Some(1));
    }

    #[test]
    fn test_find_tone_target_index_invalid() {
        assert_eq!(find_tone_target_index(""), None);
        assert_eq!(find_tone_target_index("xyz"), None);
        assert_eq!(find_tone_target_index("tsh"), None);
    }

    #[test]
    fn test_apply_tone_basic_and_accents() {
        // Tone 2 (acute \u{0301}) on 'i' in "tsui" -> "tsui\u{0301}"
        assert_eq!(apply_tone("tsui", Tone::Tone2), "tsui\u{0301}");

        // Tone 5 (circumflex \u{0302}) on 'u' in "kiu" -> "kiu\u{0302}"
        assert_eq!(apply_tone("kiu", Tone::Tone5), "kiu\u{0302}");

        // Tone 3 (grave \u{0300}) on 'a' in "tai" -> "ta\u{0300}i"
        assert_eq!(apply_tone("tai", Tone::Tone3), "ta\u{0300}i");

        // Tone 5 on first 'o' in "koo" -> "ko\u{0302}o"
        assert_eq!(apply_tone("koo", Tone::Tone5), "ko\u{0302}o");

        // Tone 7 (macron \u{0304}) on 'm' -> "m\u{0304}"
        assert_eq!(apply_tone("m", Tone::Tone7), "m\u{0304}");

        // Tone 8 (vertical line above \u{030D}) on 'n' in "sng" -> "sn\u{030D}g"
        assert_eq!(apply_tone("sng", Tone::Tone8), "sn\u{030D}g");
    }

    #[test]
    fn test_apply_tone_no_mark_and_invalid() {
        // Tone 1 and Tone 4 do not add diacritical marks
        assert_eq!(apply_tone("tsui", Tone::Tone1), "tsui");
        assert_eq!(apply_tone("tsui", Tone::Tone4), "tsui");

        // Invalid syllable without vowels or syllabic nasals returns unchanged
        assert_eq!(apply_tone("xyz", Tone::Tone2), "xyz");
        assert_eq!(apply_tone("", Tone::Tone2), "");
    }

    #[test]
    fn test_split_syllable_and_tone() {
        assert_eq!(
            split_syllable_and_tone("tsui2"),
            ("tsui", Some(Tone::Tone2))
        );
        assert_eq!(split_syllable_and_tone("tsui"), ("tsui", None));
        assert_eq!(
            split_syllable_and_tone("lim-tsui2"),
            ("lim-tsui", Some(Tone::Tone2))
        );
        assert_eq!(split_syllable_and_tone("2"), ("", Some(Tone::Tone2)));
        assert_eq!(split_syllable_and_tone("tsui0"), ("tsui0", None));
        assert_eq!(split_syllable_and_tone(""), ("", None));
    }

    #[test]
    fn test_parse_tailo_syllable_diacritics() {
        // Tone 2 (acute)
        assert_eq!(
            parse_tailo_syllable("tsuí"),
            ("tsui".to_string(), Tone::Tone2)
        );
        assert_eq!(parse_tailo_syllable("á"), ("a".to_string(), Tone::Tone2));
        assert_eq!(parse_tailo_syllable("ḿ"), ("m".to_string(), Tone::Tone2));
        assert_eq!(parse_tailo_syllable("ń"), ("n".to_string(), Tone::Tone2));

        // Tone 3 (grave)
        assert_eq!(
            parse_tailo_syllable("puànn"),
            ("puann".to_string(), Tone::Tone3)
        );
        assert_eq!(
            parse_tailo_syllable("kàu"),
            ("kau".to_string(), Tone::Tone3)
        );
        assert_eq!(parse_tailo_syllable("àm"), ("am".to_string(), Tone::Tone3));

        // Tone 5 (circumflex)
        assert_eq!(
            parse_tailo_syllable("lâi"),
            ("lai".to_string(), Tone::Tone5)
        );
        assert_eq!(parse_tailo_syllable("ê"), ("e".to_string(), Tone::Tone5));
        assert_eq!(
            parse_tailo_syllable("lâng"),
            ("lang".to_string(), Tone::Tone5)
        );

        // Tone 6 (caron)
        assert_eq!(
            parse_tailo_syllable("tóo"),
            ("too".to_string(), Tone::Tone2)
        );
        assert_eq!(parse_tailo_syllable("tǎ"), ("ta".to_string(), Tone::Tone6));

        // Tone 7 (macron)
        assert_eq!(
            parse_tailo_syllable("tuān"),
            ("tuan".to_string(), Tone::Tone7)
        );
        assert_eq!(parse_tailo_syllable("ē"), ("e".to_string(), Tone::Tone7));
        assert_eq!(parse_tailo_syllable("sī"), ("si".to_string(), Tone::Tone7));

        // Tone 8 (combining vertical line \u{030D})
        assert_eq!(
            parse_tailo_syllable("tsi̍t"),
            ("tsit".to_string(), Tone::Tone8)
        );
        assert_eq!(
            parse_tailo_syllable("gue̍h"),
            ("gueh".to_string(), Tone::Tone8)
        );
        assert_eq!(
            parse_tailo_syllable("ji̍t"),
            ("jit".to_string(), Tone::Tone8)
        );

        // Tone 9 (double acute)
        assert_eq!(
            parse_tailo_syllable("kőo"),
            ("koo".to_string(), Tone::Tone9)
        );
    }

    #[test]
    fn test_parse_tailo_syllable_unaccented_coda() {
        // Stop codas (-p, -t, -k, -h) without accent defaults to Tone 4 (yin-ru)
        assert_eq!(parse_tailo_syllable("it"), ("it".to_string(), Tone::Tone4));
        assert_eq!(
            parse_tailo_syllable("sat"),
            ("sat".to_string(), Tone::Tone4)
        );
        assert_eq!(
            parse_tailo_syllable("sak"),
            ("sak".to_string(), Tone::Tone4)
        );
        assert_eq!(
            parse_tailo_syllable("beh"),
            ("beh".to_string(), Tone::Tone4)
        );
        assert_eq!(
            parse_tailo_syllable("tsiah"),
            ("tsiah".to_string(), Tone::Tone4)
        );

        // Open or nasal codas without accent defaults to Tone 1 (yin-ping)
        assert_eq!(parse_tailo_syllable("to"), ("to".to_string(), Tone::Tone1));
        assert_eq!(
            parse_tailo_syllable("lim"),
            ("lim".to_string(), Tone::Tone1)
        );
        assert_eq!(
            parse_tailo_syllable("tai"),
            ("tai".to_string(), Tone::Tone1)
        );
        assert_eq!(
            parse_tailo_syllable("sann"),
            ("sann".to_string(), Tone::Tone1)
        );
    }

    #[test]
    fn test_parse_tailo_syllable_numeric_fallback() {
        assert_eq!(
            parse_tailo_syllable("tsui2"),
            ("tsui".to_string(), Tone::Tone2)
        );
        assert_eq!(
            parse_tailo_syllable("lim1"),
            ("lim".to_string(), Tone::Tone1)
        );
        assert_eq!(
            parse_tailo_syllable("tsit8"),
            ("tsit".to_string(), Tone::Tone8)
        );
    }

    #[test]
    fn test_parse_tailo_word_compound_and_neutral() {
        // Multi-syllable compound with hyphens
        let (plain, tones) = parse_tailo_word("it-to-lióng-tuān");
        assert_eq!(plain, "it-to-liong-tuan");
        assert_eq!(
            tones,
            vec![Tone::Tone4, Tone::Tone1, Tone::Tone2, Tone::Tone7]
        );

        // Multi-syllable compound with spaces
        let (plain, tones) = parse_tailo_word("tsi̍t gue̍h-ji̍t");
        assert_eq!(plain, "tsit-gueh-jit");
        assert_eq!(tones, vec![Tone::Tone8, Tone::Tone8, Tone::Tone8]);

        // Neutral tone prefix '--'
        let (plain, tones) = parse_tailo_word("kú--lâi");
        assert_eq!(plain, "ku-lai");
        assert_eq!(tones, vec![Tone::Tone2, Tone::Tone5]);

        // Leading neutral tone '--'
        let (plain, tones) = parse_tailo_word("--tsi̍t-kuá");
        assert_eq!(plain, "tsit-kua");
        assert_eq!(tones, vec![Tone::Tone8, Tone::Tone2]);

        // Complex multi-syllable phrase
        let (plain, tones) = parse_tailo_word("tsi̍t-puànn-ê-á");
        assert_eq!(plain, "tsit-puann-e-a");
        assert_eq!(
            tones,
            vec![Tone::Tone8, Tone::Tone3, Tone::Tone5, Tone::Tone2]
        );
    }
}
