use depclose::*;
use itertools::*;
use rpm::*;

use r2d2_sqlite::SqliteConnectionManager;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

#[derive (Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    Atom(Requirement),
    And(Box<Vec<Expression>>),
    Or(Box<Vec<Expression>>),
    Not(Box<Expression>)
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::Atom(ref req)   => write!(f, "{}", req),
            Expression::And(ref lst)    => { let strs: String = lst.iter().map(|x| x.to_string()).intersperse(String::from(" AND ")).collect();
                                             write!(f, "{}", strs)
                                           }
            Expression::Not(ref expr)   => write!(f, "NOT {}", expr),
            Expression::Or(ref lst)     => { let strs: String = lst.iter().map(|x| x.to_string()).intersperse(String::from(" OR ")).collect();
                                             write!(f, "{}", strs)
                                           }
        }
    }
}

pub fn proposition_to_expression(p: Proposition, dict: &HashMap<String, Vec<Requirement>>) -> Option<Expression> {
    match p {
        Proposition::Requires(nevra, thing)  => {
            let left_side = Expression::Atom(nevra);
            let right_side = match dict.get(&thing.name) {
                None      => Expression::Atom(thing),
                Some(lst) => { if lst.len() == 1 { Expression::Atom(lst[0].clone()) }
                               else {
                                   // Filter out duplicates in the list of things that must be
                                   // installed because they are required by the left_side.  It
                                   // would be nicer to prevent duplicates from ever getting in
                                   // here in close_dependencies, but that may not be possible.
                                   let tmp: HashSet<Requirement> = lst.clone().into_iter().collect();
                                   Expression::Or(Box::new(tmp.into_iter().map(|x| Expression::Atom(x)).collect()))
                               }
                             }
            };

            // Ignore possibilities like "libidn and libidn".  These should really be filtered out
            // by close_dependencies, but it may not be possible - what if we only know they're
            // equal after using the provided_by_dict?
            if left_side != right_side {
                Some(Expression::And(Box::new(vec!(left_side, right_side))))
            }
            else {
                None
            }
        },
        Proposition::Obsoletes(left, right)  => {
            Some(Expression::And(Box::new(vec!(Expression::Atom(left),
                                               Expression::Not(Box::new(Expression::Atom(right)))))))
        }
    }
}
