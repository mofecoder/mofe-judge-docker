use std::{collections::HashMap, fmt, fmt::Debug};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug)]
pub struct TestcaseResult {
    pub result: JudgeResult,
    pub cmd_result: CmdResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CmdResult {
    pub execution_time: i32,   // ms
    pub stdout_size: usize,    // byte
    pub execution_memory: i32, // KB
    pub ok: bool,              // exit_code == 0
    pub message: String,       // コンパイルメッセージ
    pub exit_code: i32,        // exit_code
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Problem {
    pub problem_id: i64,
    pub uuid: String,
    pub checker_path: String,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Testcase {
    pub testcase_id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, Copy, Clone, PartialEq, sqlx::Type)]
#[repr(i32)]
pub enum AggregateType {
    None,
    Sum,
    Max,
    Min
}

impl From<i32> for AggregateType {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Sum,
            2 => Self::Max,
            3 => Self::Min,
            _ => unreachable!()
        }
    }
}

impl AggregateType {
    pub fn id(&self) -> i64 {
        if *self == Self::Min {
            i64::MAX
        } else {
            0
        }
    }

    pub fn update(&self, total: i64, score: i64) -> i64 {
        match *self {
            Self::None => 0,
            Self::Min => total.min(score),
            Self::Max => total.max(score),
            Self::Sum => total + score,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct TestcaseSets {
    pub id: i64,
    pub points: i64,
    pub aggregate_type: AggregateType,
}

#[derive(sqlx::FromRow)]
pub struct TestcaseTestcaseSets {
    pub testcase_id: i64,
    pub testcase_set_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompileRequest {
    pub submit_id: i64,
    pub cmd: String, // コンパイルコマンド or 実行コマンド
}

#[derive(Serialize, Deserialize)]
pub struct CompileResponse(pub CmdResult);

impl CompileResponse {
    pub fn empty() -> Self {
        Self(CmdResult {
            execution_time: 0,
            stdout_size: 0,
            execution_memory: 0,
            ok: true,
            message: "".to_string(),
            exit_code: 0,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadRequest {
    pub submit_id: i64,
    pub code_path: String, // gcp 上のパス
    pub filename: String,  // Main.ext
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JudgeRequest {
    pub submit_id: i64,
    pub cmd: String,     // コンパイルコマンド or 実行コマンド
    pub time_limit: i32, // 実行制限時間
    pub mem_limit: i32,  // メモリ制限

    pub testcases: Vec<Testcase>, // pub testcase: Testcase,
    pub problem: Problem,         // pub problem: Problem,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JudgeResponse {
    pub submit_id: i64,
    pub status: Status,
    pub score: i64,
    pub execution_time: i32,
    pub execution_memory: i32,
    pub testcase_result_map: HashMap<i64, TestcaseResult>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct JudgeResult {
    pub status: Status,
    pub score: Option<i64>,
}

impl JudgeResult {
    pub fn from_status(status: Status) -> Self {
        Self {
            status,
            score: None
        }
    }
}

#[allow(clippy::unknown_clippy_lints)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum Status {
    AC,
    TLE,
    MLE,
    OLE,
    WA,
    RE,
    CE,
    IE,
    CP,
    QLE,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Status::AC => "AC",
            Status::TLE => "TLE",
            Status::MLE => "MLE",
            Status::OLE => "OLE",
            Status::QLE => "QLE",
            Status::WA => "WA",
            Status::RE => "RE",
            Status::CE => "CE",
            Status::IE => "IE",
            Status::CP => "CP",
        };

        write!(f, "{}", s)
    }
}

impl Status {
    #[allow(dead_code)]
    pub fn to_priority(&self) -> i32 {
        match *self {
            Status::CP => 0,
            Status::AC => 1,
            Status::TLE => 2,
            Status::MLE => 3,
            Status::OLE => 4,
            Status::QLE => 5,
            Status::WA => 6,
            Status::RE => 7,
            Status::CE => 8,
            Status::IE => 9,
        }
    }
}
