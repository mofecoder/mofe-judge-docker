use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize)]
pub struct TestcaseResult {
    pub status: Status,
    pub cmd_result: CmdResult,
}

#[derive(Deserialize, Serialize)]
pub struct CmdResult {
    pub time: i32,          // ms
    pub stdout_size: usize, // byte
    pub mem_usage: i32,     // byte
    pub ok: bool,           // exit_code == 0
    pub message: String,    // コンパイルメッセージ
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Problem {
    pub problem_id: i64,
    pub uuid: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Testcase {
    pub testcase_id: i64,
    pub name: String,
}

#[allow(dead_code)]
pub struct TestcaseSets {
    pub id: i64,
    pub points: u64,
}

#[allow(dead_code)]
pub struct TestcaseTestcaseSets {
    pub testcase_id: i64,
    pub testcase_set_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct CompileRequest {
    pub submit_id: i64,
    pub cmd: String, // コンパイルコマンド or 実行コマンド
}

#[derive(Serialize, Deserialize)]
pub struct CompileResponse(pub CmdResult);

#[derive(Serialize, Deserialize)]
pub struct DownloadRequest {
    pub submit_id: i64,
    pub code_path: String, // gcp 上のパス
    pub filename: String,  // Main.ext
}

#[derive(Serialize, Deserialize)]
pub struct JudgeRequest {
    pub submit_id: i64,
    pub cmd: String,     // コンパイルコマンド or 実行コマンド
    pub time_limit: i32, // 実行制限時間
    pub mem_limit: i32,  // メモリ制限

    pub testcases: Vec<Testcase>, // pub testcase: Testcase,
    pub problem: Problem,         // pub problem: Problem,
}

#[derive(Serialize, Deserialize)]
pub struct JudgeResponse(pub Vec<TestcaseResult>);

#[derive(Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum Status {
    AC,
    TLE,
    MLE,
    OLE,
    WA,
    RE,
    CE,
    IE,
}

impl Status {
    pub fn value(&self) -> String {
        match *self {
            Status::AC => "AC".to_string(),
            Status::TLE => "TLE".to_string(),
            Status::MLE => "MLE".to_string(),
            Status::OLE => "OLE".to_string(),
            Status::WA => "WA".to_string(),
            Status::RE => "RE".to_string(),
            Status::CE => "CE".to_string(),
            Status::IE => "IE".to_string(),
        }
    }
}
