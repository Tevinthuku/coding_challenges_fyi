use std::error::Error;

mod parser;
mod tokenizer;

pub fn parse_json(input: &'static str) -> Result<(), Box<dyn Error>> {
    let tokens = tokenizer::tokenize(input)?;
    parser::parse(tokens)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parse_json;
    use rstest::rstest;

    #[rstest]
    #[case(include_str!("../tests/step1/valid.json"), true)]
    #[case(include_str!("../tests/step1/invalid.json"), false)]
    #[case(include_str!("../tests/step2/valid.json"), true)]
    #[case(include_str!("../tests/step2/valid2.json"), true)]
    #[case(include_str!("../tests/step2/invalid.json"), false)]
    #[case(include_str!("../tests/step2/invalid2.json"), false)]
    #[case(include_str!("../tests/step3/valid.json"), true)]
    #[case(include_str!("../tests/step3/invalid.json"), false)]

    fn test_parsing_input(#[case] input: &'static str, #[case] expected: bool) {
        let is_ok = parse_json(input)
            .map_err(|err| {
                println!("{err:?}");
                err
            })
            .is_ok();

        assert_eq!(is_ok, expected)
    }
}
