mod execute_config;

use execute_config::ExecuteConfig;

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

type SandboxId = u32;

pub struct Sandbox {
    pub path: PathBuf,
    pub id: SandboxId,
}

impl Sandbox {
    fn check_installation() -> Result<()> {
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

    fn cleanup(id: SandboxId) -> Result<()> {
        let ok = Command::new("isolate")
            .args(&["--cg", "--cleanup", &format!("--box-id={}", id)])
            .status()
            .and_then(|result| Ok(result.success()))
            .map_err(anyhow::Error::from)?;

        if ok {
            Ok(())
        } else {
            Err(anyhow!("Failed to cleanup sandbox {}", id))
        }
    }

    fn create(id: SandboxId) -> Result<Self> {
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

    fn execute(&self, config: &ExecuteConfig, command: Vec<String>) -> Result<std::process::Output> {
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
        let output = sandbox.execute(
            &Default::default(), 
            vec!["/usr/bin/echo".to_string(), "test_sandbox_execute".to_string()]
        ).unwrap();
        let output = String::from_utf8_lossy(&output.stdout).trim().to_string();

        assert!(output == "test_sandbox_execute");
    }
}
