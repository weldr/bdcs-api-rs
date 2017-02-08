//! Composer Recipe Functions
//!
//! ## Overview
//!
//! Composer recipes are stored as TOML formatted files. This module provides functions for
//! listing, reading, and writing them.
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
use std::fs::{File, OpenOptions, create_dir_all};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use git2::{self, Branch, BranchType, Commit, DiffFormat, DiffOptions, ObjectType, Pathspec, Repository};
use git2::TreeBuilder;
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


/// Composer Recipe
///
/// This is used to parse the full recipe's TOML, and to write a JSON representation of
/// the Recipe.
///
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Recipe {
    pub name: String,
    pub description: Option<String>,
    pub modules: Option<Vec<Modules>>,
    pub packages: Option<Vec<Packages>>
}

/// Recipe Modules
///
/// This is used for the Recipe's `modules` section and can be serialized
/// to/from JSON and TOML.
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
/// If a repo already exists it will be opened and returned.
///
pub fn init_repo(path: &str) -> Result<Repository, RecipeError> {
    let repo_path = Path::new(path).join("git");

    if repo_path.exists() {
        Ok(try!(Repository::open(&repo_path)))
    } else {
        try!(create_dir_all(&repo_path));
        Ok(try!(Repository::init_bare(&repo_path)))
    }
}

/// Add a file to a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `file` - Path to the file to add
/// * `branch` - Name of the branch to add the file to
///
/// # Return
///
/// * Result with () or a RecipeError
///
/// This assumes that the file name is the same as the recipe name.
///
pub fn add_file(repo: &Repository, file: &str, branch: &str) -> Result<(), RecipeError> {

    Ok(())
}

