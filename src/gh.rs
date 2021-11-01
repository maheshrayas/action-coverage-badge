use anyhow::Result;
use std::error::Error;
use std::fmt;
use std::process::Command;

#[derive(Debug)]
struct GHError {
    details: String,
}
impl GHError {
    fn new(msg: String) -> GHError {
        GHError { details: msg }
    }
}

impl fmt::Display for GHError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for GHError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub(crate) fn create_pr() -> Result<()> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("create")
        .arg("-f")
        .output()?;
    if output.status.success() {
        info!("Successfully created pr to default branch")
    } else {
        return Err(GHError::new(format!(
            "Failed to while creating PR {:?}",
            String::from_utf8_lossy(&output.stderr)
        ))
        .into());
    }
    Ok(())
}

pub(crate) fn merge_pr() -> Result<()> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("merge")
        .arg("-m")
        .arg("-d")
        .output()?;
    if output.status.success() {
        info!("Successfully merged to default branch")
    } else {
        return Err(GHError::new(format!(
            "Failed to while merging the PR {:?}",
            String::from_utf8_lossy(&output.stderr)
        ))
        .into());
    }
    Ok(())
}
