use anyhow::Result;
use std::path::Path;

use crate::{
    models::Status,
    sandbox::{ExecuteConfig, Sandbox},
};

pub fn run_checker(
    checker_path: &Path,
    testcase_input: &str,
    user_output: &str,
    testcase_output: &str,
) -> Result<Status> {
    let sandbox = Sandbox::create(0)?;

    // Copy checker into the sandbox and return new path.
    let sandbox_checker_path = sandbox.path.join("checker");
    std::fs::copy(checker_path, &sandbox_checker_path)?;

    let input_path = sandbox.path.join("in.txt");
    std::fs::write(&input_path, testcase_input)?;

    let output_path = sandbox.path.join("out.txt");
    std::fs::write(&output_path, user_output)?;

    let answer_path = sandbox.path.join("ans.txt");
    std::fs::write(&answer_path, testcase_output)?;

    let output = sandbox.execute(
        &ExecuteConfig {
            ..Default::default()
        },
        vec![
            "./checker".to_string(),
            "./in.txt".to_string(),
            "./out.txt".to_string(),
            "./ans.txt".to_string(),
        ],
    )?;
    let output = String::from_utf8_lossy(&output.stdout);

    Sandbox::cleanup(0)?;

    if output.starts_with("ok") {
        Ok(Status::AC)
    } else {
        Ok(Status::WA)
    }
}