/// Add directory to a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `path` - Path to the directory to add
/// * `branch` - Name of the branch to add the directory contents to
///
/// # Return
///
/// * Result with () or a RecipeError
///
/// This will add all the files in the directory, without recursing into any directories.
/// This assumes that the file names are the same as the recipe names.
///
pub fn add_dir(repo: &Repository, path: &str, branch: &str) -> Result<(), RecipeError> {

    Ok(())
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
/// * Result with () or a RecipeError
///
/// This is used to create a new file, or to write new contents to an existing file.
///
pub fn write_recipe(repo: &Repository, name: &str, recipe: &Recipe, branch: &str) -> Result<(), RecipeError> {

    Ok(())
}

/// Rename a recipe file
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name_orig` - Original Recipe name
/// * `name_new` - New Recipe name
/// * `branch` - Name of the branch to add to
///
/// # Return
///
/// * Result with () or a RecipeError
///
pub fn rename_recipe(repo: &Repository, name_orig: &str, name_new: &str, branch: &str) -> Result<(), RecipeError> {

    Ok(())
}

/// Delete a recipe from a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `name` - Recipe name to write to
/// * `branch` - Name of the branch to add to
///
/// # Return
///
/// * Result with () or a RecipeError
///
pub fn delete_recipe(repo: &Repository, name: &str, branch: &str) -> Result<(), RecipeError> {

    Ok(())
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
/// * An Option with the array of bytes or None, or a RecipeError
///
pub fn read_recipe(repo: &Repository, name: &str, branch: &str, commit: Option<&str>) -> Result<Recipe, RecipeError> {
    Ok(Recipe {
        name: "placeholder".to_string(),
        description: None,
        modules: None,
        packages: None
    })
}

/// List the files in a branch
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `branch` - Name of the branch to list
/// * `commit` - Commit to read from, or None for HEAD
///
/// # Return
///
/// * A Vector of Strings or a RecipeError
///
pub fn list_recipes(repo: &Repository, branch: &str, commit: Option<&str>) -> Result<Vec<String>, RecipeError> {

    Ok(vec!["placeholder".to_string()])
}

/// Recipe Changes
///
/// Details about a change to a recipe
///
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecipeChange {
    pub name: String,
    pub branch: String,
    pub commit: String,
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
/// * A Vector of RecipeChange or a RecipeError
///
/// If the name is None all changes for the branch will be returned.
///
pub fn list_commits(repo: &Repository, name: Option<&str>, branch: &str) -> Result<Vec<RecipeChange>, RecipeError> {

    Ok(vec![RecipeChange {
        name: "placeholder".to_string(),
        branch: "empty".to_string(),
        commit: "empty".to_string(),
        summary: "empty".to_string(),
    }])
}

pub struct RecipeDiff {
    from: String,
    to: String,
    diff: Vec<String>
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
/// * RecipeDiff or a RecipeError
///
/// If new_commit is None HEAD will be used.
///
pub fn recipe_changes(repo: &Repository,
                      name: &str,
                      branch: &str,
                      old_commit: &str,
                      new_commit: Option<&str>) -> Result<RecipeDiff, RecipeError> {

    Ok(RecipeDiff {
        from: "placeholder".to_string(),
        to: "empty".to_string(),
        diff: vec![]
    })
}

/// Diff two recipes
///
/// # Arguments
///
/// * `repo` - An open Repository
/// * `branch` - Name of the branch
/// * `name_1` - First Recipe name
/// * `name_2` - Second Recipe name
/// * `commit_1` - Commit for name_1
/// * `commit_2` - Commit for name_2
///
/// # Return
///
/// * RecipeDiff or a RecipeError
///
/// If commit_1 or commit_2 (or both) are None then HEAD will be used.
///
pub fn recipes_changes(repo: &Repository,
                       branch: &str,
                       name_1: &str,
                       name_2: &str,
                       commit_1: Option<&str>,
                       commit_2: Option<&str>) -> Result<RecipeDiff, RecipeError> {

    Ok(RecipeDiff {
        from: "placeholder".to_string(),
        to: "empty".to_string(),
        diff: vec![]
    })
}

/// Return a list of the recipe names
///
/// # Arguments
///
/// * `path` - path to directory with recipes
///
/// # Return
///
/// * A Vector of Strings Result
///
/// This will read all of the files ending in .toml from the directory path
/// and return a vector of just the recipe names.
///
/// TODO Make this a lazy iterator
///
pub fn list(path: &str) -> Result<Vec<String>, RecipeError> {
    let recipes_glob = path.to_string() + "*.toml";

    let mut result = Vec::new();
    for path in try!(glob(&recipes_glob)).filter_map(Result::ok) {
        // Parse the TOML recipe into a Recipe struct
        let mut input = String::new();
        let _ = try!(File::open(path))
                    .read_to_string(&mut input);
        let recipe: Recipe = try!(toml::from_str(&input).or(Err(RecipeError::ParseTOML)));
        result.push(recipe.name);
    }
    Ok(result)
}


/// Read a recipe TOML file and return a Recipe struct
///
/// # Arguments
///
/// * `path` - path to the recipe TOML file
///
/// # Returns
///
/// * A Recipe Result.
///
pub fn read(path: &str) -> Result<Recipe, RecipeError> {
        let mut input = String::new();
        let _ = try!(File::open(path))
                    .read_to_string(&mut input);
        toml::from_str::<Recipe>(&input).or(Err(RecipeError::ParseTOML))
}


/// Write a Recipe to disk
///
/// # Arguments
///
/// * `path` - path to directory with recipes
///
/// # Returns
///
/// * a bool Result
///
pub fn write(path: &str, recipe: &Recipe) -> Result<bool, RecipeError> {
    let recipe_toml = try!(toml::to_string(&recipe).or(Err(RecipeError::ParseTOML)));

    let path = format!("{}{}.toml", path, recipe.name.clone().replace(" ", "-"));

    let _ = try!(OpenOptions::new()
                 .write(true)
                 .truncate(true)
                 .create(true)
                 .open(&path))
            .write_all(recipe_toml.as_bytes());
    Ok(true)
}
