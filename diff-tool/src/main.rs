use rayon::prelude::*;
use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    mem,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};
use std::{sync::mpsc::channel, thread};

use bytes::{Bytes, BytesMut};

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);

    let original_file = args.next().ok_or("Failed to get the original file")?;
    let second_file = args.next().ok_or("Failed to get the new file")?;
    let (tx, rx) = channel::<Bytes>();

    let thread = thread::spawn(|| read_second_file(second_file, tx).unwrap());

    let now = Instant::now();
    let f = File::open(original_file)?;
    let f = BufReader::new(f);
    let content1 = f.lines().collect::<Result<Vec<_>, _>>()?;
    let content1 = content1
        .iter()
        .map(|line| line.as_str())
        .collect::<Vec<_>>();
    handle_differences_from_chunks(&content1, rx)?;
    thread.join().unwrap();
    let elapsed = now.elapsed();
    println!("Elapsed: {}", elapsed.as_secs());
    Ok(())
}

const BUFFER_SIZE: usize = 1024 * 512;

fn read_second_file(
    original_file: String,
    bytes_sender: Sender<Bytes>,
) -> Result<(), Box<dyn Error>> {
    let file = File::open(original_file)?;
    let mut file = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut left_over = BytesMut::with_capacity(BUFFER_SIZE);

    loop {
        let bytes_filled = file.fill_buf()?;
        if bytes_filled.is_empty() {
            break;
        }
        let bytes_consumed = bytes_filled.len();

        let last_newline = bytes_filled.iter().rposition(|&b| b == b'\n');
        if let Some(new_line_idx) = last_newline {
            let bytes = &bytes_filled[..new_line_idx + 1];
            left_over.extend_from_slice(bytes);
            let bytes_to_send = mem::take(&mut left_over);
            let bytes_to_send = bytes_to_send.freeze();
            bytes_sender.send(bytes_to_send).unwrap();
            if new_line_idx + 1 < bytes_consumed {
                left_over.extend_from_slice(&bytes_filled[new_line_idx + 1..]);
            }
        }
        file.consume(bytes_consumed);
    }
    drop(bytes_sender);
    Ok(())
}

fn handle_differences_from_chunks(
    lines1: &[&str],
    receiver: Receiver<Bytes>,
) -> Result<(), Box<dyn Error>> {
    let received = receiver
        .into_iter()
        .flat_map(|chunk| {
            chunk
                .par_split(|b| *b == b'\n')
                .map(|line| std::str::from_utf8(line).map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let received = received
        .iter()
        .map(|line| line.as_str())
        .collect::<Vec<_>>();
    let diff = differences(lines1, &received);
    diff.for_each(|line| println!("{}", line));
    Ok(())
}

fn differences<'a>(
    lines1: &'a [&'a str],
    lines2: &'a [&'a str],
) -> impl ParallelIterator<Item = String> + 'a {
    let common = longest_common_sequence_for_many(lines1, lines2);

    lines1
        .par_iter()
        .map(|line1| (line1, true))
        .chain(lines2.par_iter().map(|line2| (line2, false)))
        .filter_map(move |(line, is_original)| {
            if common.contains(line) {
                None
            } else {
                Some(format!("{} {}", if is_original { "<" } else { ">" }, line))
            }
        })
}

fn longest_common_sequence_for_many<'a>(
    lines1: &'a [&'a str],
    lines2: &'a [&'a str],
) -> HashSet<&'a str> {
    lines1
        .par_iter()
        .flat_map(|line1| {
            lines2
                .par_iter()
                .filter_map(|line2| longest_common_sequence(line1, line2))
        })
        .collect::<HashSet<_>>()
}

fn longest_common_sequence<'a>(str1: &'a str, str2: &'a str) -> Option<&'a str> {
    // @Tev: TBH I'm not 100% sure about this, step1 compared character to character, however
    // the next steps seem to be concerned with the entire string itself, not just the characters within the strings that match.

    if str1 == str2 {
        return Some(str1);
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rayon::iter::ParallelIterator;
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
        let mut result = super::differences(lines_1, lines_2).collect::<Vec<_>>();
        result.sort();
        let mut expected_diff = expected_diff.to_vec();
        expected_diff.sort();
        assert_eq!(result, expected_diff);
    }
}
