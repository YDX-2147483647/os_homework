use judger::{check, CheckResult, Config, WrongAnswerResult};
use std::io::Write;
use std::{env, process};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    match check(&config) {
        Ok(cases) => {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);

            let mut green = ColorSpec::new();
            green.set_fg(Some(Color::Green));
            let mut red = ColorSpec::new();
            red.set_fg(Some(Color::Red));

            for case in cases {
                print!("{:?}: ", case.path.file_stem().unwrap());
                match case.result {
                    CheckResult::Accepted => {
                        stdout.set_color(&green).unwrap();
                        writeln!(&mut stdout, "✓").unwrap();
                        stdout.reset().unwrap();
                    }
                    CheckResult::WrongAnswer(WrongAnswerResult { your, expected }) => {
                        stdout.set_color(&red).unwrap();
                        writeln!(&mut stdout, "✗").unwrap();
                        stdout.reset().unwrap();
                        
                        print!("  your:\n");
                        stdout.set_color(&red).unwrap();
                        writeln!(&mut stdout, "\n{}\n", your).unwrap();
                        stdout.reset().unwrap();
                     
                        print!("  expected:\n");
                        println!("\n{}\n", expected);
                    }
                }
            }
        }
        Err(err) => eprint!("Check failed: {}", err),
    }
}
