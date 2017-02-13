extern crate bdcs;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use r2d2_sqlite::SqliteConnectionManager;
use bdcs::depclose::*;
use std::collections::{HashMap, HashSet};
use std::env;

#[derive (PartialEq, Eq, Hash)]
enum Expression {
    Atom(String),
    Nevra(NEVRA),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>)
}

fn expression_to_string(x: Expression) -> String {
    match x {
        Expression::Atom(a)     => a,
        Expression::Nevra(a)    => a.to_string(),
        Expression::And(a, b)   => format!("{} and {}", expression_to_string(*a), expression_to_string(*b)),
        Expression::Or(a, b)    => format!("{} or {}", expression_to_string(*a), expression_to_string(*b)),
        Expression::Not(a)      => format!("not {}", expression_to_string(*a))
    }
}

fn build_or_expression(lst: &mut Vec<NEVRA>) -> Expression {
    let hd = lst.remove(0);

    if lst.len() == 0 { Expression::Nevra(hd) }
    else { Expression::Or(Box::new(Expression::Nevra(hd)),
                          Box::new(build_or_expression(lst))) }
}

fn proposition_to_expression(p: Proposition, dict: &HashMap<String, Vec<NEVRA>>) -> Option<Expression> {
    match p {
        Proposition::Requires(nevra, thing)  => {
            let left_side = Expression::Nevra(nevra);
            let right_side = match dict.get(&thing) {
                None      => Expression::Atom(thing),
                Some(lst) => { if lst.len() == 1 { Expression::Nevra(lst[0].clone()) }
                               else {
                                   // Filter out duplicates in the list of things that must be
                                   // installed because they are required by the left_side.  It
                                   // would be nicer to prevent duplicates from ever getting in
                                   // here in close_dependencies, but that may not be possible.
                                   let mut our_lst = lst.clone();
                                   our_lst.sort();
                                   our_lst.dedup();
                                   build_or_expression(&mut our_lst)
                               }
                             }
            };

            // Ignore possibilities like "libidn and libidn".  These should really be filtered out
            // by close_dependencies, but it may not be possible - what if we only know they're
            // equal after using the provided_by_dict?
            if left_side != right_side {
                Some(Expression::And(Box::new(left_side), Box::new(right_side)))
            }
            else {
                None
            }
        },
        Proposition::Obsoletes(left, right)  => {
            Some(Expression::And(Box::new(Expression::Atom(left)),
                                 Box::new(Expression::Not(Box::new(Expression::Atom(right))))))
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

    let (props, provided_by_dict) = close_dependencies(&conn, &argv).unwrap_or_default();

    let mut exprs = HashSet::new();

    // Add boolean expressions for each thing that was requested to be installed.
    for thing in argv {
        exprs.insert(Expression::Atom(thing));
    }

    // Convert all the Propositions given by close_dependencies into boolean expressions
    // that can be solved.  This also involves translating Provides into what actually
    // provides them.
    for p in props {
        if let Some(x) = proposition_to_expression(p, &provided_by_dict) {
            exprs.insert(x);
        }
    }

    for x in exprs { println!("{}", expression_to_string(x)) }
}
