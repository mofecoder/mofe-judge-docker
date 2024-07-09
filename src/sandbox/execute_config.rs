/// http://www.ucw.cz/moe/isolate.1.html に参照すること
pub struct ExecuteConfig {
    pub meta: Option<String>,
    pub mem: Option<u64>,
    pub time: Option<f64>,
    pub wall_time: Option<f64>,
    pub extra_time: Option<f64>,
    pub stack: Option<u64>,
    pub fsize: Option<u64>,
    pub quota: Option<u64>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub chdir: Option<String>,
    /// ここが 0 と設定すると、制限がなくなる
    pub processes: Option<u64>,
    pub stderr_to_stdout: bool,
    pub verbose: bool,
    pub silent: bool,

    pub env: Option<Vec<String>>,
    pub full_env: bool,

    pub dir: Option<Vec<String>>,
    pub no_default_dir: bool,

    pub cg: bool,
    pub cg_mem: Option<u64>,
}

impl Default for ExecuteConfig {
    fn default() -> Self {
        ExecuteConfig {
            meta: None,
            mem: None,
            time: None,
            wall_time: None,
            extra_time: None,
            stack: None,
            fsize: None,
            quota: None,
            stdin: None,
            stdout: None,
            stderr: None,
            stderr_to_stdout: false,
            chdir: None,
            processes: Some(0),
            verbose: false,
            silent: false,

            env: None,
            full_env: false,

            dir: None,
            no_default_dir: false,

            cg: true,
            cg_mem: None,
        }
    }
}

impl ExecuteConfig {
    pub fn build_flags(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        macro_rules! push_arg {
            ($a:expr, $b:expr) => {
                if let Some(value) = &$b {
                    args.push(format!($a, value));
                }
            };
        }

        macro_rules! push_flag {
            ($a:expr, $b:expr) => {
                if $b {
                    args.push($a.to_string())
                }
            };
        }

        // 引数を処理する
        push_arg!("--meta={}", self.meta);
        push_arg!("--mem={}", self.mem);
        push_arg!("--time={}", self.time);
        push_arg!("--wall-time={}", self.wall_time);
        push_arg!("--extra-time={}", self.extra_time);
        push_arg!("--stack={}", self.stack);
        push_arg!("--fsize={}", self.fsize);
        push_arg!("--quota={}", self.quota);
        push_arg!("--stdin={}", self.stdin);
        push_arg!("--stdout={}", self.stdout);
        push_arg!("--stderr={}", self.stderr);
        push_arg!("--chdir={}", self.chdir);
        push_arg!("--processes={}", self.processes);
        push_arg!("--cg-mem={}", self.cg_mem);

        // フラグを処理する
        push_flag!("--stderr-to-stdout", self.stderr_to_stdout);
        push_flag!("--verbose", self.verbose);
        push_flag!("--silent", self.silent);
        push_flag!("--full-env", self.full_env);
        push_flag!("--no-default-dir", self.no_default_dir);
        push_flag!("--cg", self.cg);

        // 一個以上使える引数を処理する
        if let Some(env) = &self.env {
            for arg in env {
                args.push(format!("--env={}", &arg));
            }
        }
        if let Some(dir) = &self.dir {
            for arg in dir {
                args.push(format!("--dir={}", &arg));
            }
        }

        args
    }
}
