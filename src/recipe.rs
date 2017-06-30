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
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str;
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, NaiveDateTime, FixedOffset};
use git2::{self, BranchType, Commit, DiffFormat, DiffOptions, Oid, ObjectType};
use git2::{Repository, Signature, Time};
use glob::{self, glob};
use regex::{self, Regex};
use semver;
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
    SemVerError(semver::SemVerError),
    RegexError(regex::Error),
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

impl From<semver::SemVerError> for RecipeError {
    fn from(err: semver::SemVerError) -> RecipeError {
        RecipeError::SemVerError(err)
    }
}

impl From<regex::Error> for RecipeError {
    fn from(err: regex::Error) -> RecipeError {
        RecipeError::RegexError(err)
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
    pub version: String,
    #[serde(default)]
    pub modules: Vec<Modules>,
    #[serde(default)]
    pub packages: Vec<Packages>
}

impl Recipe {
    /// Convert the recipe name to a filename
    pub fn filename(&self) -> Result<String, RecipeError> {
        recipe_filename(&self.name)
    }

    /// Convert the version string to a SemVer Version
    pub fn version(&self) -> Result<semver::Version, RecipeError> {
        Ok(try!(semver::Version::parse(&self.version)))
    }

    /// Increment the patch number (z in x.y.z)
    pub fn increment_patch(&mut self) -> Result<(), RecipeError> {
        let mut version = try!(semver::Version::parse(&self.version));
        version.increment_patch();
        self.version = version.to_string();
        Ok(())
    }

    /// Increment the minor number (y in x.y.z), and set z=0
    #[allow(dead_code)]
    pub fn increment_minor(&mut self) -> Result<(), RecipeError> {
        let mut version = try!(semver::Version::parse(&self.version));
        version.increment_minor();
        self.version = version.to_string();
        Ok(())
    }

