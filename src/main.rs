use std::env;

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use env_logger::Env;

use self::lang::read_cov_report;
mod git;
mod lang;

fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);
    // check for the lang
    //let langu = env::var("INPUT_LANGUAGE").expect("Missing input parameter: repo");
    let token = env::var("INPUT_TOKEN").expect("Missing input parameter: Token");
    let user = env::var("INPUT_USER").expect("Missing input parameter: USER");
    let email = env::var("INPUT_EMAIL").expect("Missing input parameter: EMAIL");
    git::git_branch("test")?;
    info!("Currently on branch test");
    let percent = read_cov_report()?;
    let badge_color = git::get_color(percent);
    let z = git::update_readme(percent,badge_color)?;
    // commit the code
    git::commit_push("test",&user,&token,&email)?;
    // creat pr
    // merge the pr
    Ok(())
}
