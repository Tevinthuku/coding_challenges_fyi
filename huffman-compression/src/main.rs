pub mod encode_decode;
mod prefix_code_table;

use std::{env, error::Error};

use encode_decode::encode_and_decode;

const SKIP_CHALLENGE_PATH: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(SKIP_CHALLENGE_PATH);

    let input_file_name = args.next().ok_or("failed to get input_file_name")?;

    let output_file_name = args.next().ok_or("failed to get output_file_name")?;
    encode_and_decode(input_file_name, output_file_name)?;
    Ok(())
}
