//! Welder Recipe Functions
//!
//! ## Overview
//!
//! Welder recipes are stored as TOML formatted files in a git repository.
//! This module provides functions for listing, reading, and writing them.
//!

// Copyright (C) 2016-2017 Red Hat, Inc.
//
// This file is part of bdcs-api-server.
//
// bdcs-api-server is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// bdcs-api-server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with bdcs-api-server.  If not, see <http://www.gnu.org/licenses/>.

use std::clone::Clone;
use std::fs::{File, create_dir_all};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str;
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, NaiveDateTime, FixedOffset};
use git2::{self, BranchType, Commit, DiffFormat, DiffOptions, Oid, ObjectType};
use git2::{Repository, Signature, Time};
use glob::{self, glob};
use toml;


/// Recipe git repo, used with Rocket's managed state system
pub struct RecipeRepo(Mutex<Repository>);
impl RecipeRepo {
    pub fn new(repo_path: &str) -> RecipeRepo {
        // Open an existing repo or create a new one
        let repo = init_repo(repo_path).unwrap();
        RecipeRepo(Mutex::new(repo))
    }

    pub fn repo(&self) -> MutexGuard<Repository> {
        self.0.lock().unwrap()
    }
}


/// Recipe Errors
#[derive(Debug)]
pub enum RecipeError {
    Io(io::Error),
    Git2(git2::Error),
    Glob(glob::PatternError),
    Utf8(str::Utf8Error),
    TomlSer(toml::ser::Error),
    TomlDe(toml::de::Error),
    RecipeName,
    Branch,
    ParseTOML
}

impl From<io::Error> for RecipeError {
    fn from(err: io::Error) -> RecipeError {
        RecipeError::Io(err)
    }
}

impl From<git2::Error> for RecipeError {
    fn from(err: git2::Error) -> RecipeError {
        RecipeError::Git2(err)
    }
}

impl From<glob::PatternError> for RecipeError {
    fn from(err: glob::PatternError) -> RecipeError {
        RecipeError::Glob(err)
    }
}

impl From<str::Utf8Error> for RecipeError {
    fn from(err: str::Utf8Error) -> RecipeError {
        RecipeError::Utf8(err)
    }
}

impl From<toml::ser::Error> for RecipeError {
    fn from(err: toml::ser::Error) -> RecipeError {
        RecipeError::TomlSer(err)
    }
}

impl From<toml::de::Error> for RecipeError {
    fn from(err: toml::de::Error) -> RecipeError {
        RecipeError::TomlDe(err)
    }
}


/// Welder Recipe
///
/// This is used to parse the full recipe's TOML, and to write a JSON representation of
/// the Recipe.
///
/// Empty modules or packages are represented as an empty list.
///
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Recipe {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub modules: Vec<Modules>,
    #[serde(default)]
    pub packages: Vec<Packages>
}

impl Recipe {
    fn filename(&self) -> Result<String, RecipeError> {
        recipe_filename(&self.name)
    }
}

/// Recipe Modules
///
/// This is used for the Recipe's `modules` section.
///
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Modules {
    pub name: String,
    pub version: Option<String>
}

/// Recipe Packages
///
/// This is used for the Recipe's `packages` section
///
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Packages {
    pub name: String,
    pub version: Option<String>
}


// From 24 days of Rust
/// Find master branch's HEAD and return it
///
/// # Arguments
///
/// * `repo` - An open repository
///
/// # Returns
///
/// * master branch's HEAD Commit
///
///
fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
            obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}


/// Return a filename for the recipe
///
/// # Arguments
///
/// * `name` - A recipe name string
///
/// # Returns
///
/// * A String or a RecipeError
///
/// This appends '.toml' to the recipe name after replacing spaces with '-'
///
fn recipe_filename(name: &str) -> Result<String, RecipeError> {
    if name.len() > 0 {
        Ok(format!("{}.toml", name.clone().replace(" ", "-")))
    } else {
        Err(RecipeError::RecipeName)
    }
}


