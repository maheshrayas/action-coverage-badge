use std::env;

#[macro_use]
extern crate log;

use anyhow::Result;
use env_logger::Env;
mod gh;
mod git;
mod lang;
use lang::read_cov_report;
use git::Git;

const BRANCH: &str = "coverage";

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
    // set environment variable for gh
    env::set_var("GITHUB_TOKEN", &token);
    let git = Git::new(BRANCH,&user,&token,&email );
    git.git_branch()?;
    info!("Currently on branch {}",BRANCH);
    let percent = read_cov_report()?;
    let badge_color = git::get_color(percent);
    // update the readme
    Git::<'_>::update_readme(percent, badge_color, "README.md")?;
    // commit the code
    git.commit_push()?;
    // creat pr
    gh::create_pr()?;
    // merge the pr
    gh::merge_pr()?;
    Ok(())
}
