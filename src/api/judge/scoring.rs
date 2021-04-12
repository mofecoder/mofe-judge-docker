use crate::{
    db::DbPool,
    models::{JudgeRequest, JudgeResponse, Status, TestcaseSets, TestcaseTestcaseSets},
};
use anyhow::Result;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

pub async fn scoring(
    conn: Arc<DbPool>,
    req: &JudgeRequest,
    submit_result: &JudgeResponse,
) -> Result<i64> {
    if Status::IE == submit_result.status || Status::CE == submit_result.status {
        return Ok(0);
    }

    let conn = Arc::as_ref(&conn);
    let testcase_sets: Vec<TestcaseSets> = sqlx::query_as(
        r#"
    SELECT id, points FROM testcase_sets
    WHERE deleted_at IS NULL AND problem_id = ?
    "#,
    )
    .bind(req.problem.problem_id)
    .fetch_all(conn)
    .await?;

    let testcase_testcase_sets: Vec<TestcaseTestcaseSets> = sqlx::query_as(
        r#"
    SELECT testcase_id, testcase_set_id FROM testcase_testcase_sets
    INNER JOIN testcases ON testcase_testcase_sets.testcase_id = testcases.id
    WHERE problem_id = ? AND testcase_testcase_sets.deleted_at IS NULL AND testcases.deleted_at IS NULL
    "#,
    )
    .bind(req.problem.problem_id)
    .fetch_all(conn)
    .await?;

    let mut testcase_set_map: HashMap<i64, Vec<i64>> = HashMap::new();
    for testcase_testcase_set in &testcase_testcase_sets {
        match testcase_set_map.entry(testcase_testcase_set.testcase_set_id) {
            Entry::Vacant(e) => {
                e.insert(vec![testcase_testcase_set.testcase_id]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(testcase_testcase_set.testcase_id);
            }
        }
    }

    let mut score = 0i64;
    for testcase_set in &testcase_sets {
        let mut is_ac = true;
        for testcase_id in &testcase_set_map[&testcase_set.id] {
            if submit_result.testcase_result_map[testcase_id].status != Status::AC {
                is_ac = false;
                break;
            }
        }
        if is_ac {
            let point: i64 = testcase_set.points as i64;
            score += point;
        }
    }

    Ok(score)
}