/// Convert git2::Time to RFC3339 time string
fn time_rfc2822(time: Time) -> String {
    let offset = FixedOffset::east(time.offset_minutes() * 60);
    let dt = DateTime::<FixedOffset>::from_utc(NaiveDateTime::from_timestamp(time.seconds(), 0), offset);
    dt.to_rfc2822()
}


/// Initialize the Recipe's git repo
///
/// # Arguments
///
/// * `path` - path to recipe directory
///
/// # Return
///
/// * A Result with a Repository or a RecipeError
///
/// A bare git repo will be created in ./git/ at the specified path.
/// If a repo already exists it will be opened and returned instead of
/// creating a new one.
///
pub fn init_repo(path: &str) -> Result<Repository, RecipeError> {
    let repo_path = Path::new(path).join("git");

    if repo_path.exists() {
        Ok(try!(Repository::open(&repo_path)))
    } else {
        try!(create_dir_all(&repo_path));
        let repo = try!(Repository::init_bare(&repo_path));

        {
            // Make an initial empty commit
            let sig = try!(Signature::now("bdcs-api-server", "user-email"));
            let tree_id = {
                let mut index = try!(repo.index());
                try!(index.write_tree())
            };
            let tree = try!(repo.find_tree(tree_id));
            try!(repo.commit(Some("HEAD"), &sig, &sig, "Initial Recipe repository commit", &tree, &[]));
        }

        Ok(repo)
    }
}

/// Add directory to a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `path` - Path to the directory to add
/// * `branch` - Name of the branch to add the directory contents to
/// * `replace` - Set to true to replace an existing recipe
///
/// # Return
///
/// * Result with () or a RecipeError
///
/// This will add all the files in the `path` directory, without recursing
/// into any sub-directories. The files will be added as individual commits,
/// using [add_file](fn.add_file.html)
///
/// If `replace` is false it will skip recipes that already exist in the repository.
///
pub fn add_dir(repo: &Repository, path: &str, branch: &str, replace: bool) -> Result<(), RecipeError> {
    let toml_glob = format!("{}/*.toml", path);
    for recipe_file in glob(&toml_glob).unwrap().filter_map(Result::ok) {
        if let Some(file) = recipe_file.to_str() {
            match add_file(repo, file, branch, replace) {
                Ok(true) => debug!("Added {} to branch {}", file, branch),
                Ok(false) => debug!("Skipping {}, already in branch {}", file, branch),
                Err(e) => error!("add_dir->add_file failed"; "file" => file, "error" => format!("{:?}", e))
            }
        }
    }
    Ok(())
}

/// Add a file to a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `file` - Path to the file to add
/// * `branch` - Name of the branch to add the file to
/// * `replace` - Set to true to replace an existing recipe
///
/// # Return
///
/// * Result with `true` or a RecipeError
///
/// Files are read into a [Recipe](struct.Recipe.html) struct before being written to disk.
/// The filename committed to git is the name inside the recipe after replacing spaces with '-'
/// and appending .toml to it. It is not the filename it is read from.
///
/// If `replace` is false it will skip recipes that already exist in the repository.
///
pub fn add_file(repo: &Repository, file: &str, branch: &str, replace: bool) -> Result<bool, RecipeError> {
    let mut input = String::new();
    let _ = try!(File::open(file)).read_to_string(&mut input);
    let recipe = try!(toml::from_str::<Recipe>(&input).or(Err(RecipeError::ParseTOML)));

    // Skip existing recipes (using the same recipe.name)
    if replace == false {
        match read(repo, &recipe.name, branch, None) {
            Ok(_) => return Ok(false),
            Err(_) => {}
        }
    }

    write(repo, &recipe, branch)
}

