mod run;
use std::{error::Error, ffi::OsStr, fs, path::PathBuf};

use run::run;

pub struct RegressionTestSet {
    /// 要测试的程序
    pub program: String,
    /// 用来参考的程序（认为其输出永远正确）
    pub ref_program: String,
    /// 测试用例所在文件夹（只会用到其中的`*.in`）
    pub cases: String,
}

pub struct GivenOutputTestSet {
    /// 要测试的程序
    pub program: String,
    /// 测试用例所在文件夹（只会用到其中的`*.in`和`*.out`）
    pub cases: String,
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

pub fn check_regression(test_set: &RegressionTestSet) -> Result<Vec<CheckedCase>, Box<dyn Error>> {
    fs::read_dir(&test_set.cases)?
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension() == Some(OsStr::new("in")))
        .map(|path| {
            let input = fs::read_to_string(&path)?;

            let your = run(&test_set.program, &input)?;
            let expected = run(&test_set.ref_program, &input)?;

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

pub fn check_given_output(
    test_set: &GivenOutputTestSet,
) -> Result<Vec<CheckedCase>, Box<dyn Error>> {
    fs::read_dir(&test_set.cases)?
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension() == Some(OsStr::new("in")))
        .map(|path| {
            let input = fs::read_to_string(&path)?;

            let your = run(&test_set.program, &input)?;
            let expected = fs::read_to_string(path.with_extension("out"))?;

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
