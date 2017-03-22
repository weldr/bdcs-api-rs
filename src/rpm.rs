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
use std::str::FromStr;

/// Representation of epoch-version-release data
#[derive(Clone, Debug, Hash)]
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
        match self {
            &ReqOperator::GreaterThanEqual => o == &Ordering::Greater || o == &Ordering::Equal,
            &ReqOperator::GreaterThan      => o == &Ordering::Greater,
            &ReqOperator::EqualTo          => o == &Ordering::Equal,
            &ReqOperator::LessThanEqual    => o == &Ordering::Less || o == &Ordering::Equal,
            &ReqOperator::LessThan         => o == &Ordering::Less
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
                (&None, _) => return true,
                (_, &None) => return true,
                (&Some((ref provides_operator, ref provides_evr)), &Some((ref requires_operator, ref requires_evr))) =>
                    (provides_operator, provides_evr, requires_operator, requires_evr)
        };

        // Special case, oh boy!
        // If the epochs and versions match, one side has no release, and that side is =, >=, or <=, it's a match.
        // e.g. Provides: whatever <= 1.0, Requires: whatever >= 1.0-9
        if provides_evr.epoch.unwrap_or(0) == requires_evr.epoch.unwrap_or(0) &&
                provides_evr.version == requires_evr.version {
            if provides_operator == &Ordering::Equal && provides_evr.release == String::from("") {
                return true;
            }

            if requires_operator == &Ordering::Equal && requires_evr.release == String::from("") {
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

pub fn vercmp(v1: &str, v2: &str) -> Ordering {
    // RPM only cares about ASCII digits, ASCII alphabetic, and tilde
    fn is_version_char(&c: &char) -> bool {
        c.is_ascii() && (c.is_digit(10) || c.is_alphabetic() || c == '~')
    }
    fn not_version_char(&c: &char) -> bool {
        !is_version_char(&c)
    }

    // to avoid overflow, strip leading 0's, and then compare as string, longer string wins
    fn compare_ints(s1: &str, s2: &str) -> Ordering {
        fn is_zero(&c: &char) -> bool { c == '0' }
        let s1_stripped = s1.chars().skip_while(is_zero).collect::<String>();
        let s2_stripped = s2.chars().skip_while(is_zero).collect::<String>();
        let s1_len = s1_stripped.len();
        let s2_len = s2_stripped.len();

        if s1_len > s2_len {
            Ordering::Greater
        } else if s2_len > s1_len {
            Ordering::Less
        } else {
            s1_stripped.cmp(&s2_stripped)
        }
    }

    // Kind of like take_while but you can get to the rest of the string, too
    fn split_at_predicate<'a, P>(s: &'a String, p: P) -> (&'a str, &'a str)  where
            P: Fn(char) -> bool {
        let s_index = s.find(p);
        match s_index {
            Some(i) => s.split_at(i),
            None    => (s.as_str(), "")
        }
    }

    // Is there a way to compose ! and the is_* functions?
    fn not_is_digit(c: char) -> bool {
        !(c.is_ascii() && c.is_digit(10))
    }
    fn not_is_alphabetic(c: char) -> bool {
        !(c.is_ascii() && c.is_alphabetic())
    }

    // Remove all leading characters we don't care about
    let v1_stripped = v1.chars().skip_while(not_version_char).collect::<String>();
    let v2_stripped = v2.chars().skip_while(not_version_char).collect::<String>();

    // If both strings are empty after stripping, the versions are equal
    if v1_stripped.is_empty() && v2_stripped.is_empty() {
        return Ordering::Equal;
    }

    // Tilde is less than everything, including the empty string
    if v1_stripped.starts_with('~') && v2_stripped.starts_with('~') {
        let (_, v1_rest) = v1_stripped.split_at(1);
        let (_, v2_rest) = v2_stripped.split_at(1);
        return vercmp(v1_rest, v2_rest);
    } else if v1_stripped.starts_with('~') {
        return Ordering::Less;
    } else if v2_stripped.starts_with('~') {
        return Ordering::Greater;
    } else if v1_stripped.is_empty() {
        return Ordering::Less;
    } else if v2_stripped.is_empty() {
        return Ordering::Greater;
    }

    // Now we have two definitely not-empty strings that do not start with ~
    // rpm compares strings by digit and non-digit components, so split out
    // the current component

    let v1_first = v1_stripped.clone().chars().nth(0).unwrap();
    let v2_first = v2_stripped.clone().chars().nth(0).unwrap();

    let ((v1_prefix, v1_rest), (v2_prefix, v2_rest)) =
        if v1_first.is_digit(10) {
            (split_at_predicate(&v1_stripped, not_is_digit),
             split_at_predicate(&v2_stripped, not_is_digit))
        } else {
            (split_at_predicate(&v1_stripped, not_is_alphabetic),
             split_at_predicate(&v2_stripped, not_is_alphabetic))
        };


    // Number segments are greater than non-number segments, so if we're looking at an alpha in v1
    // and a number in v2, v2 is greater. The opposite case, v1 being a number and v2 being a non-number,
    // is handled by v1_prefix being empty and therefore less in the eyes of compare_ints.
    if !v1_first.is_digit(10) && v2_first.is_digit(10) {
        Ordering::Less
    } else if v1_first.is_digit(10) {
        match compare_ints(v1_prefix, v2_prefix) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => vercmp(v1_rest, v2_rest)
        }
    } else {
        match v1_prefix.cmp(v2_prefix) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => vercmp(v1_rest, v2_rest)
        }
    }
}
