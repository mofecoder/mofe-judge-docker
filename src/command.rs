use crate::models::*;
use crate::sandbox::*;
use anyhow::Result;
use std::convert::TryInto;

pub async fn exec_cmd(cmd: &str, time_limit: i32) -> Result<CmdResult> {
    let sandbox = Sandbox::create(0u32)?;

    let meta_path = sandbox.path.join("meta.txt");
    let script_path = sandbox.path.join("exec_cmd.sh");

    std::fs::write(
        &script_path,
        format!(
            "{}{}",
            r#"
#!/bin/bash
export PATH=$PATH:/usr/local/go/bin
export PATH="$HOME/.cargo/bin:$PATH"
"#,
            cmd,
        )
        .as_bytes(),
    )?;

    let output = sandbox.execute(
        &ExecuteConfig {
            meta: Some("meta.txt".to_string()),
            time: Some(time_limit.try_into()?),
            wall_time: Some(time_limit.try_into()?),
            ..Default::default()
        },
        vec!["/bin/bash".to_string(), "exec_cmd.sh".to_string()],
    )?;

    let meta: Meta = std::fs::read_to_string(&meta_path)?.parse()?;
    let message = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(CmdResult {
        ok: meta.exitcode == Some(0),
        execution_time: meta.time.unwrap_or(0.0).floor() as i32,
        stdout_size: message.len(),
        message,
        execution_memory: meta.cg_mem.unwrap_or(0) as i32,
    })
}

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=748440ceb10a1797d4e5ff14c57bdfeb
