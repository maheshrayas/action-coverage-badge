use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
struct GoReport {
    #[serde(rename = "Packages")]
    packages: Vec<Packages>,
}

#[derive(Deserialize, Debug)]
struct Packages {
    #[serde(rename = "Functions")]
    functions: Vec<Functions>,
}
#[derive(Deserialize, Debug)]
struct Functions {
    #[serde(rename = "Statements")]
    statements: Vec<Statements>,
}
#[derive(Deserialize, Debug)]
struct Statements {
    #[serde(rename = "Start")]
    start: i16,
    #[serde(rename = "End")]
    end: i16,
    #[serde(rename = "Reached")]
    reached: i16,
}
pub fn read_cov_report() -> Result<f64> {
    let contents = fs::read_to_string("cover.json").expect("Something went wrong reading the file");

    let go_report: GoReport = serde_json::from_str(contents.as_str())
        .with_context(|| format!("failed to read the cover.json"))?;
    let mut no_of_statments = 0.0;
    let mut no_of_covered = 0.0;
    for p in go_report.packages {
        for f in p.functions {
            for s in f.statements {
                if s.reached == 1 {
                    no_of_covered += 1.0
                }
                no_of_statments += 1.0
            }
        }
    }
    let cov = (no_of_covered / no_of_statments) * 100.0;
    info!("% covered {}", cov);
    Ok(cov)
}
