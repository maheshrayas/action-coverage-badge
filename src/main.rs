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
    let test_on = env::var("INPUT_TEST_ON").expect("Missing input parameter: TEST_ON");
    // set environment variable for gh
    env::set_var("GITHUB_TOKEN", &token);
    let percent = read_cov_report()?;
    let badge_color = git::get_color(percent);

    let proxy = match env::var("HTTP_PROXY") {
        Ok(proxy) => proxy,
        Err(_) => String::from(""),
    };

    let branch = if test_on == "default" {
        "coverage".to_string()
    }else{
        env::var("GITHUB_HEAD_REF").unwrap()
    };

    let git = Git::new(&branch, &user, &token, &email, &proxy,&test_on);

    // update the readme
    Git::<'_>::update_readme(percent, badge_color, "README.md")?;

    if test_on == "pr" {
        // commit the code
        git.commit_push_to_default_branch()?;

    }else {

        git.git_branch()?;
        info!("Currently on branch {}",branch);
        // commit the code
        git.commit_push_to_default_branch()?;
        // creat pr
        gh::create_pr()?;
        // merge the pr
        gh::merge_pr()?;
    }

    Ok(())
}
