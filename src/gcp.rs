use anyhow::Result;
use cloud_storage::Object;
use std::{fs::File, io::Write};

const SUBMIT_SOURCE_BUCKET: &str = "cafecoder-submit-source";
const TESTCASE_BUCKET: &str = "cafecoder-testcase";

pub async fn download_submit_source(path: &str, name: &str) -> Result<()> {
    let bytes = Object::download(SUBMIT_SOURCE_BUCKET, path).await?;

    let mut file = File::create(name)?;
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