    /// Increment the major number (x in x.y.z) and set z=0
    #[allow(dead_code)]
    pub fn increment_major(&mut self) -> Result<(), RecipeError> {
        let mut version = try!(semver::Version::parse(&self.version));
        version.increment_major();
        self.version = version.to_string();
        Ok(())
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
pub fn recipe_filename(name: &str) -> Result<String, RecipeError> {
    if !name.is_empty() {
        Ok(format!("{}.toml", name.replace(" ", "-")))
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


/// Find all tags pointing to a commit id
///
/// # Arguments
///
/// * `commit` - A Commit hash
/// * `repo` - An open repository
///
/// # Retuns
///
/// * A Vec of Tag names of the form <branch>/<filename>/r<rev>
///
/// This searches all repo tags on every call and returns a list of all of the ones pointing
/// to the commit.
///
fn find_commit_tags(repo: &Repository, branch: &str, filename: &str, commit: git2::Oid) -> Result<Vec<String>, git2::Error> {
    let mut tags = Vec::new();
    for r in try!(repo.references_glob(&format!("refs/tags/{}/{}/r*", branch, filename))) {
        // Get the tag reference and the commit it points to. Skip everything else
        let tag_ref = match r {
            Ok(r) => r,
            Err(_) => continue
        };
        let tagged_commit = match tag_ref.peel(git2::ObjectType::Commit) {
            Ok(t) => t,
            Err(_) => continue
        };
        // Is this the commit we are looking for?
        if tagged_commit.id() != commit {
            continue;
        }
        // Convert the tag name into branch/filename/rX string, trimming off refs/tags/
        // Because Rust is really crappy at just trimming off the first 10 characters of a string.
        let tag_name = match tag_ref.name() {
            Some(name) => name.trim_left_matches("refs/tags/").to_string(),
            None => continue
        };
        debug!("find_commit_tags"; "name" => tag_name,
                                   "target" => format!("{:?}", tag_ref.target()),
                                   "tagged commit" => format!("{:?}", tagged_commit.id()));
        tags.push(tag_name);
    }

    Ok(tags)
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
        return Ok(try!(Repository::open(&repo_path)));
    }

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
/// * Result with `true` if written, `false` if skipped because it exists, or a RecipeError
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
    if !replace && read(repo, &recipe.name, branch, None).is_ok() {
        return Ok(false);
    }

    write(repo, &recipe, branch, None)
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
pub fn write(repo: &Repository, recipe: &Recipe, branch: &str, message: Option<&str>) -> Result<bool, RecipeError> {
    // Does the branch exist? If not, create it based on master
    match repo.find_branch(branch, BranchType::Local) {
        Ok(_) => {}
        Err(_) => {
            let parent_commit = try!(find_last_commit(repo));
            try!(repo.branch(branch, &parent_commit, false));
        }
    }

    let branch_id  = try_opt!(try!(repo.find_branch(branch, BranchType::Local)).get().target(), Ok(false));
    debug!("Branch {}'s id is {}", branch, branch_id);

    // Make a copy so we can bump the version if needed
    let mut recipe = recipe.clone();

    // If the new recipe has an empty version, set it to 0.0.1
    if recipe.version == "" {
        recipe.version = "0.0.1".to_string();
    }

    // If it has an invalid semver in version, return an error.
    let new_version = try!(semver::Version::parse(&recipe.version));

    // Save it with sorted packages and modules
    recipe.packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    recipe.modules.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Read the previous version of this recipe, compare its .version to the new one.
    // If they are the same bump the patch level before saving the new one.
    if let Ok(last_version) = read(repo, &recipe.name, branch, None)
                                .and_then(|last_recipe| last_recipe.version()) {
        if last_version == new_version {
            try!(recipe.increment_patch())
        }
    }

    let parent_commit = try!(repo.find_commit(branch_id));
    let blob_id = {
        // NOTE toml::to_string() can fail depending on which struct elements are empty
        // we use try_from to work around this by converting to a Value first.
        let recipe_toml = try!(toml::Value::try_from(&recipe));
        try!(repo.blob(recipe_toml.to_string().as_bytes()))
    };
    let tree_id = {
        let mut tree = repo.treebuilder(Some(&parent_commit.tree().unwrap())).unwrap();
        try!(tree.insert(try!(recipe.filename()), blob_id, 0o100644));
        tree.write().unwrap()
    };
    let tree = try!(repo.find_tree(tree_id));
    let sig = try!(Signature::now("bdcs-api-server", "user-email"));
    let commit_msg = {
        match message {
            Some(msg) => {
                format!("Recipe {}, version {} saved\n\n{}", recipe.name, recipe.version, msg)
            }
            None => {
                format!("Recipe {}, version {} saved", recipe.name, recipe.version)
            }
        }
    };
    let branch_ref = format!("refs/heads/{}", branch);
    try!(repo.commit(Some(&branch_ref), &sig, &sig, &commit_msg, &tree, &[&parent_commit]));
    debug!("Recipe commit:"; "branch" => branch, "recipe_name" => recipe.name, "commit_msg" => commit_msg);

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
    let spec = match commit {
        Some(commit) => format!("{}:{}", commit, try!(recipe_filename(name))),
        None => format!("{}:{}", branch, try!(recipe_filename(name)))
    };
    let object = try!(repo.revparse_single(&spec[..]));
    let blob = try!(repo.find_blob(object.id()));
    let blob_str = try!(str::from_utf8(blob.content()));
    let mut recipe = try!(toml::from_str::<Recipe>(blob_str).or(Err(RecipeError::ParseTOML)));
    recipe.packages.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    recipe.modules.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(recipe)
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
pub fn list(repo: &Repository, branch: &str, _commit: Option<&str>) -> Result<Vec<String>, RecipeError> {
    let mut recipes = Vec::new();

    // TODO use commit instead of branch head if it isn't None
    let branch_id = try_opt!(try!(repo.find_branch(branch, BranchType::Local)).get().target(), Ok(recipes));
    debug!("branch {}'s id is {}", branch, branch_id);

    let parent_commit = try!(repo.find_commit(branch_id));
    let tree = try!(parent_commit.tree());
    for entry in tree.iter() {
        // filenames end with .toml, strip that off and return the base.
        if let Some(name) = entry.name() {
            let recipe_name = name.rsplitn(2, '.').last().unwrap_or("");
            recipes.push(recipe_name.to_string());
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
    let branch_id = try_opt!(try!(repo.find_branch(branch, BranchType::Local)).get().target(), Ok(false));
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
    let branch_id = try_opt!(try!(repo.find_branch(branch, BranchType::Local)).get().target(), Ok(false));
    debug!("Branch {}'s id is {}", branch, branch_id);

    // Find the commit to revert to
    let revert_tree = try!(try!(repo.find_commit(try!(Oid::from_str(commit)))).tree());
    let entry = try_opt!(revert_tree.get_name(&filename), Ok(false));

    let revert_id = entry.id();
    debug!("revert"; "filename" => filename, "id" => format!("{}", revert_id));

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
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>
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
    let re = try!(Regex::new(r"^.*r(\d+)"));

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

    let filename = try!(recipe_filename(name));
    let mut diffopts = DiffOptions::new();
    diffopts.pathspec(&filename);

    let mut commits = Vec::new();
    for id in revwalk {
        let commit = try!(repo.find_commit(try!(id)));
        let tree = try!(commit.tree());
        let tree_entry = tree.get_name(&filename);
        if tree_entry.is_some() {
            // Check to see if the file changed between the parents and this commit
            let m = commit.parents().all(|parent| {
                match_with_parent(repo, &commit, &parent, &mut diffopts)
                .unwrap_or(false)
            });
            if m {
                // Is there a tag pointing to commit?
                let tags = try!(find_commit_tags(repo, branch, &filename, commit.id()));
                if tags.len() > 1 {
                    error!("Too many tags"; "commit" => format!("{:?}", commit.id()),
                                            "tags" => format!("{:?}", tags));
                }
                // Convert the tag to the revision number only
                let revision = {
                    if tags.is_empty() {
                        None
                    } else {
                        match re.captures(&tags[0]) {
                            // Yes, this is a pile of unwraps, but thanks to the regex it should
                            // not fail since it won't match without digits.
                            Some(caps) => Some(caps.get(1).unwrap().as_str().parse().unwrap()),
                            None => None
                        }
                    }
                };

                commits.push(RecipeCommit {
                                commit:   commit.id().to_string(),
                                time:     time_rfc2822(commit.time()),
                                message:  commit.message().unwrap_or("Missing").to_string(),
                                revision: revision
                });
            }
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

    let filename = try!(recipe_filename(name));
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
                    diff_lines.push(l.to_string());
                }
            }
        }
        true
    }));

    Ok(diff_lines)
}


/// Tag a recipe's most recent commit
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name
///
/// # Return
///
/// * true if the tag was successful, false if not or if already tagged.
///
/// This uses git tags, of the form `refs/tags/<branch>/<filename>/r<revision>`
/// Only the most recent recipe commit can be tagged to prevent out of order tagging.
/// Revisions start at 1 and increment for each new commit that is tagged.
/// If the commit has already been tagged it will return false.
///
pub fn tag(repo: &Repository, recipe_name: &str, branch: &str) -> Result<bool, RecipeError> {
    let commits = try!(commits(repo, recipe_name, branch));
    if commits.len() < 1 {
        return Ok(false);
    }

    let mut last_rev = 0;
    for entry in &commits {
        if entry.revision.is_some() {
            if entry.commit == commits[0].commit {
                // There are no new commits since the last revision
                debug!("recipe tag: No new commits"; "name" => recipe_name, "branch" => branch);
                return Ok(false);
            }

            // Extract the revision number from the tag.
            last_rev = entry.revision.unwrap();
            break;
        }
    }
    // At this point we have a new commit in commits[0], and a previous revision in last_rev
    info!("recipe tag"; "commit" => commits[0].commit, "last_rev" => last_rev);

    // Create a new commit with the next revision
    let filename = try!(recipe_filename(recipe_name));
    let name = format!("{}/{}/r{}", branch, filename, last_rev+1);
    let sig = try!(Signature::now("bdcs-api-server", "user-email"));
    let commit_oid = try!(Oid::from_str(&commits[0].commit));
    let target = try!(repo.find_object(commit_oid, Some(git2::ObjectType::Commit)));

    try!(repo.tag(&name, &target, &sig, &name, false));

    Ok(true)
}
