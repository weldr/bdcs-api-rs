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

use std::fs::File;
use std::io;
use std::io::prelude::*;

use glob::{self, glob};
use toml;


/// Recipe Errors
#[derive(Debug)]
pub enum RecipeError {
    Io(io::Error),
    Glob(glob::PatternError),
    ParseTOML
}

impl From<io::Error> for RecipeError {
    fn from(err: io::Error) -> RecipeError {
        RecipeError::Io(err)
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
#[derive(Debug, RustcDecodable, RustcEncodable, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
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
#[derive(Debug, RustcDecodable, RustcEncodable, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Modules {
    pub name: String,
    pub version: Option<String>
}

/// Recipe Packages
///
/// This is used for the Recipe's `packages` section
///
#[derive(Debug, RustcDecodable, RustcEncodable, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Packages {
    pub name: String,
    pub version: Option<String>
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
        let recipe: Recipe = try!(toml::decode_str(&input).ok_or(RecipeError::ParseTOML));
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
        toml::decode_str::<Recipe>(&input).ok_or(RecipeError::ParseTOML)
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
    let recipe_toml = toml::encode::<Recipe>(&recipe);

    let path = format!("{}{}.toml", path, recipe.name.clone().replace(" ", "-"));
    let _ = try!(File::create(&path))
                .write_all(toml::encode_str(&recipe_toml).as_bytes());
    Ok(true)
}
