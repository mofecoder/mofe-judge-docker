use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestJson {
    pub session_id: String,
    pub cmd: String,       // コンパイルコマンド or 実行コマンド
    pub code_path: String, // gcp 上のパス
    pub filename: String,  // Main.ext
    pub problem_id: usize,
    pub time_limit: usize, // 実行制限時間
    pub mem_limit: usize,  // メモリ制限

    pub testcase: Testcase,// pub testcase: Testcase,
    pub problem: Problem, // pub problem: Problem,
}

pub struct CmdResult {
    pub time: u128,      // ms
    pub mem_usage: u64,  // MB
    pub ok: bool,        // exit_code == 0
    pub message: String, // コンパイルメッセージ
}

#[derive(Deserialize)]
pub struct Problem {
    pub problem_id: u64,
    pub uuid: String,
}

#[derive(Deserialize)]
pub struct Testcase {
    pub testcase_id: u64,
    pub name: String,
}

pub struct TestcaseSets {
    pub id: u64,
    pub points: u64,
}

pub struct TestcaseTestcaseSets {
    pub testcase_id: u64,
    pub testcase_set_id: u64
}