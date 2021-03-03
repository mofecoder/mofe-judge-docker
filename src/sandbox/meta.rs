use std::str::FromStr;

#[derive(Default)]
pub struct Meta {
    pub cg_mem: Option<u64>,
    pub cg_oom_killed: Option<u64>,
    pub csw_forced: Option<u64>,
    pub csw_voluntary: Option<u64>,
    pub exitcode: Option<u64>,
    pub exitsig: Option<u64>,
    pub killed: Option<u64>,
    pub max_rss: Option<u64>,
    pub message: Option<String>,
    pub status: Option<String>,
    pub time: Option<f64>,
    pub time_wall: Option<f64>,
}

impl FromStr for Meta {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut meta: Meta = Default::default();

        for line in value.lines() {
            if let Some(pos) = line.find(":") {
                let line_key = &line[0..pos];
                let line_value = &line[pos + 1..];

                macro_rules! push_meta {
                    ($key:tt, $type:tt) => {
                        if line_key == std::stringify!($key) {
                            let parsed_value: $type =
                                line_value.parse().map_err(anyhow::Error::from)?;
                            meta.$key = Some(parsed_value);
                        }
                    };
                }

                push_meta!(cg_mem, u64);
                push_meta!(cg_oom_killed, u64);
                push_meta!(csw_forced, u64);
                push_meta!(csw_voluntary, u64);
                push_meta!(exitcode, u64);
                push_meta!(exitsig, u64);
                push_meta!(killed, u64);
                push_meta!(max_rss, u64);
                push_meta!(message, String);
                push_meta!(status, String);
                push_meta!(time, f64);
                push_meta!(time_wall, f64);
            }
        }

        Ok(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_meta() {
        let meta: Meta = "cg_mem:100\ntime:0.01\nstatus:TO\n".parse().unwrap();
        assert!(meta.cg_mem == Some(100));
        assert!(meta.time == Some(0.01));
        assert!(meta.status == Some("TO".to_string()));
        assert!(meta.csw_forced.is_none());
    }
}
