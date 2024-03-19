use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use itertools::Itertools;

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);

    let original_file = args.next().ok_or("Failed to get the original file")?;
    let f = File::open(original_file)?;
    let f = BufReader::new(f);
    let content = f.lines().collect::<Result<Vec<_>, _>>()?;
    let content = content.iter().map(|line| line.as_str()).collect_vec();

    let new_file = args.next().ok_or("Failed to get the new file")?;
    let f = File::open(new_file)?;
    let f = BufReader::new(f);
    let content2 = f.lines().collect::<Result<Vec<_>, _>>()?;
    let content2 = content2.iter().map(|line| line.as_str()).collect_vec();

    let diff = differences(&content, &content2);
    println!("Diff: {:?}", diff);

    Ok(())
}

fn differences(lines1: &[&str], lines2: &[&str]) -> Vec<String> {
    let common = longest_common_sequence_for_many(lines1, lines2);
    let common = common
        .iter()
        .map(|line| line.as_str())
        .collect::<HashSet<_>>();

    lines1
        .iter()
        .chain(lines2)
        .filter_map(|line| {
            if common.contains(line) {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect_vec()
}

fn longest_common_sequence_for_many(lines1: &[&str], lines2: &[&str]) -> Vec<String> {
    lines1
        .iter()
        .flat_map(|line1| {
            lines2
                .iter()
                .map(|line2| longest_common_sequence(line1, line2))
                .filter(|line2| line2 == *line1)
        })
        .filter(|result| !result.is_empty())
        .collect::<Vec<_>>()
}

fn longest_common_sequence(str1: &str, str2: &str) -> String {
    let expected_capacity = str1.len().min(str2.len());
    let mut result = String::with_capacity(expected_capacity);

    let peek1 = str1.chars().peekable();
    let mut peek2 = str2.chars().peekable();
    for c1 in peek1 {
        if let Some(char) = peek2.next_if_eq(&c1) {
            result.push(char);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("Hello world", "Hello world", "Hello world")]
    #[case("ABC", "XYZ", "")]
    #[case("ABCD", "AC", "AC")]
    #[case("AABCXY", "XYZ", "XY")]
    #[case("", "", "")]
    fn test_least_common_sequence(#[case] str1: &str, #[case] str2: &str, #[case] expected: &str) {
        let result = super::longest_common_sequence(str1, str2);
        assert_eq!(result, expected);
    }

    #[fixture]
    #[once]
    fn lines_1() -> &'static [&'static str] {
        &[
            "Coding Challenges helps you become a better software engineer through that build real applications.",
            "I share a weekly coding challenge aimed at helping software engineers level up their skills through deliberate practice.",
            "I've used or am using these coding challenges as exercise to learn a new programming language or technology.",
            "Each challenge will have you writing a full application or tool. Most of which will be based on real world tools and utilities.",
        ]
    }

    #[fixture]
    #[once]
    fn lines_2() -> &'static [&'static str] {
        &[
            "Helping you become a better software engineer through coding challenges that build real applications.",
            "I share a weekly coding challenge aimed at helping software engineers level up their skills through deliberate practice.",
            "These are challenges that I've used or am using as exercises to learn a new programming language or technology.",
            "Each challenge will have you writing a full application or tool. Most of which will be based on real world tools and utilities.",
        ]
    }

    #[fixture]
    #[once]
    fn expected_lcs() -> &'static [&'static str] {
        &[
            "I share a weekly coding challenge aimed at helping software engineers level up their skills through deliberate practice.",
            "Each challenge will have you writing a full application or tool. Most of which will be based on real world tools and utilities.",
        ]
    }

    #[rstest]
    fn test_least_common_sequence_for_many(
        lines_1: &'static [&'static str],
        lines_2: &'static [&'static str],
        expected_lcs: &'static [&'static str],
    ) {
        let result = super::longest_common_sequence_for_many(lines_1, lines_2);
        assert_eq!(result, expected_lcs);
    }

    #[test]
    fn test_least_common_sequence_for_many_2() {
        let lines1 = ["This is a test which contains:", "this is the lcs"];
        let lines2 = ["this is the lcs", "we're testing"];
        let expected = ["this is the lcs"];
        let result = super::longest_common_sequence_for_many(&lines1, &lines2);
        assert_eq!(result, expected);
    }

    #[fixture]
    #[once]
    fn expected_diff() -> &'static [&'static str] {
        &[
            "Coding Challenges helps you become a better software engineer through that build real applications.",
            "Helping you become a better software engineer through coding challenges that build real applications.",
            "I've used or am using these coding challenges as exercise to learn a new programming language or technology.",
            "These are challenges that I've used or am using as exercises to learn a new programming language or technology.",
        ]
    }

    #[rstest]
    fn test_differences(
        lines_1: &'static [&'static str],
        lines_2: &'static [&'static str],
        expected_diff: &'static [&'static str],
    ) {
        let mut result = super::differences(lines_1, lines_2);
        result.sort();
        let mut expected_diff = expected_diff.to_vec();
        expected_diff.sort();
        assert_eq!(result, expected_diff);
    }
}
