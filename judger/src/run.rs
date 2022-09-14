use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;

/// Parse program's output
fn parse_output(bytes: Vec<u8>) -> Result<String, FromUtf8Error> {
    let output = String::from_utf8(bytes)?
        .replace("\r\n", "\n")
        .trim_end()
        .to_string();
    Ok(output)
}

/// Run program and get its output
pub fn run(program: &str, input: &str) -> Result<String, Box<dyn Error>> {
    // 1. Prepare the thread.
    // stdout must be configured with `Stdio::piped` in order to use
    // `program.stdout`
    let mut program = Command::new(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // 2. Write the input.
    // Writing from another thread ensures that stdout is being read
    // at the same time, avoiding the deadlock.
    // https://doc.rust-lang.org/std/process/index.html#handling-io
    let mut stdin = program.stdin.take().expect("Failed to get stdin");
    let input = input.to_owned();
    std::thread::spawn(move || {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    });

    // 3. Ger the output.
    let output = program.wait_with_output()?;
    let output = parse_output(output.stdout)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_regular_output() {
        let bytes = " a\r\nb\r\n".as_bytes().to_vec();
        assert_eq!(parse_output(bytes).unwrap(), " a\nb");
    }

    #[test]
    fn parse_invalid_output() {
        let bytes: Vec<u8> = vec![0, 159]; // This can be your `output.stdout`.
        assert!(parse_output(bytes).is_err());
    }
}
