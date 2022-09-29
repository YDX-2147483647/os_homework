use std::io::Write;

use clap::Parser;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use judger::{
    check_given_output, check_regression, CheckResult, GivenOutputTestSet, RegressionTestSet,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 要测试的程序
    #[clap(value_parser)]
    pub program: String,
    /// 测试用例所在文件夹（若提供`ref_program`，则只会用到其中的`*.in`）
    #[clap(value_parser)]
    pub test_cases: String,
    /// 用来参考的程序（认为其输出永远正确）。若不提供，则与`test_cases`中的`*.out`比较。
    #[clap(value_parser)]
    pub ref_program: Option<String>,
}

fn main() {
    let args = Args::parse();

    let result = match args.ref_program {
        Some(ref_program) => check_regression(&RegressionTestSet {
            program: args.program,
            ref_program,
            cases: args.test_cases,
        }),
        None => check_given_output(&GivenOutputTestSet {
            program: args.program,
            cases: args.test_cases,
        }),
    };

    match result {
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
                    CheckResult::WrongAnswer(_) => {
                        stdout.set_color(&red).unwrap();
                        writeln!(&mut stdout, "✗").unwrap();
                        stdout.reset().unwrap();
                    }
                }
            }
        }
        Err(err) => eprint!("Check failed: {}", err),
    };
}
