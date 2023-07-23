use anyhow::{Error, Result};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::{client::Client, http::objects::get::GetObjectRequest};
use std::default::Default;
use std::vec::Vec;
use std::{fs::File, io::Write};

const SUBMIT_SOURCE_BUCKET: &str = "cafecoder-submit-source";
const CHECKER_BUCKET: &str = "cafecoder-submit-source";
const TESTCASE_BUCKET: &str = "cafecoder-testcase";

// static client = Client

pub struct GcpClient {
    client: Client,
    source_bucket_name: String,
    checker_bucket_name: String,
    testcase_bucket_name: String,
}

impl GcpClient {
    pub async fn new(client: Client) -> Result<Self, Error> {
        Ok(Self {
            client,
            source_bucket_name: SUBMIT_SOURCE_BUCKET.into(),
            testcase_bucket_name: TESTCASE_BUCKET.into(),
            checker_bucket_name: CHECKER_BUCKET.into(),
        })
    }

    pub async fn download_submit_source(&self, source_name: &str, path: &str) -> Result<()> {
        let bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.source_bucket_name.clone(),
                    object: source_name.into(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;

        let path = crate::JUDGE_DIR.join(path);
        let mut file = File::create(&path)?;

        file.write_all(&bytes)?;

        Ok(())
    }

    pub async fn download_testcase(
        &self,
        problem_uuid: &str,
        testcase_name: &str,
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        let _ = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")?;

        let input_bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.testcase_bucket_name.clone(),
                    object: format!("{}/input/{}", problem_uuid, testcase_name).into(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;
        let output_bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.testcase_bucket_name.clone(),
                    object: format!("{}/output/{}", problem_uuid, testcase_name).into(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;
        Ok((input_bytes, output_bytes))
    }

    pub async fn download_checker(&self, checker_name: &str, path: &str) -> Result<()> {
        let bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.checker_bucket_name.clone(),
                    object: checker_name.into(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;

        let path = crate::JUDGE_DIR.join(path);
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;

        Ok(())
    }
}
