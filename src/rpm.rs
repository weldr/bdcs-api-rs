//! RPM Types and Functions
//!
//! ## Overview
//!
//! Various utilities for dealing with RPM data
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

use std::ascii::AsciiExt;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;

/// Representation of epoch-version-release data
#[derive(Clone, Debug)]
pub struct EVR {
    pub epoch: Option<u32>,
    pub version: String,
    pub release: String
}

impl Ord for EVR {
    fn cmp(&self, other: &EVR) -> Ordering {
        // no epoch is equivalent to an epoch of 0
        let epoch_1 = self.epoch.unwrap_or(0);
        let epoch_2 = other.epoch.unwrap_or(0);

        if epoch_1 != epoch_2 {
            epoch_1.cmp(&epoch_2)
        } else if vercmp(self.version.as_str(), other.version.as_str()) != Ordering::Equal {
            vercmp(self.version.as_str(), other.version.as_str())
        } else {
            vercmp(self.release.as_str(), other.release.as_str())
        }
    }
}

impl PartialOrd for EVR {
    fn partial_cmp(&self, other: &EVR) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for EVR {
    fn eq(&self, other: &EVR) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for EVR {}

// Since Eq is manually implements, the same needs to happen for Hash.
// It's possible for two versions that are not bitwise-equivalent to
// be equivalent in the eyes of ==, and if those two versions hashed to
// different values then bad things could happen.
impl Hash for EVR {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.epoch.hash(state);

        // use RPMSplit to normalize the version and release before hashing
        RPMSplit::new(self.version.as_str()).collect::<Vec<String>>().hash(state);
        RPMSplit::new(self.release.as_str()).collect::<Vec<String>>().hash(state);
    }
}

impl fmt::Display for EVR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.epoch, self.release.as_str()) {
            (Some(e), "") => write!(f, "{}:{}", e, self.version),
            (Some(e), _)  => write!(f, "{}:{}-{}", e, self.version, self.release),
            (None, "")    => write!(f, "{}", self.version),
            (None, _)     => write!(f, "{}-{}", self.version, self.release)
        }
    }
}

impl FromStr for EVR {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // illegal characters for version and release
        fn illegal_char(c: char) -> bool {
            !(c.is_ascii() && (c.is_digit(10) || c.is_alphabetic() || "._+%{}~".contains(c)))
        }

        // If there's a colon, parse the part before it as an epoch
        let c_index = s.find(':');
        let (epoch, s_rest) = match c_index {
            Some(i) => {
                // Split the string into <epoch-string>, ":version-release", then split again
                // to remove the ':'. Parse the epoch as an unsigned int.
                let (epoch_str, colon_version) = s.split_at(i);
                let epoch = if let Ok(e) = epoch_str.parse::<u32>() {
                    Some (e)
                } else {
                    return Err(String::from("Epoch must be an unsigned int"));
                };
                let (_, s_rest) = colon_version.split_at(1);

                (epoch, s_rest)
            },
            None   => (None, s)
        };

        // version-release is separated by -
        let v_index = s_rest.find('-');
        let (s_version, s_release) = match v_index {
            // The version-release string can't start with '-'
            Some(0) => return Err(String::from("Missing version component")),
            Some(x) => {
                // Return the strings on either side of the hyphen
                let (s_version, hyphen_release) = s_rest.split_at(x);
                let (_, s_release) = hyphen_release.split_at(1);

                // Make sure the release isn't empty
                if s_release.is_empty() {
                    return Err(String::from("Missing release component"));
                }

                (s_version, s_release)
            },
            // No release, just version
            None    => (s_rest, "")
        };

        // check for illegal characters
        if s_version.contains(illegal_char) {
            return Err(String::from(format!("{}: Illegal character in version {}", s, s_version)));
        }
        if s_release.contains(illegal_char) {
            return Err(String::from(format!("{}: Illegal character in release {}", s, s_release)));
        }

        Ok(EVR {epoch: epoch, version: String::from(s_version), release: String::from(s_release)})
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ReqOperator {
    GreaterThanEqual,
    GreaterThan,
    EqualTo,
    LessThanEqual,
    LessThan
}

impl fmt::Display for ReqOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &ReqOperator::GreaterThanEqual => ">=",
            &ReqOperator::GreaterThan      => ">",
            &ReqOperator::EqualTo          => "=",
            &ReqOperator::LessThanEqual    => "<=",
            &ReqOperator::LessThan         => "<"
        })
    }
}

