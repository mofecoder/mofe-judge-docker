use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{collections::HashMap, fmt};

#[derive(Deserialize, Serialize)]
pub struct TestcaseResult {
    pub status: Status,
    pub cmd_result: CmdResult,
}

#[derive(Deserialize, Serialize)]
pub struct CmdResult {
    pub execution_time: i32,   // ms
    pub stdout_size: usize,    // byte
    pub execution_memory: i32, // KB
    pub ok: bool,              // exit_code == 0
    pub message: String,       // コンパイルメッセージ
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

#[derive(sqlx::FromRow)]
pub struct TestcaseSets {
    pub id: i64,
    pub points: u64,
}

#[derive(sqlx::FromRow)]
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
pub struct JudgeResponse {
    pub submit_id: i64,
    pub status: Status,
    pub score: i64,
    pub execution_time: i32,
    pub execution_memory: i32,
    pub testcase_result_map: HashMap<i64, TestcaseResult>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Status::AC => "AC",
            Status::TLE => "TLE",
            Status::MLE => "MLE",
            Status::OLE => "OLE",
            Status::WA => "WA",
            Status::RE => "RE",
            Status::CE => "CE",
            Status::IE => "IE",
        };

        write!(f, "{}", s)
    }
}

impl Status {
    pub fn to_priority(&self) -> i32 {
        match *self {
            Status::AC => 1,
            Status::TLE => 2,
            Status::MLE => 3,
            Status::OLE => 4,
            Status::WA => 5,
            Status::RE => 6,
            Status::CE => 7,
            Status::IE => 8,
        }
    }
}
