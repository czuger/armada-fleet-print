//! Text encoding and metrics for the built-in Helvetica font with `WinAnsiEncoding`.

/// Advance widths (1/1000 em) of the printable ASCII range 0x20..=0x7E, from the Helvetica AFM.
const ASCII_WIDTHS: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // 0x20
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, // 0x30
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, // 0x40
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, // 0x50
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, // 0x60
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584, // 0x70
];

/// Width used outside ASCII, where most Helvetica glyphs are 556 units wide.
const DEFAULT_WIDTH: u16 = 556;

/// Encodes `text` in WinAnsiEncoding; characters it cannot represent become `?`.
pub(crate) fn encode(text: &str) -> Vec<u8> {
    text.chars().map(encode_char).collect()
}

/// Width in points of `text` set in Helvetica at `size`. Exact for ASCII, approximate beyond.
pub(crate) fn text_width(text: &str, size: f32) -> f32 {
    let units: u32 = encode(text)
        .into_iter()
        .map(|byte| {
            let index = usize::from(byte).wrapping_sub(0x20);
            u32::from(ASCII_WIDTHS.get(index).copied().unwrap_or(DEFAULT_WIDTH))
        })
        .sum();
    units as f32 * size / 1000.0
}

fn encode_char(c: char) -> u8 {
    match c {
        ' '..='~' | '\u{A0}'..='\u{FF}' => c as u8,
        '€' => 0x80,
        '‚' => 0x82,
        'ƒ' => 0x83,
        '„' => 0x84,
        '…' => 0x85,
        '†' => 0x86,
        '‡' => 0x87,
        'ˆ' => 0x88,
        '‰' => 0x89,
        'Š' => 0x8A,
        '‹' => 0x8B,
        'Œ' => 0x8C,
        'Ž' => 0x8E,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '˜' => 0x98,
        '™' => 0x99,
        'š' => 0x9A,
        '›' => 0x9B,
        'œ' => 0x9C,
        'ž' => 0x9E,
        'Ÿ' => 0x9F,
        _ => b'?',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_french_text() {
        assert_eq!(encode("Flotte é’œ"), b"Flotte \xE9\x92\x9C");
        assert_eq!(encode("日本"), b"??");
    }

    #[test]
    fn measures_ascii() {
        assert!((text_width("10", 24.0) - 26.688).abs() < 1e-3);
        assert!((text_width("page", 1000.0) - 2224.0).abs() < 1e-3);
    }
}
