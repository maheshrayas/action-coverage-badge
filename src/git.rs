use anyhow::{Context, Result};
use regex::Regex;
use std::fs;

use git2::{
    Commit, Cred, Direction, IndexAddOption, ObjectType, ProxyOptions, PushOptions,
    RemoteCallbacks, Repository, Signature,
};

const RED: &str = "red";
const GREEN: &str = "green";
const YELLOW: &str = "yellow";
const INACTIVE: &str = "inactive";

pub(crate) struct Git<'a> {
    pub(crate) branch: &'a str,
    pub(crate) http_user: &'a str,
    pub(crate) http_pass: &'a str,
    pub(crate) git_email: &'a str,
    pub(crate) proxy: &'a str,
    pub(crate) pr_sha: &'a str,
}

pub(crate) fn get_color<'a>(percentage: f64) -> &'a str {
    if (80.0..=100.0).contains(&percentage) {
        GREEN
    } else if (40.0..80.0).contains(&percentage) {
        YELLOW
    } else if percentage < 40.0 {
        RED
    } else {
        INACTIVE
    }
}

impl<'a> Git<'a> {
    pub(crate) fn new(
        branch: &'a str,
        http_user: &'a str,
        http_pass: &'a str,
        git_email: &'a str,
        proxy: &'a str,
        pr_sha:&'a str,
    ) -> Self {
        Git {
            branch,
            http_user,
            http_pass,
            git_email,
            proxy,
            pr_sha,
        }
    }

    pub(crate) fn git_branch(&self) -> Result<()> {
        let repo = Repository::open(".")
            .with_context(|| "Something went wrong while setting up repository".to_string())?;
        let head = repo.head()?;
        let oid = head.target().unwrap();
        let commit = repo.find_commit(oid)?;
        let _ = repo
            .branch(self.branch, &commit, false)
            .with_context(|| format!("could not create new branch `{}`", self.branch))?;
        let obj = repo
            .revparse_single(&("refs/heads/".to_owned() + self.branch))
            .with_context(|| format!("could not create new branch `{}`", self.branch))?;
        let _ = repo
            .checkout_tree(&obj, None)
            .with_context(|| format!("Failed while checkout tree -> {}", self.branch))?;
        repo.set_head(&("refs/heads/".to_owned() + self.branch))
            .with_context(|| format!("Failed to checkout branch {}", self.branch))?;
        Ok(())
    }

    pub fn update_readme(percent: f64, color: &str, file: &str) -> Result<()> {
        let cov = format!("{:.1$}", percent, 2);
        let md_file = fs::read_to_string(file)?;
        let re =
            Regex::new(r"https://img.shields.io/badge/coverage-([\d.\d]+)%25-([a-z]+)").unwrap();
        let replace = re
            .captures(&md_file)
            .with_context(|| "no valid coverage url found in img shields".to_string())?;
        let replaced = md_file.replace(
            &replace[0],
            format!("https://img.shields.io/badge/coverage-{}%25-{}", cov, color).as_str(),
        );
        fs::write(file, replaced).with_context(|| "failed to update the coverage".to_string())?;
        Ok(())
    }

    pub(crate) fn commit_push(&self) -> Result<()> {
        let repo = Repository::open(".")
            .with_context(|| "Something went wrong while setting up repository".to_string())?;
        let mut index = repo.index()?;
        index.add_all(["README.md"].iter(), IndexAddOption::FORCE, None)?;
        index.write()?;
        let oid = index.write_tree()?;
        let signature = Signature::now(self.http_user, self.git_email)?;
        let parent_commit = Self::find_last_commit(&repo)?;
        println!("parent commit: {:?}",parent_commit);
        let tree = repo.find_tree(oid)?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "AUTOCOMMIT: Updating coverage",
            &tree,
            &[&parent_commit],
        )?;
        let mut remote = repo.find_remote("origin")?;
        let mut p = ProxyOptions::new();
        if self.proxy != "" {
            p.url(&self.proxy);
        }

        remote.connect_auth(Direction::Push, Some(self.create_callbacks()), Some(p))?;
        let mut push_options = PushOptions::default();
        push_options.remote_callbacks(self.create_callbacks());

        let ref_specs = format!("refs/heads/{}", self.branch);

        remote.push(
            &[[
                [ref_specs.to_owned(), ref_specs].join(":"),
            ]
            .join("")],
            Some(&mut push_options),
        )?;

        Ok(())
    }

    fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        obj.into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }

    fn create_callbacks(&self) -> RemoteCallbacks {
        let mut callbacks = RemoteCallbacks::new();
        callbacks
            .credentials(move |_, _, _| Cred::userpass_plaintext(self.http_user, self.http_pass));
        callbacks
    }
}
