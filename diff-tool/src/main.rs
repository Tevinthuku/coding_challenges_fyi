use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);

    let original_file = args.next().ok_or("Failed to get the original file")?;
    let f = File::open(original_file)?;
    let f = BufReader::new(f);
    let content1 = f.lines().collect::<Result<Vec<_>, _>>()?;

    let new_file = args.next().ok_or("Failed to get the new file")?;
    let f = File::open(new_file)?;
    let f = BufReader::new(f);
    let content2 = f.lines().collect::<Result<Vec<_>, _>>()?;

    let diff = differences(&content1, &content2);

    diff.for_each(|item| {
        println!("{}", item);
    });
    Ok(())
}

fn differences<'a>(
    lines1: &'a [impl AsRef<str>],
    lines2: &'a [impl AsRef<str>],
) -> impl Iterator<Item = String> + 'a {
    let common = longest_common_sequence_for_many(lines1, lines2);

    lines1
        .iter()
        .map(|line1| (line1.as_ref(), true))
        .chain(lines2.iter().map(|line2| (line2.as_ref(), false)))
        .filter_map(move |(line, is_original)| {
            if common.contains(line) {
                None
            } else {
                Some(format!("{} {}", if is_original { "<" } else { ">" }, line))
            }
        })
}

fn longest_common_sequence_for_many<'a>(
    lines1: &'a [impl AsRef<str>],
    lines2: &'a [impl AsRef<str>],
) -> HashSet<&'a str> {
    lines1
        .iter()
        .flat_map(|line1| {
            lines2
                .iter()
                .filter_map(|line2| longest_common_sequence(line1.as_ref(), line2.as_ref()))
        })
        .collect::<HashSet<_>>()
}

fn longest_common_sequence<'a>(str1: &'a str, str2: &'a str) -> Option<&'a str> {
    let peek1 = str1.chars().peekable();
    let mut peek2 = str2.chars().peekable();

    for c1 in peek1 {
        peek2.next_if_eq(&c1)?;
    }

    Some(str1)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("Hello world", "Hello world", "Hello world")]
    fn test_least_common_sequence(#[case] str1: &str, #[case] str2: &str, #[case] expected: &str) {
        let result = super::longest_common_sequence(str1, str2);
        assert_eq!(result, Some(expected));
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
        let lines_1 = lines_1.iter().map(ToString::to_string).collect::<Vec<_>>();
        let lines_2 = lines_2.iter().map(ToString::to_string).collect::<Vec<_>>();
        let result = super::longest_common_sequence_for_many(&lines_1, &lines_2);
        let expected_lcs = expected_lcs.iter().copied().collect::<HashSet<_>>();
        assert_eq!(result, expected_lcs);
    }

    #[test]
    fn test_least_common_sequence_for_many_2() {
        let lines1 = ["This is a test which contains:", "this is the lcs"];
        let lines2 = vec!["this is the lcs", "we're testing"];
        let expected = ["this is the lcs"].into_iter().collect();
        let result = super::longest_common_sequence_for_many(&lines1, &lines2);
        assert_eq!(result, expected);
    }

    #[fixture]
    #[once]
    fn expected_diff() -> &'static [&'static str] {
        &[
            "< Coding Challenges helps you become a better software engineer through that build real applications.",
            "> Helping you become a better software engineer through coding challenges that build real applications.",
            "< I've used or am using these coding challenges as exercise to learn a new programming language or technology.",
            "> These are challenges that I've used or am using as exercises to learn a new programming language or technology.",
        ]
    }

    #[rstest]
    fn test_differences(
        lines_1: &'static [&'static str],
        lines_2: &'static [&'static str],
        expected_diff: &'static [&'static str],
    ) {
        let result = super::differences(lines_1, lines_2)
            .sorted()
            .collect::<Vec<_>>();
        let mut expected_diff = expected_diff.to_vec();
        expected_diff.sort();
        assert_eq!(result, expected_diff);
    }
}
