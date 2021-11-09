use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::env;

use git2::{
    Cred, Direction, IndexAddOption, ProxyOptions, PushOptions, RemoteCallbacks, Repository,
    Signature,Oid,
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
   pub(crate) test_on: &'a str,
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
   pub(crate) fn new(branch: &'a str, http_user:&'a str, http_pass:&'a str,git_email:&'a str, proxy:&'a str,test_on:&'a str) -> Self {
        Git{
        branch,
        http_user,
        http_pass,
        git_email,
        proxy,
        test_on,
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

    pub(crate) fn commit_push_to_default_branch(&self
    ) -> Result<()> {
        let repo = Repository::open(".")
            .with_context(|| "Something went wrong while setting up repository".to_string())?;
        let mut index = repo.index()?;
        index.add_all(["README.md"].iter(), IndexAddOption::FORCE, None)?;
        index.write()?;

        let oid : Oid;
        if self.test_on == "default"{
             oid = index.write_tree()?;
        }else{
            let git_ref= env::var("GITHUB_HEAD_REF").unwrap();
            oid = Oid::from_str(git_ref.as_str()).unwrap();
        }
        let signature = Signature::now(self.http_user, self.git_email)?;
        let parent_commit = repo.find_commit(oid).unwrap();
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

        remote.connect_auth(
            Direction::Push,
            Some(self.create_callbacks()),
            None,
        )?;
        let mut push_options = PushOptions::default();
        push_options.remote_callbacks(self.create_callbacks());

        let ref_specs = format!("refs/heads/{}", self.branch);

        remote.push(
            &[[
                String::from("+"),
                [ref_specs.to_owned(), ref_specs].join(":"),
            ]
            .join("")],
            Some(&mut push_options),
        )?;
        Ok(())
    }

    // fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    //     let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    //     obj.into_commit()
    //         .map_err(|_| git2::Error::from_str("Couldn't find commit"))
    // }

    // fn commit_push(&self) -> Result<()> {
    //     let repo = Repository::open(".")
    //     .with_context(|| "Something went wrong while cloning".to_string())?;
    //     let mut index = repo.index()?;
    //     let my_oid_str =  env::var("GITHUB_HEAD_REF").unwrap().as_str();
    //     let oid = Oid::from_str(my_oid_str).unwrap();
    //     let commit = repo.find_commit(oid).unwrap();
    
    //     let branch = repo.branch(
    //         my_oid_str,
    //         &commit,
    //         false,
    //     );
    
    //     let obj = repo.revparse_single(&("refs/heads/".to_owned() + my_oid_str)).unwrap(); 
    
    //     repo.checkout_tree(
    //         &obj,
    //         None,
    //     );
    //     repo.set_head(&("refs/heads/".to_owned() + my_oid_str));
    //     let mut remote = repo.find_remote("origin")?;
    //     remote.connect_auth(
    //         Direction::Push,
    //         Some(self.create_callbacks()),
    //         None,
    //     )?;
    //     let mut push_options = PushOptions::default();
    //     push_options.remote_callbacks(self.create_callbacks());

    //     let ref_specs = format!("refs/heads/{}", self.branch);

    //     remote.push(
    //         &[[
    //             String::from("+"),
    //             [ref_specs.to_owned(), ref_specs].join(":"),
    //         ]
    //         .join("")],
    //         Some(&mut push_options),
    //     )?;
    
    // }

    fn create_callbacks(&self) -> RemoteCallbacks {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_, _, _| Cred::userpass_plaintext(self.http_user, self.http_pass));
        callbacks
    }

}
