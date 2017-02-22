use depclose::*;
use itertools::*;
use rpm::*;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

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
            Some(Expression::And(Box::new(vec!(Expression::Atom(left), Expression::Not(Box::new(Expression::Atom(right)))))))
        }
    }
}

pub fn unit_propagation(exprs: &mut Vec<Expression>, assignments: &mut HashMap<String, bool>) -> bool {
    let mut ever_changed = false;

    loop {
        let mut indices_to_remove = Vec::new();
        let mut indices_to_replace = Vec::new();
        let mut changed = false;

        for i in 0..exprs.len() {
            match exprs[i] {
                Expression::Atom(ref a) => {
                    if !assignments.contains_key(&a.name) {
                        assignments.insert(a.name.clone(), true);
                        changed = true;
                        indices_to_remove.push(i);
                    } else if assignments.get(&a.name) == Some(&true) {
                        changed = true;
                        indices_to_remove.push(i);
                    } else {
                        panic!("conflict resolving {}", a.name);
                    }
                },

                Expression::Not(box Expression::Atom(ref a)) => {
                    if !assignments.contains_key(&a.name) {
                        assignments.insert(a.name.clone(), false);
                        changed = true;
                        indices_to_remove.push(i);
                    } else if assignments.get(&a.name) == Some(&false) {
                        changed = true;
                        indices_to_remove.push(i);
                    } else {
                        panic!("conflict resolving {}", a.name);
                    }
                },

                Expression::And(box ref mut and_list) => {
                    // recurse on this list of expressions
                    if unit_propagation(and_list, assignments) {
                        changed = true;
                    }

                    // if there's only one thing left in the list, the And is actually just that
                    // thing.
                    if and_list.len() == 1 {
                        indices_to_replace.push(i);
                        changed = true;
                    } else if and_list.is_empty() {
                        indices_to_remove.push(i);
                        changed = true;
                    }
                },

                Expression::Or(box ref mut or_list) => {
                    if unit_propagation(or_list, assignments) {
                        changed = true;
                    }

                    if or_list.len() == 1 {
                        indices_to_replace.push(i);
                        changed = true;
                    } else if or_list.is_empty() {
                        indices_to_remove.push(i);
                        changed = true;
                    }
                },

                _ => { }
            }
        }

        for i in indices_to_replace {
            let mut expr_;
            {
                let expr = match exprs.index_mut(i) {
                    &mut Expression::And(box ref mut and_list) => and_list.index(0),
                    &mut Expression::Or(box ref mut or_list)   => or_list.index(0),
                    _ => unreachable!()
                };
                expr_ = expr.clone();
            }
            exprs.remove(i);
            exprs.insert(i, expr_);
        }

        indices_to_remove.sort_by(|a, b| b.cmp(a));
        for i in indices_to_remove {
            exprs.remove(i);
        }

        if !changed {
            break;
        } else {
            ever_changed = true;
        }
    }

    ever_changed
}
