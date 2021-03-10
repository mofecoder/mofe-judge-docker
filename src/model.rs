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

#[derive(Deserialize, FromRow)]
pub struct Problem {
    pub problem_id: i64,
    pub uuid: String,
}

#[derive(Deserialize, FromRow)]
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
