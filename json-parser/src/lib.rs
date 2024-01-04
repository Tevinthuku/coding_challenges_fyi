use std::error::Error;

mod parser;
mod tokenizer;

pub fn parse_json(input: &str) -> Result<(), Box<dyn Error>> {
    let tokens = tokenizer::tokenize(input);
    parser::parse(tokens)?;

    println!("The json is valid");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parse_json;
    use rstest::rstest;

    #[rstest]
    #[case(include_str!("../tests/step1/valid.json"), true)]
    #[case(include_str!("../tests/step1/invalid.json"), false)]

    fn test_parsing_input(#[case] input: &str, #[case] expected: bool) {
        let is_ok = parse_json(input).is_ok();

        assert_eq!(is_ok, expected)
    }
}
