use anyhow::{anyhow, Result};
use std::path::Path;

use crate::{
    models::Status,
    sandbox::{ExecuteConfig, Sandbox},
};

/// Compile the checker file at `checker_source`, and place the compiled file at `checker_target`.
/// Pass the path of `testlib.h` as `testlib_path`.
/// Only C++ is supported.
pub fn compile_checker(
    checker_source: &Path,
    checker_target: &Path,
    testlib_path: &Path,
) -> Result<()> {
    let sandbox = Sandbox::create(0)?;

    let sandbox_checker_source = sandbox.path.join("checker.cpp");
    let sandbox_checker_target = sandbox.path.join("checker");
    let sandbox_testlib_path = sandbox.path.join("testlib.h");

    std::fs::copy(checker_source, &sandbox_checker_source)?;
    std::fs::copy(testlib_path, &sandbox_testlib_path)?;

    // TODO harden compilation process restrictions
    let output = sandbox.execute(
        &ExecuteConfig {
            time: Some(60.0),
            wall_time: Some(60.0),
            cg_mem: Some(1_024_000),
            // Unlimited processes is needed for compiler.
            dir: Some(vec![
                format!("/judge={}:rw", crate::JUDGE_DIR.to_string_lossy()),
            ]),
            processes: Some(0),
            full_env: true,
            ..Default::default()
        },
        vec![
            "/usr/bin/g++".to_string(),
            "-O2".to_string(),
            "checker.cpp".to_string(),
            "-o".to_string(),
            "checker".to_string(),
        ],
    )?;

    if !output.status.success() {
        // TODO confirm what to return
        let error_message = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!(
            "Failed to compile checker (error message below):\n{}",
            &error_message
        ));
    }

    std::fs::copy(&sandbox_checker_target, checker_target)?;

    Ok(())
}

/// Runs checker with the provided input, output and answer in the sandbox and return the result.
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

    dbg!("Testcase input: {}", &testcase_input);
    dbg!("User output:    {}", &user_output);
    dbg!("Testcase output:{}", &testcase_output);

    let output = sandbox.execute(
        &ExecuteConfig {
            // TODO confirm time limit and memory limit
            time: Some(1.0),
            wall_time: Some(1.0),
            cg_mem: Some(256_000),
            ..Default::default()
        },
        vec![
            "./checker".to_string(),
            "./in.txt".to_string(),
            "./out.txt".to_string(),
            "./ans.txt".to_string(),
        ],
    )?;
    dbg!("Output: {}", String::from_utf8_lossy(&output.stdout));
    let output = String::from_utf8_lossy(&output.stderr);
    dbg!("Stderr: {}", &output);

    Sandbox::cleanup(0)?;

    if output.starts_with("ok") {
        Ok(Status::AC)
    } else {
        Ok(Status::WA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compile_run_useless_checker_ac() -> Result<()> {
        let checker_source_path = PathBuf::from("/tmp/checker.cpp");
        let checker_target_path = PathBuf::from("/tmp/checker");
        let testlib_path = PathBuf::from("/tmp/testlib.h");

        std::fs::write(
            &checker_source_path,
            r#"
                #include <iostream>
                #include "testlib.h"
                int main() {
                    std::cout << AC_MESSAGE << std::endl;
                    return 0;
                }
            "#,
        )?;
        std::fs::write(
            &testlib_path,
            r#"
                #define AC_MESSAGE "ok"
            "#,
        )?;

        compile_checker(&checker_source_path, &checker_target_path, &testlib_path)?;

        let result = run_checker(&checker_target_path, "", "output", "output")?;
        assert_eq!(result, Status::AC);

        Ok(())
    }

    #[test]
    fn test_compile_run_useless_checker_wa() -> Result<()> {
        let checker_source_path = PathBuf::from("/tmp/checker.cpp");
        let checker_target_path = PathBuf::from("/tmp/checker");
        let testlib_path = PathBuf::from("/tmp/testlib.h");

        std::fs::write(
            &checker_source_path,
            r#"
                #include <iostream>
                #include "testlib.h"
                int main() {
                    std::cout << WA_MESSAGE << std::endl;
                    return 0;
                }
            "#,
        )?;
        std::fs::write(
            &testlib_path,
            r#"
                #define WA_MESSAGE "wa"
            "#,
        )?;

        compile_checker(&checker_source_path, &checker_target_path, &testlib_path)?;

        let result = run_checker(&checker_target_path, "", "output", "output")?;
        assert_eq!(result, Status::WA);

        Ok(())
    }
}