impl FromStr for ReqOperator {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">=" => Ok(ReqOperator::GreaterThanEqual),
            ">"  => Ok(ReqOperator::GreaterThan),
            "="  => Ok(ReqOperator::EqualTo),
            "<=" => Ok(ReqOperator::LessThanEqual),
            "<"  => Ok(ReqOperator::LessThan),
            _    => Err(String::from("Invalid operator"))
        }
    }
}

// Match a ReqOperator with an Operator, so, e.g., >= matches > and =
impl PartialEq<Ordering> for ReqOperator {
    fn eq(&self, o: &Ordering) -> bool {
        match *self {
            ReqOperator::GreaterThanEqual => o == &Ordering::Greater || o == &Ordering::Equal,
            ReqOperator::GreaterThan      => o == &Ordering::Greater,
            ReqOperator::EqualTo          => o == &Ordering::Equal,
            ReqOperator::LessThanEqual    => o == &Ordering::Less || o == &Ordering::Equal,
            ReqOperator::LessThan         => o == &Ordering::Less
        }
    }
}

// Also implement the reverse
impl PartialEq<ReqOperator> for Ordering {
    fn eq(&self, o: &ReqOperator) -> bool {
        o == self
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Requirement {
    pub name: String,
    pub expr: Option<(ReqOperator, EVR)>
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expr {
            Some((ref oper, ref evr)) => write!(f, "{} {} {}", self.name, oper, evr),
            None              => write!(f, "{}", self.name)
        }
    }
}

impl FromStr for Requirement {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl<'a>  From<&'a str> for Requirement {
    // If anything goes wrong, stuff the whole thing in name
    // This way expressions with unparseable versions will be matched against that exact string
    fn from(s: &str) -> Self {
        fn try_from(s: &str) -> Result<Requirement, String> {
            let mut split_str = s.split_whitespace();
            let name = split_str.next().ok_or("Missing requirement name")?;
            let expr = match split_str.next() {
                Some(oper) => Some((oper.parse::<ReqOperator>()?,
                                    split_str.next().ok_or("Missing version in requirement expression")?.parse::<EVR>()?)),
                None       => None
            };

            // Make sure that the whole string has been read
            match split_str.next() {
                None    => Ok(Requirement{name: String::from(name), expr: expr}),
                Some(_) => Err(String::from("Extra data after version"))
            }
        }

        match try_from(s) {
            Ok(req) => req,
            Err(_)  => Requirement{name: s.to_string(), expr: None}
        }
    }
}

impl Requirement {
    pub fn satisfies(&self, requires: &Requirement) -> bool {
        // Names gotta match
        if self.name != requires.name {
            return false;
        }

        // unpack the expression parts
        // If either half is missing the expression, it's a match
        let (provides_operator, provides_evr, requires_operator, requires_evr) =
            match (&self.expr, &requires.expr) {
                (&None, _) | (_, &None) => return true,
                (&Some((ref provides_operator, ref provides_evr)), &Some((ref requires_operator, ref requires_evr))) =>
                    (provides_operator, provides_evr, requires_operator, requires_evr)
        };

        // Special case, oh boy!
        // If the epochs and versions match, one (and only one) side has no release, and that side is =, >=, or <=, it's a match.
        // e.g. Provides: whatever <= 1.0, Requires: whatever >= 1.0-9
        if provides_evr.epoch.unwrap_or(0) == requires_evr.epoch.unwrap_or(0) &&
                provides_evr.version == requires_evr.version {
            if provides_operator == &Ordering::Equal && provides_evr.release == "" && requires_evr.release != "" {
                return true;
            }

            if requires_operator == &Ordering::Equal && requires_evr.release == "" && provides_evr.release != "" {
                return true;
            }
        }

        // Now unravel whether the ranges overlap
        match provides_evr.cmp(requires_evr) {
            // true if Provides: >[=] x || Requires: <[=] y
            Ordering::Less    => provides_operator == &Ordering::Greater || requires_operator == &Ordering::Less,

            // true if Provides <[=] x || Requires: >[=] y
            Ordering::Greater => provides_operator == &Ordering::Less || requires_operator == &Ordering::Greater,

            // true if the directions match
            Ordering::Equal   => (provides_operator == &Ordering::Less && requires_operator == &Ordering::Less) ||
                                 (provides_operator == &Ordering::Equal && requires_operator == &Ordering::Equal) ||
                                 (provides_operator == &Ordering::Greater && requires_operator == &Ordering::Greater)
        }
    }
}

struct RPMSplit<'a> {
    state: Peekable<Chars<'a>>
}

impl<'a> RPMSplit<'a> {
    fn new(s: &str) -> RPMSplit {
        RPMSplit{state: s.chars().peekable()}
    }
}

impl<'a> Iterator for RPMSplit<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        fn is_version_char(c: char) -> bool {
            c.is_ascii() && (c.is_digit(10) || c.is_alphabetic() || c == '~')
        }

        let mut ret = String::new();

        // Skip to the first meaningful char
        let mut next_char = self.state.peek().cloned();
        while let Some(c) = next_char {
            if is_version_char(c) {
                break;
            }
            self.state.next();
            next_char = self.state.peek().cloned();
        }

        match next_char {
            // Figure out what kind of version component this is: number, alpha, or a tilde
            Some(c) => {
                if c == '~' {
                    ret.push('~');
                    self.state.next();
                } else if c.is_digit(10) {
                    // Skip any leading 0's in the number
                    while let Some(c) = next_char {
                        if c != '0' {
                            break;
                        }
                        self.state.next();
                        next_char = self.state.peek().cloned();
                    }

                    // Put the rest of the numbers into ret
                    while let Some(c) = next_char {
                        if !is_version_char(c) || !c.is_digit(10) {
                            break;
                        }
                        ret.push(c);
                        self.state.next();
                        next_char = self.state.peek().cloned();
                    }

                    // See if anything actually got put into ret. If not, the digits were
                    // all 0, push a single 0
                    if ret.is_empty() {
                        ret.push('0');
                    }
                } else {
                    // Put any alpha characters into ret
                    while let Some(c) = next_char {
                        if !is_version_char(c) || !c.is_alphabetic() {
                            break;
                        }
                        ret.push(c);
                        self.state.next();
                        next_char = self.state.peek().cloned();
                    }
                }
            },

            // If nothing was left, iteration is done
            None => return None
        }

        Some(ret)
    }
}

