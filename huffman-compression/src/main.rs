pub mod tree;

use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self},
};

use itertools::Itertools;

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);

    let file_name = args.next().ok_or("failed to get file_name")?;
    char_count_mapping(file_name)?;
    Ok(())
}

fn char_count_mapping(file_name: impl AsRef<str>) -> Result<HashMap<char, u32>, Box<dyn Error>> {
    let content = fs::read_to_string(file_name.as_ref())?;

    let mapping = content
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .into_grouping_map_by(|&x| x)
        .fold(0_u32, |acc, _key, _value| acc + 1);

    Ok(mapping)
}

#[cfg(test)]
mod tests {
    use crate::char_count_mapping;

    #[test]
    fn test_char_count_mapping() {
        let file_name = "test-135-0.txt";
        let mut mapping = char_count_mapping(file_name).expect("Read file provided");

        assert_eq!(mapping.remove(&'X'), Some(333));
        assert_eq!(mapping.remove(&'t'), Some(223000));
    }
}
