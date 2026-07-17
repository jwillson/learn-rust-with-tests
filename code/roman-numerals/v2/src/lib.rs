const ALL_ROMAN_NUMERALS: [(u16, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

pub fn convert_to_roman(mut arabic: u16) -> String {
    let mut result = String::new();

    for (value, symbol) in ALL_ROMAN_NUMERALS {
        while arabic >= value {
            result.push_str(symbol);
            arabic -= value;
        }
    }

    result
}

// ANCHOR: code
pub fn convert_to_arabic(mut roman: &str) -> u16 {
    let mut arabic = 0;

    for (value, symbol) in ALL_ROMAN_NUMERALS {
        while let Some(rest) = roman.strip_prefix(symbol) {
            arabic += value;
            roman = rest;
        }
    }

    arabic
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: [(u16, &str); 20] = [
        (1, "I"),
        (2, "II"),
        (3, "III"),
        (4, "IV"),
        (5, "V"),
        (6, "VI"),
        (9, "IX"),
        (10, "X"),
        (14, "XIV"),
        (18, "XVIII"),
        (20, "XX"),
        (39, "XXXIX"),
        (40, "XL"),
        (47, "XLVII"),
        (49, "XLIX"),
        (50, "L"),
        (798, "DCCXCVIII"),
        (1006, "MVI"),
        (1984, "MCMLXXXIV"),
        (3999, "MMMCMXCIX"),
    ];

    #[test]
    fn converts_arabic_numbers_to_roman_numerals() {
        for (arabic, want) in CASES {
            let got = convert_to_roman(arabic);
            assert_eq!(got, want, "for {arabic}");
        }
    }

    // ANCHOR: test
    #[test]
    fn converts_roman_numerals_to_arabic_numbers() {
        for (want, roman) in CASES {
            let got = convert_to_arabic(roman);
            assert_eq!(got, want, "for {roman}");
        }
    }
    // ANCHOR_END: test
}
