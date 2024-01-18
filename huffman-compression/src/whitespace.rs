pub fn unicode_decoding(input: &str) -> &str {
    match input {
        "\\u{0009}" => "\t",
        "\\u{000A}" => "\n",
        "\\u{000D}" => "\r",
        "\\u{0020}" => " ",
        c => c,
    }
}

pub fn unicode_encoding(input: char) -> String {
    match input {
        '\t' => "\\u{0009}".to_owned(),
        '\n' => "\\u{000A}".to_owned(),
        '\r' => "\\u{000D}".to_owned(),
        ' ' => "\\u{0020}".to_owned(),
        c => c.to_string(),
    }
}
