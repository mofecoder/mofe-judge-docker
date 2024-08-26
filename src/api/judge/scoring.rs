use crate::{
    db::DbPool,
    models::{JudgeRequest, JudgeResponse, Status, TestcaseSets, TestcaseTestcaseSets},
};
use anyhow::Result;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use crate::models::{AggregateType, TestcaseResult};
use crate::models::Status::*;

fn aggregate_testcase_set(
    testcase_set: &TestcaseSets,
    testcase_ids: &Vec<i64>,
    testcase_results: &HashMap<i64, TestcaseResult>
) -> i64 {
    let agg = testcase_set.aggregate_type;
    let mut total = agg.id();
    if agg == AggregateType::None {
        let mut is_ac = true;
        for testcase_id in testcase_ids {
            if testcase_results[testcase_id].result.status != AC {
                is_ac = false;
                break;
            }
        }
        if is_ac {
            total += testcase_set.points;
        }
        return total
    }
    for testcase_id in testcase_ids {
        let result = testcase_results[testcase_id].result;
        let testcase_score = result.score;

        if result.status == TLE || result.status == RE ||
            result.status == MLE || result.status == OLE {
            total = agg.update(total, 0);
        } else if result.score.is_some() {
            total = agg.update(total, testcase_score.unwrap())
        }
    }
    if total == agg.id() { 0 } else { total }
}

pub async fn scoring(
    conn: Arc<DbPool>,
    req: &JudgeRequest,
    submit_result: &JudgeResponse,
) -> Result<i64> {
    if IE == submit_result.status || CE == submit_result.status {
        return Ok(0);
    }

    let conn = Arc::as_ref(&conn);
    let testcase_sets: Vec<TestcaseSets> = sqlx::query_as(
        r#"
    SELECT id, points, aggregate_type FROM testcase_sets
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
        score += aggregate_testcase_set(
            testcase_set,
            testcase_set_map.get(&testcase_set.id).unwrap_or(&vec![]),
            &submit_result.testcase_result_map,
        );
    }

    Ok(score)
}
