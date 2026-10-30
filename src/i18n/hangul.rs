/// Maps Korean 2-beolsik input characters (Compatibility Jamo, Choseong/Jungseong Jamo,
/// and composed syllables) to their corresponding QWERTY keys.
///
/// This allows all keyboard shortcuts (Vim navigation h/j/k/l, view switching 1-4/m/w/d/a,
/// action keys c, n, r, q, t, o, etc.) to function transparently even when the user's
/// terminal IME is set to Korean.
pub fn hangul_to_qwerty(c: char) -> char {
    match c {
        // --- Hangul Compatibility Jamo: Consonants (U+3131 ~ U+314E) ---
        'ㄱ' => 'r',
        'ㄲ' => 'R',
        'ㄳ' => 'r',
        'ㄴ' => 's',
        'ㄵ' => 's',
        'ㄶ' => 's',
        'ㄷ' => 'e',
        'ㄸ' => 'E',
        'ㄹ' => 'f',
        'ㄺ' => 'f',
        'ㄻ' => 'f',
        'ㄼ' => 'f',
        'ㄽ' => 'f',
        'ㄾ' => 'f',
        'ㄿ' => 'f',
        'ㅀ' => 'f',
        'ㅁ' => 'a',
        'ㅂ' => 'q',
        'ㅃ' => 'Q',
        'ㅄ' => 'q',
        'ㅅ' => 't',
        'ㅆ' => 'T',
        'ㅇ' => 'd',
        'ㅈ' => 'w',
        'ㅉ' => 'W',
        'ㅊ' => 'c',
        'ㅋ' => 'z',
        'ㅌ' => 'x',
        'ㅍ' => 'v',
        'ㅎ' => 'g',

        // --- Hangul Compatibility Jamo: Vowels (U+314F ~ U+3163) ---
        'ㅏ' => 'k',
        'ㅐ' => 'o',
        'ㅑ' => 'i',
        'ㅒ' => 'O',
        'ㅓ' => 'j',
        'ㅔ' => 'p',
        'ㅕ' => 'u',
        'ㅖ' => 'P',
        'ㅗ' => 'h',
        'ㅘ' => 'h',
        'ㅙ' => 'h',
        'ㅚ' => 'h',
        'ㅛ' => 'y',
        'ㅜ' => 'n',
        'ㅝ' => 'n',
        'ㅞ' => 'n',
        'ㅟ' => 'n',
        'ㅠ' => 'b',
        'ㅡ' => 'm',
        'ㅢ' => 'm',
        'ㅣ' => 'l',

        // --- Choseong Jamo (U+1100 ~ U+1112) ---
        '\u{1100}' => 'r', // ㄱ
        '\u{1101}' => 'R', // ㄲ
        '\u{1102}' => 's', // ㄴ
        '\u{1103}' => 'e', // ㄷ
        '\u{1104}' => 'E', // ㄸ
        '\u{1105}' => 'f', // ㄹ
        '\u{1106}' => 'a', // ㅁ
        '\u{1107}' => 'q', // ㅂ
        '\u{1108}' => 'Q', // ㅃ
        '\u{1109}' => 't', // ㅅ
        '\u{110A}' => 'T', // ㅆ
        '\u{110B}' => 'd', // ㅇ
        '\u{110C}' => 'w', // ㅈ
        '\u{110D}' => 'W', // ㅉ
        '\u{110E}' => 'c', // ㅊ
        '\u{110F}' => 'z', // ㅋ
        '\u{1110}' => 'x', // ㅌ
        '\u{1111}' => 'v', // ㅍ
        '\u{1112}' => 'g', // ㅎ

        // --- Jungseong Jamo (U+1161 ~ U+1175) ---
        '\u{1161}' => 'k', // ㅏ
        '\u{1162}' => 'o', // ㅐ
        '\u{1163}' => 'i', // ㅑ
        '\u{1164}' => 'O', // ㅒ
        '\u{1165}' => 'j', // ㅓ
        '\u{1166}' => 'p', // ㅔ
        '\u{1167}' => 'u', // ㅕ
        '\u{1168}' => 'P', // ㅖ
        '\u{1169}' => 'h', // ㅗ
        '\u{116A}' => 'h', // ㅘ
        '\u{116B}' => 'h', // ㅙ
        '\u{116C}' => 'h', // ㅚ
        '\u{116D}' => 'y', // ㅛ
        '\u{116E}' => 'n', // ㅜ
        '\u{116F}' => 'n', // ㅝ
        '\u{1170}' => 'n', // ㅞ
        '\u{1171}' => 'n', // ㅟ
        '\u{1172}' => 'b', // ㅠ
        '\u{1173}' => 'm', // ㅡ
        '\u{1174}' => 'm', // ㅢ
        '\u{1175}' => 'l', // ㅣ

        // --- Composed Hangul Syllables (U+AC00 ~ U+D7A3) ---
        c if ('\u{AC00}'..='\u{D7A3}').contains(&c) => {
            let code = c as u32 - 0xAC00;
            let choseong_idx = code / (21 * 28);
            match choseong_idx {
                0 => 'r',  // ㄱ
                1 => 'R',  // ㄲ
                2 => 's',  // ㄴ
                3 => 'e',  // ㄷ
                4 => 'E',  // ㄸ
                5 => 'f',  // ㄹ
                6 => 'a',  // ㅁ
                7 => 'q',  // ㅂ
                8 => 'Q',  // ㅃ
                9 => 't',  // ㅅ
                10 => 'T', // ㅆ
                11 => 'd', // ㅇ
                12 => 'w', // ㅈ
                13 => 'W', // ㅉ
                14 => 'c', // ㅊ
                15 => 'z', // ㅋ
                16 => 'x', // ㅌ
                17 => 'v', // ㅍ
                18 => 'g', // ㅎ
                _ => c,
            }
        }

        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hangul_vim_navigation() {
        assert_eq!(hangul_to_qwerty('ㅗ'), 'h');
        assert_eq!(hangul_to_qwerty('ㅓ'), 'j');
        assert_eq!(hangul_to_qwerty('ㅏ'), 'k');
        assert_eq!(hangul_to_qwerty('ㅣ'), 'l');
    }

    #[test]
    fn test_hangul_shortcuts() {
        assert_eq!(hangul_to_qwerty('ㅂ'), 'q');
        assert_eq!(hangul_to_qwerty('ㅅ'), 't');
        assert_eq!(hangul_to_qwerty('ㅊ'), 'c');
        assert_eq!(hangul_to_qwerty('ㅜ'), 'n');
        assert_eq!(hangul_to_qwerty('ㄱ'), 'r');
        assert_eq!(hangul_to_qwerty('ㅐ'), 'o');
        assert_eq!(hangul_to_qwerty('ㅡ'), 'm');
        assert_eq!(hangul_to_qwerty('ㅈ'), 'w');
        assert_eq!(hangul_to_qwerty('ㅇ'), 'd');
        assert_eq!(hangul_to_qwerty('ㅁ'), 'a');
        assert_eq!(hangul_to_qwerty('ㄷ'), 'e');
    }

    #[test]
    fn test_latin_chars_unchanged() {
        assert_eq!(hangul_to_qwerty('q'), 'q');
        assert_eq!(hangul_to_qwerty('h'), 'h');
        assert_eq!(hangul_to_qwerty('j'), 'j');
        assert_eq!(hangul_to_qwerty('k'), 'k');
        assert_eq!(hangul_to_qwerty('l'), 'l');
        assert_eq!(hangul_to_qwerty('1'), '1');
    }

    #[test]
    fn test_composed_syllables() {
        assert_eq!(hangul_to_qwerty('가'), 'r');
        assert_eq!(hangul_to_qwerty('바'), 'q');
        assert_eq!(hangul_to_qwerty('초'), 'c');
        assert_eq!(hangul_to_qwerty('사'), 't');
    }
}
