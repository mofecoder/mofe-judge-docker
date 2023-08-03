mod execute_config;
mod meta;

pub use execute_config::ExecuteConfig;
pub use meta::Meta;

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

type SandboxId = u32;

pub struct Sandbox {
    pub id: SandboxId,
    pub path: PathBuf,
}

impl Sandbox {
    #[allow(dead_code)]
    pub fn check_installation() -> Result<()> {
        Command::new("which")
            .arg("isolate")
            .status()
            .map_err(anyhow::Error::from)
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(anyhow!("isolate is not found. Please make sure you have installed ioi/isolate correctly."))
                }
            })
    }

    pub fn cleanup(id: SandboxId) -> Result<()> {
        let res = Command::new("isolate")
            .args(&["--cg", "--cleanup", &format!("--box-id={}", id)])
            .output();
        if res.is_err() {
            return Err(anyhow!(
                "Failed to cleanup sandbox (Failed to execute) {}\n",
                id,
            ));
        }

        let res = res.unwrap();
        if res.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to cleanup sandbox {}\n{}\n",
                id,
                String::from_utf8(res.stderr).unwrap_or("".to_string())
            ))
        }
    }

    pub fn create(id: SandboxId) -> Result<Self> {
        Self::cleanup(id)?;

        let output = Command::new("isolate")
            .args(&["--cg", "--init", &format!("--box-id={}", id)])
            .output()?;
        let mut path: PathBuf = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string()
            .into();
        // 実際ファイルを置くところはその中の box というフォルダです
        path.push("box");

        Ok(Sandbox { id, path })
    }

    pub fn execute(
        &self,
        config: &ExecuteConfig,
        command: Vec<String>,
    ) -> Result<std::process::Output> {
        let mut args = config.build_flags();
        args.push(format!("--box-id={}", self.id));
        args.push("--run".to_string());
        args.push("--".to_string());
        for segment in command {
            args.push(segment);
        }

        Command::new("isolate")
            .current_dir(self.path.as_path())
            .args(&args)
            .output()
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_installation() {
        Sandbox::check_installation().unwrap();
    }

    #[test]
    fn test_sandbox_creation() {
        Sandbox::create(0u32).unwrap();
        Sandbox::cleanup(0u32).unwrap();
    }

    #[test]
    fn test_sandbox_execute() {
        let sandbox = Sandbox::create(0u32).unwrap();
        let output = sandbox
            .execute(
                &Default::default(),
                vec![
                    "/usr/bin/echo".to_string(),
                    "test_sandbox_execute".to_string(),
                ],
            )
            .unwrap();
        let output = String::from_utf8_lossy(&output.stdout).trim().to_string();

        assert!(output == "test_sandbox_execute");
    }

    #[test]
    fn test_bash_script() {
        let sandbox = Sandbox::create(0u32).unwrap();
        let path = sandbox.path.join("test.sh");
        std::fs::write(path, "#!/bin/sh\necho test_bash_script\n").unwrap();
        let output = sandbox
            .execute(
                &Default::default(),
                vec!["/bin/sh".to_string(), "test.sh".to_string()],
            )
            .unwrap();
        let output = String::from_utf8_lossy(&output.stdout).trim().to_string();

        assert!(output == "test_bash_script");
    }

    #[test]
    fn test_bash_script_timeout() {
        let sandbox = Sandbox::create(0u32).unwrap();
        let meta_path = sandbox.path.join("meta.txt");
        let script_path = sandbox.path.join("test.sh");

        std::fs::write(&script_path, "#!/bin/sh\nsleep 5\necho test_bash_script\n").unwrap();
        let _output = sandbox
            .execute(
                &ExecuteConfig {
                    meta: Some("meta.txt".to_string()),
                    time: Some(0.01),
                    wall_time: Some(0.01),
                    processes: Some(2), // sleep は外部プロセス
                    ..Default::default()
                },
                vec!["/bin/sh".to_string(), "test.sh".to_string()],
            )
            .unwrap();

        let meta = std::fs::read_to_string(&meta_path).unwrap();
        assert!(meta.contains("status:TO"));
    }

    #[test]
    fn test_environment_variable() {
        let sandbox = Sandbox::create(0u32).unwrap();

        let output = sandbox
            .execute(
                &ExecuteConfig {
                    processes: Some(1),
                    env: Some(vec!["SANDBOX_ROOT=directory".to_string()]),
                    ..Default::default()
                },
                vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    "echo $SANDBOX_ROOT".to_string(),
                ],
            )
            .unwrap();
        let output = String::from_utf8_lossy(&output.stdout);

        assert!(output.trim() == "directory");
    }
}