/// Write a recipe to a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name to write to (just the name, no path)
/// * `contents` - An array of bytes to write
/// * `branch` - Name of the branch to add to
///
/// # Return
///
/// * Result with `true` or a RecipeError
///
/// This is used to create a new file, or to write new contents to an existing file.
/// If the branch does not exist, it will be created. By convention the `master`
/// branch is used for example recipes.
///
pub fn write(repo: &Repository, recipe: &Recipe, branch: &str) -> Result<bool, RecipeError> {
    // Does the branch exist? If not, create it based on master
    match repo.find_branch(branch, BranchType::Local) {
        Ok(_) => {}
        Err(_) => {
            let parent_commit = try!(find_last_commit(&repo));
            try!(repo.branch(branch, &parent_commit, false));
        }
    }
    if let Some(branch_id) = try!(repo.find_branch(branch, BranchType::Local))
                                      .get()
                                      .target() {
        debug!("Branch {}'s id is {}", branch, branch_id);
        let parent_commit = try!(repo.find_commit(branch_id));
        let blob_id = {
            // NOTE toml::to_string() can fail depending on which struct elements are empty
            // we use try_from to work around this by converting to a Value first.
            let recipe_toml = try!(toml::Value::try_from(recipe));
            try!(repo.blob(recipe_toml.to_string().as_bytes()))
        };
        let tree_id = {
            let mut tree = repo.treebuilder(Some(&parent_commit.tree().unwrap())).unwrap();
            try!(tree.insert(try!(recipe.filename()), blob_id, 0o100644));
            tree.write().unwrap()
        };
        let tree = try!(repo.find_tree(tree_id));
        let sig = try!(Signature::now("bdcs-api-server", "user-email"));
        let commit_msg = format!("Recipe {} saved", recipe.name);
        let branch_ref = format!("refs/heads/{}", branch);
        try!(repo.commit(Some(&branch_ref), &sig, &sig, &commit_msg, &tree, &[&parent_commit]));
        debug!("Recipe commit:"; "branch" => branch, "recipe_name" => recipe.name, "commit_msg" => commit_msg);
    }

    Ok(true)
}

/// Read a recipe from a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name to read
/// * `commit` - Commit to read from, or None for HEAD
///
/// # Return
///
/// * A Result with a Recipe, or a RecipeError
///
/// The recipe name is converted to a filename by appending '.toml' and replacing
/// all spaces with '-'
///
pub fn read(repo: &Repository, name: &str, branch: &str, commit: Option<&str>) -> Result<Recipe, RecipeError> {
    // Read a filename from a branch.
    let spec = {
        match commit {
            Some(commit) => format!("{}:{}", commit, try!(recipe_filename(name))),
            None => format!("{}:{}", branch, try!(recipe_filename(name)))
        }
    };
    let object = try!(repo.revparse_single(&spec[..]));
    let blob = try!(repo.find_blob(object.id()));
    let blob_str = try!(str::from_utf8(blob.content()));
    Ok(try!(toml::from_str::<Recipe>(blob_str).or(Err(RecipeError::ParseTOML))))
}

/// List the recipes in a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `branch` - Name of the branch to list
/// * `commit` - Commit to read from, or None for HEAD
///
/// # Return
///
/// * A Result with a Vector of Strings or a RecipeError
///
pub fn list(repo: &Repository, branch: &str, commit: Option<&str>) -> Result<Vec<String>, RecipeError> {
    let mut recipes = Vec::new();

    // TODO use commit instead of branch head if it isn't None
    if let Some(branch_id) = try!(repo.find_branch(branch, BranchType::Local))
                                      .get()
                                      .target() {

        debug!("branch {}'s id is {}", branch, branch_id);
        let parent_commit = try!(repo.find_commit(branch_id));
        let tree = try!(parent_commit.tree());
        for entry in tree.iter() {
            // filenames end with .toml, strip that off and return the base.
            if let Some(name) = entry.name() {
                let recipe_name = name.rsplitn(2, ".").last().unwrap_or("");
                recipes.push(recipe_name.to_string());
            }
        }
    }

    Ok(recipes)
}

