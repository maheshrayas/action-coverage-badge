use anyhow::{Context, Result};
use std::{fs, };
use regex::{Regex};


use git2::{
     Commit, Direction,  IndexAddOption, ObjectType, PushOptions,RemoteCallbacks,Cred,
    Repository, Signature,
};

const RED: &str = "red";
const GREEN: &str = "green";
const YELLOW: &str = "yellow";
const INACTIVE: &str = "inactive";

pub(crate) fn get_color<'a>(percentage : f64) -> &'a str {

    if (percentage>=80.0) && (percentage<=100.0) {
        return GREEN

    } else if (percentage>=40.0) && (percentage<80.0) {
        return YELLOW

    }
    else if percentage<40.0 {
        return RED
    }
    return INACTIVE
}

pub(crate) fn git_branch(branch: &str) -> Result<()> {
    let repo = Repository::open(".")
        .with_context(|| "Something went wrong while setting up repository".to_string())?;
    let head = repo.head()?;
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid)?;
    let _ = repo
        .branch(branch, &commit, false)
        .with_context(|| format!("could not create new branch `{}`", branch))?;
    let obj = repo
        .revparse_single(&("refs/heads/".to_owned() + branch))
        .with_context(|| format!("could not create new branch `{}`", branch))?;
    let _ = repo
        .checkout_tree(&obj, None)
        .with_context(|| format!("Failed while checkout tree -> {}", branch))?;
    repo.set_head(&("refs/heads/".to_owned() + branch))
        .with_context(|| format!("Failed to checkout branch {}", branch))?;
    Ok(())
}


pub fn update_readme(percent:f64,color:&str) -> Result<()>{
    let cov = format!("{:.1$}",percent,2);
    let md_file = fs::read_to_string("README.md")?;
    let re =  Regex::new(r"https://img.shields.io/badge/coverage-([\d.\d]+)%25-([a-z]+)").unwrap();
    let replace =  re.captures(&md_file).with_context(||format!("no valid coverage url found in img shields"))?;
    let replaced =  md_file.replace(&replace[0], format!("https://img.shields.io/badge/coverage-{}%25-{}",cov,color).as_str());
    fs::write("README.md", replaced).with_context(||format!("failed to update the coverage"))?;
    Ok(())
}

pub(crate) fn commit_push(branch:&str, httpsuser: &str,
    httpspass: &str,email:&str) -> Result<()> {
    let repo = Repository::open(".")
        .with_context(|| "Something went wrong while setting up repository".to_string())?;
    let mut index = repo.index()?;
    index.add_all(["README.md"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let oid = index.write_tree()?;
    let signature = Signature::now(httpsuser, email)?;
    let parent_commit = find_last_commit(&repo)?;
    let tree = repo.find_tree(oid)?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "AUTOCOMMIT: Updating coverage",
        &tree,
        &[&parent_commit],
    )?;
    let mut  remote= repo.find_remote("origin")?;

    remote.connect_auth(Direction::Push, Some(create_callbacks(&httpsuser, &httpspass)), None)?;
    let mut push_options = PushOptions::default();
    push_options.remote_callbacks(create_callbacks(&httpsuser, &httpspass));

    let ref_specs = format!("refs/heads/{}", branch);
    remote.push(&[[ref_specs.to_owned(), ref_specs].join(":")], Some(&mut push_options))?;
    Ok(())
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}


fn create_callbacks<'a>(httpsuser:&'a str, pass:&'a str) -> RemoteCallbacks<'a>{
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_, _, _| {
        Cred::userpass_plaintext(httpsuser,pass)
    });
    callbacks
}