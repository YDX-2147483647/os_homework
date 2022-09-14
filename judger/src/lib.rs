mod run;
use std::{error::Error, ffi::OsStr, fs, path::PathBuf};

use run::run;

pub struct Config {
    /// 要测试的程序
    pub program: String,
    /// 用来参考的程序（认为其输出永远正确）
    pub ref_program: String,
    /// 测试用例所在文件夹（只会用到其中的`*.in`）
    pub test_cases: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // ignore args[0]

        let program = match args.next() {
            Some(a) => a,
            None => return Err("Didn't give the program to test"),
        };
        let ref_program = match args.next() {
            Some(a) => a,
            None => return Err("Didn't give the program for reference"),
        };
        let test_cases = match args.next() {
            Some(a) => a,
            None => return Err("Didn't give the directory of test cases"),
        };

        if args.next().is_some() {
            return Err("Too many arguments");
        }

        Ok(Config {
            program,
            ref_program,
            test_cases,
        })
    }
}

#[derive(Debug)]
pub struct WrongAnswerResult {
    pub expected: String,
    pub your: String,
}

#[derive(Debug)]
pub enum CheckResult {
    Accepted,
    WrongAnswer(WrongAnswerResult),
}

pub struct CheckedCase {
    pub path: PathBuf,
    pub result: CheckResult,
}

pub fn check(config: &Config) -> Result<Vec<CheckedCase>, Box<dyn Error>> {
    fs::read_dir(&config.test_cases)?
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension() == Some(OsStr::new("in")))
        .map(|path| {
            let input = fs::read_to_string(&path)?;

            let your = run(&config.program, &input)?;
            let expected = run(&config.ref_program, &input)?;

            if your == expected {
                Ok(CheckedCase {
                    result: CheckResult::Accepted,
                    path,
                })
            } else {
                Ok(CheckedCase {
                    path,
                    result: CheckResult::WrongAnswer(WrongAnswerResult { expected, your }),
                })
            }
        })
        .collect()
}