/// Delete a recipe from a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name to delete
/// * `branch` - Name of the branch to add to
///
/// # Return
///
/// * Result with () or a RecipeError
///
/// Branch and filename must exist otherwise a RecipeError will be returned.
///
pub fn delete(repo: &Repository, recipe_name: &str, branch: &str) -> Result<bool, RecipeError> {
    // Does the branch exist? If not, it's an error
    match repo.find_branch(branch, BranchType::Local) {
        Ok(_) => {}
        Err(_) => {
            return Err(RecipeError::Branch);
        }
    }

    let filename = try!(recipe_filename(recipe_name));
    if let Some(branch_id) = try!(repo.find_branch(branch, BranchType::Local))
                                      .get()
                                      .target() {
        debug!("Branch {}'s id is {}", branch, branch_id);
        let parent_commit = try!(repo.find_commit(branch_id));
        let tree_id = {
            let mut tree = repo.treebuilder(Some(&parent_commit.tree().unwrap())).unwrap();
            try!(tree.remove(&filename));
            tree.write().unwrap()
        };
        let tree = try!(repo.find_tree(tree_id));
        let sig = try!(Signature::now("bdcs-api-server", "user-email"));
        let commit_msg = format!("Recipe {} deleted", filename);
        let branch_ref = format!("refs/heads/{}", branch);
        try!(repo.commit(Some(&branch_ref), &sig, &sig, &commit_msg, &tree, &[&parent_commit]));
        debug!("Recipe delete commit:"; "branch" => branch, "recipe_name" => recipe_name,
               "filename" => filename, "commit_msg" => commit_msg);
    }

    Ok(true)
}


/// Revert a recipe to a previous commit
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name to revert
/// * `commit` - Commit hash to revert to
/// * `branch` - Name of the branch
///
/// # Return
///
/// * Result with true or a RecipeError
///
/// Branch and filename must exist otherwise a RecipeError will be returned.
///
pub fn revert(repo: &Repository, recipe_name: &str, branch: &str, commit: &str) -> Result<bool, RecipeError> {
    // Does the branch exist? If not, it's an error
    match repo.find_branch(branch, BranchType::Local) {
        Ok(_) => {}
        Err(_) => {
            return Err(RecipeError::Branch);
        }
    }

    let filename = try!(recipe_filename(recipe_name));
    if let Some(branch_id) = try!(repo.find_branch(branch, BranchType::Local))
                                      .get()
                                      .target() {
        // Find the commit to revert to
        let revert_tree = try!(try!(repo.find_commit(try!(Oid::from_str(commit)))).tree());
        let revert_entry = revert_tree.get_name(&filename);
        match revert_entry {
            Some(entry) => {
                let revert_id = entry.id();
                debug!("revert"; "filename" => filename, "id" => format!("{}", revert_id));

                debug!("Branch {}'s id is {}", branch, branch_id);
                let parent_commit = try!(repo.find_commit(branch_id));
                let tree_id = {
                    let mut tree = repo.treebuilder(Some(&parent_commit.tree().unwrap())).unwrap();
                    try!(tree.insert(&filename, revert_id, 0o100644));
                    tree.write().unwrap()
                };
                let tree = try!(repo.find_tree(tree_id));
                let sig = try!(Signature::now("bdcs-api-server", "user-email"));
                let commit_msg = format!("Recipe {} reverted to commit {}", filename, commit);
                let branch_ref = format!("refs/heads/{}", branch);
                try!(repo.commit(Some(&branch_ref), &sig, &sig, &commit_msg, &tree, &[&parent_commit]));
                debug!("Recipe revert commit:"; "branch" => branch, "recipe_name" => recipe_name,
                       "filename" => filename, "commit" => commit, "commit_msg" => commit_msg);
            }
            None => return Ok(false)
        }
    }

    Ok(true)
}


/// Recipe Commit
///
/// Details about changes to a recipe
///
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecipeCommit {
    pub commit: String,
    pub time: String,
    pub summary: String
}

