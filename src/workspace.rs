//! Welder eorkspace Functions
//!
//! ## Overview
//!
//! Welder recipes are stored as TOML formatted files in a git repository.
//! The workspace is a temporary copy of the recipe while the API is working
//! on it, but before it has been committed.
//!

// Copyright (C) 2017 Red Hat, Inc.
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

use std::fs::{File, OpenOptions, create_dir_all};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use git2::Repository;
use toml;

use recipe::{self, Recipe, recipe_filename};

/// Workspace function errors
#[derive(Debug)]
pub enum WorkspaceError {
    Io(io::Error),
    TomlSer(toml::ser::Error),
    TomlDe(toml::de::Error),
    RecipeError(recipe::RecipeError)
}

impl From<io::Error> for WorkspaceError {
    fn from(err: io::Error) -> WorkspaceError {
        WorkspaceError::Io(err)
    }
}

impl From<toml::ser::Error> for WorkspaceError {
    fn from(err: toml::ser::Error) -> WorkspaceError {
        WorkspaceError::TomlSer(err)
    }
}

impl From<toml::de::Error> for WorkspaceError {
    fn from(err: toml::de::Error) -> WorkspaceError {
        WorkspaceError::TomlDe(err)
    }
}

impl From<recipe::RecipeError> for WorkspaceError {
    fn from(err: recipe::RecipeError) -> WorkspaceError {
        WorkspaceError::RecipeError(err)
    }
}


/// Return the workspace path for a repo branch
///
/// # Arguments
///
/// * `repo` - An open repository
/// * `branch` - Name of a branch
///
/// # Returns
///
/// * A Path with the full path to the branch's workspace directory
///
pub fn workspace_dir(repo: &Repository, branch: &str) -> PathBuf {
    debug!("workspace_dir"; "repo.path()" => format!("{:?}", repo.path()), "branch" => format!("{:?}", branch));
    PathBuf::from(repo.path())
          .join("workspace")
          .join(branch)
}


/// Create the workspace + path directory if it doesn't exist
///
/// # Arguments
///
/// * `dir` - The full path to the directory that should exist
///
/// # Returns
///
/// * Nothing or an error of some kind
///
pub fn check_workspace_dir(dir: &Path) -> Result<(), WorkspaceError> {
    debug!("check_workspace_dir"; "dir" => format!("{:?}", dir));
    if !dir.exists() {
        try!(create_dir_all(dir));
    }
    Ok(())
}

/// Return the full path to a workspace recipe
///
/// # Arguments
///
/// * `workspace` - The full path of the branch's workspace directory
/// * `recipe_filename` - The recipe's filename
///
/// # Returns
///
/// * Full path to the recipe in the workspace
///
fn workspace_recipe_filename(workspace: &Path, recipe_filename: &str) -> PathBuf {
    PathBuf::from(workspace)
            .join(recipe_filename)
}


/// Read a recipe from the workspace
///
/// # Arguments
///
/// * `workspace` - The full path of the branch's workspace directory
/// * `name` - The name of the recipe to read
///
/// # Returns
///
/// * The Recipe, if it exists
///
pub fn read_from_workspace(workspace: &Path, name: &str ) -> Option<Recipe> {
    debug!("read_from_workspace"; "workspace" => format!("{:?}", workspace), "name" => format!("{:?}", name));
    err_opt!(check_workspace_dir(workspace), None);

    let filename = err_opt!(recipe_filename(name), None);
    let ws_filename = workspace_recipe_filename(workspace, &filename);
    let mut input = String::new();
    let _ = err_opt!(File::open(ws_filename), None)
                          .read_to_string(&mut input);
    let recipe = err_opt!(toml::from_str::<Recipe>(&input), None);

    Some(recipe)
}


/// Write a recipe to the workspace
///
/// # Arguments
///
/// * `workspace` - The full path of the branch's workspace directory
///
/// # Returns
///
/// * A Recipe or ...
///
pub fn write_to_workspace(workspace: &Path, recipe: &Recipe) -> Result<(), WorkspaceError> {
    debug!("write_to_workspace"; "workspace" => format!("{:?}", workspace), "recipe" => format!("{:?}", recipe));
    try!(check_workspace_dir(workspace));

    let filename = try!(recipe.filename());
    let ws_filename = workspace_recipe_filename(workspace, &filename);
    debug!("write_to_workspace"; "ws_filename" => format!("{:?}", ws_filename));
    let mut file = try!(OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .truncate(true)
                                    .open(ws_filename));
    let recipe_toml = try!(toml::Value::try_from(&recipe));
    try!(file.write_all(recipe_toml.to_string().as_bytes()));

    Ok(())
}