pub fn vercmp(v1: &str, v2: &str) -> Ordering {
    // split up the versions by component
    let mut v1_parts = RPMSplit::new(v1);
    let mut v2_parts = RPMSplit::new(v2);

    vercmp_parts(&mut v1_parts, &mut v2_parts)
}

fn vercmp_parts(v1: &mut RPMSplit, v2: &mut RPMSplit) -> Ordering {
    // to avoid overflow, compare integers as string, longer string wins
    // leading 0's are already stripped
    fn compare_ints(s1: &str, s2: &str) -> Ordering {
        let s1_len = s1.len();
        let s2_len = s2.len();

        if s1_len > s2_len {
            Ordering::Greater
        } else if s2_len > s1_len {
            Ordering::Less
        } else {
            s1.cmp(s2)
        }
    }

    // can't pattern match Option<String> against Option<&str>, thanks rust
    let v1_next = v1.next();
    let v2_next = v2.next();

    // Tilde is less than everything, including the empty string
    if v1_next == Some("~".to_string()) && v2_next == Some("~".to_string()) {
        vercmp_parts(v1, v2)
    } else if v1_next == Some("~".to_string()) {
        Ordering::Less
    } else if v2_next == Some("~".to_string()) {
        Ordering::Greater
    } else {
        match (v1_next, v2_next) {
            // If both are empty, the versions are equal
            (None, None) => Ordering::Equal,

            // If one string is empty and the other is not tilde, empty loses
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,

            (Some(s1), Some(s2)) => {
                // Number segments are greater than non-number segments
                // RPMSplit::next() will never return Some(""), so unwrap should be safe
                let s1_first = s1.chars().next().unwrap();
                let s2_first = s2.chars().next().unwrap();
                if s1_first.is_digit(10) {
                    if !s2_first.is_digit(10) {
                        Ordering::Greater
                    } else {
                        compare_ints(s1.as_str(), s2.as_str()).then_with(|| vercmp_parts(v1, v2))
                    }
                } else if s2_first.is_digit(10) {
                    Ordering::Less
                } else {
                    s1.cmp(&s2).then_with(|| vercmp_parts(v1, v2))
                }
            }
        }
    }
}
