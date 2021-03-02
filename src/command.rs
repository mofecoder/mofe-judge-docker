use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::prelude::PermissionsExt,
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use crate::model::*;
use anyhow::Result;
use tokio::time::sleep;

pub fn create_sh(cmd: &str) -> Result<()> {
    let mut f = File::create("./exec_cmd.sh")?;

    f.write_all(
        br#"
#!/bin/bash
export PATH=$PATH:/usr/local/go/bin
export PATH="$HOME/.cargo/bin:$PATH"
    "#,
    )?;
    f.write_all(&format!("{}\n", cmd).as_bytes())?;
    f.write_all(b"echo $? > exit_code.txt")?;

    let perms = Permissions::from_mode(0o777);
    f.set_permissions(perms)?;

    Ok(())
}

pub async fn exec_cmd(req: &RequestJson) -> Result<CmdResult> {
    create_sh(&req.cmd)?;

    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg("/usr/bin/time -v ./exec_cmd.sh 2>&1 | grep -E 'Maximum' | awk '{ print $6 }' > mem_usage.txt");

    let mut child = cmd.spawn();
    for _ in 1..10 {
        if child.is_ok() {
            break;
        }

        child = cmd.spawn();
        sleep(Duration::new(1, 0)).await;
    }

    let child_arc = Arc::new(Mutex::new(child.unwrap()));

    let child = child_arc.clone();
    let cmd_handler = async move {
        let start = Instant::now();
        let res = child.lock().unwrap().wait();
        let end = start.elapsed();

        (res, end.as_millis())
    };

    let timeout = async {
        sleep(Duration::new(req.time_limit as u64, 0)).await;
        ()
    };

    let time = tokio::select! {
        _ = timeout => {
            child_arc.lock().unwrap().kill()?; // todo: 子プロセスも kill
            req.time_limit as u128
        }
        res = cmd_handler => res.1
    };

    Ok(CmdResult {
        ok: String::from_utf8(std::fs::read("/userStderr.txt")?)
            .unwrap()
            .trim_end()
            == &*"0",
        time,
        message: String::from_utf8(std::fs::read("/userStderr.txt")?).unwrap(),
        mem_usage: String::from_utf8(std::fs::read("/mem_usage.txt")?)
            .unwrap()
            .trim_end()
            .parse()
            .unwrap(),
    })
}

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=748440ceb10a1797d4e5ff14c57bdfeb
