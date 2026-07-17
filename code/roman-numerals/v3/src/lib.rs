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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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

    #[test]
    fn converts_roman_numerals_to_arabic_numbers() {
        for (want, roman) in CASES {
            let got = convert_to_arabic(roman);
            assert_eq!(got, want, "for {roman}");
        }
    }

    // ANCHOR: properties
    proptest! {
        #[test]
        fn converting_to_roman_and_back_is_lossless(arabic in 1..=3999u16) {
            let roman = convert_to_roman(arabic);

            prop_assert_eq!(convert_to_arabic(&roman), arabic);
        }

        #[test]
        fn never_more_than_three_consecutive_identical_symbols(arabic in 1..=3999u16) {
            let roman = convert_to_roman(arabic);

            for symbol in ["I", "V", "X", "L", "C", "D", "M"] {
                let four_in_a_row = symbol.repeat(4);
                prop_assert!(
                    !roman.contains(&four_in_a_row),
                    "found {} in {}",
                    four_in_a_row,
                    roman
                );
            }
        }
    }
    // ANCHOR_END: properties
}
