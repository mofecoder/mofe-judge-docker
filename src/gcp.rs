use anyhow::Result;
use cloud_storage::Object;
use std::{fs::File, io::Write};

const SUBMIT_SOURCE_BUCKET: &str = "cafecoder-submit-source";
const CHECKER_BUCKET: &str = "cafecoder-submit-source/checker";
const TESTCASE_BUCKET: &str = "cafecoder-testcase";

pub async fn download_submit_source(source_name: &str, path: &str) -> Result<()> {
    let bytes = Object::download(SUBMIT_SOURCE_BUCKET, source_name).await?;

    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}

pub async fn download_testcase(
    problem_uuid: &str,
    testcase_name: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let _ = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")?;

    let input_bytes = Object::download(
        TESTCASE_BUCKET,
        &format!("{}/input/{}", problem_uuid, testcase_name),
    )
    .await?;
    let output_bytes = Object::download(
        TESTCASE_BUCKET,
        &format!("{}/output/{}", problem_uuid, testcase_name),
    )
    .await?;

    Ok((input_bytes, output_bytes))
}

pub async fn download_checker(checker_name: &str, path: &str) -> Result<()> {
    let bytes = Object::download(
        &format!("{}/{}", CHECKER_BUCKET, checker_name), 
        "checker.cpp",
    ).await?;

    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}