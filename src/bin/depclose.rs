extern crate bdcs;
extern crate itertools;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use bdcs::depclose::*;
use bdcs::rpm::*;
use itertools::*;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::process::exit;
use std::str::FromStr;

#[derive (PartialEq, Eq, Hash)]
enum Expression {
    Atom(Requirement),
    And(Vec<Box<Expression>>),
    Or(Vec<Box<Expression>>),
    Not(Box<Expression>)
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::Atom(ref req)   => write!(f, "{}", req),
            Expression::And(ref lst)    => { let strs: String = lst.iter().map(|x| x.to_string()).intersperse(String::from(" AND ")).collect();
                                             write!(f, "{}", strs)
                                           }
            Expression::Not(ref expr)   => write!(f, "{}", expr),
            Expression::Or(ref lst)     => { let strs: String = lst.iter().map(|x| x.to_string()).intersperse(String::from(" OR ")).collect();
                                             write!(f, "{}", strs)
                                           }
        }
    }
}

fn proposition_to_expression(p: Proposition, dict: &HashMap<String, Vec<Requirement>>) -> Option<Expression> {
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
                                   Expression::Or(tmp.into_iter().map(|x| Box::new(Expression::Atom(x))).collect())
                               }
                             }
            };

            // Ignore possibilities like "libidn and libidn".  These should really be filtered out
            // by close_dependencies, but it may not be possible - what if we only know they're
            // equal after using the provided_by_dict?
            if left_side != right_side {
                Some(Expression::And(vec!(Box::new(left_side), Box::new(right_side))))
            }
            else {
                None
            }
        },
        Proposition::Obsoletes(left, right)  => {
            Some(Expression::And(vec!(Box::new(Expression::Atom(left)),
                                      Box::new(Expression::Not(Box::new(Expression::Atom(right)))))))
        }
    }
}

fn main() {
    let mut argv: Vec<String> = env::args().collect();
    if argv.len() < 3 {
        println!("depclose metadata.db RPM [RPM...]");
    }

    // Remove the program, grab the database.
    argv.remove(0);
    let db = argv.remove(0);

    let cfg = r2d2::Config::builder().build();
    let mgr = SqliteConnectionManager::new(db.as_str());
    let pool = r2d2::Pool::new(cfg, mgr).unwrap();

    let conn = pool.get().unwrap();

    let (props, provided_by_dict) = match close_dependencies(&conn, &vec!(String::from("x86_64")), &argv) {
        Err(e)  => { println!("Error: {}", e);
                     exit(1);
                   }
        Ok(tup) => tup
    };

    let mut exprs = HashSet::new();

    // Add boolean expressions for each thing that was requested to be installed.
    for thing in argv {
        exprs.insert(Expression::Atom(Requirement::from_str(thing.as_str()).unwrap()));
    }

    // Convert all the Propositions given by close_dependencies into boolean expressions
    // that can be solved.  This also involves translating Provides into what actually
    // provides them.
    for p in props {
        if let Some(x) = proposition_to_expression(p, &provided_by_dict) {
            exprs.insert(x);
        }
    }

    for x in exprs { println!("{}", x) }
}