/// List the commits for a recipe in a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name
/// * `branch` - Name of the branch to list
///
/// # Return
///
/// * A Vector of RecipeCommit or a RecipeError
///
/// If the name is None all changes for the branch will be returned.
///
pub fn commits(repo: &Repository, name: &str, branch: &str) -> Result<Vec<RecipeCommit>, RecipeError> {
    // Does the branch exist? If not, it's an error
    match repo.find_branch(branch, BranchType::Local) {
        Ok(_) => {}
        Err(_) => {
            return Err(RecipeError::Branch);
        }
    }

    let mut revwalk = try!(repo.revwalk());
    revwalk.set_sorting(git2::SORT_TIME);
    try!(revwalk.push_ref(&format!("refs/heads/{}", branch)));

    let filename = try!(recipe_filename(&name));
    let mut diffopts = DiffOptions::new();
    diffopts.pathspec(&filename);

    let mut commits = Vec::new();
    for id in revwalk {
        let mut commit = try!(repo.find_commit(try!(id)));
        let tree = try!(commit.tree());
        let tree_entry = tree.get_name(&filename);
        match tree_entry {
            Some(_) => {
                // Check to see if the file changed between the parents and this commit
                let m = commit.parents().all(|parent| {
                    match_with_parent(repo, &commit, &parent, &mut diffopts)
                    .unwrap_or(false)
                });
                if m {
                    commits.push(RecipeCommit {
                                    commit: format!("{}", commit.id()),
                                    time: time_rfc2822(commit.time()),
                                    summary: format!("{}", commit.summary().unwrap_or("Missing"))
                    });
                }
            }
            None => {}
        }
    }

    Ok(commits)
}


// From git-rs log example
/// Check for changes between commit and the commit's parent
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `commit` - A commit
/// * `parent` - A parent of the commit
/// * `opts` - Diff options (usually pathspec to limit it to a file)
///
/// # Return
///
/// Returns true if there were changes, false if not.
///
fn match_with_parent(repo: &Repository, commit: &Commit, parent: &Commit,
                     opts: &mut DiffOptions) -> Result<bool, RecipeError> {
    let a = try!(parent.tree());
    let b = try!(commit.tree());
    let diff = try!(repo.diff_tree_to_tree(Some(&a), Some(&b), Some(opts)));
    Ok(diff.deltas().len() > 0)
}


/// Diff two commits for a recipe
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name
/// * `branch` - Name of the branch
/// * `old_commit` - Older commit to use
/// * `new_commit` - New commit to use
///
/// # Return
///
/// * A Vec of String (diff lines) or a RecipeError
///
/// If new_commit is None HEAD will be used.
///
pub fn diff(repo: &Repository,
                      name: &str,
                      branch: &str,
                      old_commit: &str,
                      new_commit: Option<&str>) -> Result<Vec<String>, RecipeError> {
    // Does the branch exist? If not, it's an error
    match repo.find_branch(branch, BranchType::Local) {
        Ok(_) => {}
        Err(_) => {
            return Err(RecipeError::Branch);
        }
    }

    // Show the diff between 2 revisions of a file using the commit ids
    let old_tree = try!(try!(repo.find_commit(try!(Oid::from_str(old_commit)))).tree());
    let new_tree = match new_commit {
        Some(id) => {
            try!(try!(repo.find_commit(try!(Oid::from_str(id)))).tree())
        },
        None => {
            // No new_commit, use the HEAD of the branch as the one to compare with
            let branch_id = try!(repo.find_branch(branch, BranchType::Local)).get().target().unwrap();
            try!(try!(repo.find_commit(branch_id)).tree())
        }
    };

    let filename = try!(recipe_filename(&name));
    let mut opts = DiffOptions::new();
    opts.patience(true)
        .minimal(true)
        .pathspec(filename);
    let diff = try!(repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut opts)));

    let mut diff_lines = Vec::new();
    try!(diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        match line.origin() {
            '+' | '-' | ' ' => diff_lines.push(format!("{}{}", line.origin(), str::from_utf8(line.content()).unwrap().trim_right())),
            _ => {
                for l in str::from_utf8(line.content()).unwrap().lines() {
                    diff_lines.push(format!("{}", l));
                }
            }
        }
        true
    }));

    Ok(diff_lines)
}
