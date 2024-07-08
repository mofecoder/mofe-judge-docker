use crate::models::*;
use crate::sandbox::*;
use anyhow::Result;
use std::time::Duration;
use tokio::process::Command;

// time_limit は sec 単位
pub async fn exec_execute_cmd(cmd: &str, time_limit: f64) -> Result<CmdResult> {
    let sandbox = Sandbox::create(0u32)?;

    let meta_path = std::env::current_dir()?.join("meta.txt");
    let script_path = sandbox.path.join("exec_cmd.sh");

    std::fs::write(
        &script_path,
        format!(
            "{}{}",
            r#"
#!/bin/bash
export PATH=$PATH:/usr/local/go/bin
export PATH="$HOME/.cargo/bin:$PATH"
cd /judge
"#,
            cmd,
        )
        .as_bytes(),
    )?;

    let output = sandbox.execute(
        &ExecuteConfig {
            meta: Some(meta_path.to_string_lossy().to_string()),
            time: Some(time_limit + 0.2),
            wall_time: Some(time_limit * 3.0),
            full_env: true,
            dir: Some(vec![
                format!("/judge={}:rw", crate::JUDGE_DIR.to_string_lossy()),
                "/root=/root:rw".to_string(),
                "/etc".to_string(),
                "/opt".to_string(),
                "/usr".to_string(),
            ]),
            ..Default::default()
        },
        vec!["/bin/bash".to_string(), "exec_cmd.sh".to_string()],
    )?;

    let meta: Meta = std::fs::read_to_string(&meta_path)?.parse()?;
    let message = format!(
        "isolate error\nstdout:{}\nstderr:{}\n",
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    );

    Ok(CmdResult {
        ok: meta.exitcode == Some(0),
        exit_code: match meta.exitcode {
            Some(code) => code as i32,
            None => -1,
        },
        execution_time: (meta
            .time_wall
            .unwrap_or_default()
            .max(meta.time.unwrap_or_default())
            * 1000.0)
            .floor() as i32,
        stdout_size: message.len(),
        message,
        execution_memory: meta.cg_mem.unwrap_or(0) as i32,
    })
}

// time_limit は sec 単位
pub async fn exec_compile_cmd(cmd: &str, time_limit: i32) -> Result<CmdResult> {
    let script_path = "/judge/exec_cmd.sh";

    std::fs::write(
        &script_path,
        format!(
            "{}{}",
            r#"
#!/bin/bash
export PATH=$PATH:/usr/local/go/bin
export PATH="$HOME/.cargo/bin:$PATH"
cd /judge
"#,
            cmd,
        )
        .as_bytes(),
    )?;

    let child = Command::new("/bin/bash")
        .current_dir("/judge")
        .arg("exec_cmd.sh")
        .spawn()?;
    let output = tokio::time::timeout(Duration::new(time_limit as u64, 0), async {
        child.wait_with_output().await
    })
    .await??;

    let message = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    );

    Ok(CmdResult {
        ok: output.status.success(),
        exit_code: output.status.code().unwrap_or(-1),
        execution_time: 0,
        stdout_size: message.len(),
        message,
        execution_memory: 0,
    })
}
