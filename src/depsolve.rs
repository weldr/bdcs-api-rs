use depclose::*;
use itertools::*;
use rpm::*;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;
use std::rc::Rc;
use std::cell::RefCell;

pub fn unit_propagation(exprs: &mut Vec<Rc<RefCell<DepExpression>>>, assignments: &mut HashMap<DepAtom, bool>) -> bool {
    unit_propagation_helper(exprs, assignments, true)
}

// if assign is true, when a unit is found, process it and add it to the cache so that it will be propagated
// if assign is false, propagate units from the cache but do not add new units to the cache
// In short, assign is false when processing an OR. Given an expression like:
//
//    And(A, Not(B), Or(A, And(D, C), E), Or(D, E))
//
// the first pass might result in something like:
//
//    A=true, B=false;
//    And(Or(A, And(D, C), E), Or(D, E))
//
// Then, when processing the Or(A, And(D, C), E), the A can be removed because we know it's true,
// but we shouldn't do anything with And(D, C), since messing with D will mess with the Or(D, E)
// branch. The And(D, C) portion could be false and the parent Or() expression can still be true,
// so don't try to infer anything from those values.
fn unit_propagation_helper(exprs: &mut Vec<Rc<RefCell<DepExpression>>>, assignments: &mut HashMap<DepAtom, bool>, assign: bool) -> bool {
    let mut ever_changed = false;

    loop {
        let mut indices_to_remove = Vec::new();
        let mut indices_to_replace = Vec::new();
        let mut changed = false;

        for (i, val) in exprs.iter().enumerate() {
            match *(val.borrow_mut()) {
                DepExpression::Atom(ref a) => {
                    if !assignments.contains_key(&a) {
                        if assign {
                            assignments.insert(a.clone(), true);
                            changed = true;
                            indices_to_remove.push(i);
                        }
                    } else if assignments.get(&a) == Some(&true) {
                        changed = true;
                        indices_to_remove.push(i);
                    } else {
                        panic!("conflict resolving {}", a);
                    }
                },

                DepExpression::Not(ref rc) => {
                    match *(rc.borrow()) {
                        DepExpression::Atom(ref a) => {
                            if !assignments.contains_key(&a) {
                                if assign {
                                    assignments.insert(a.clone(), false);
                                    changed = true;
                                    indices_to_remove.push(i);
                                }
                            } else if assignments.get(&a) == Some(&false) {
                                changed = true;
                                indices_to_remove.push(i);
                            } else {
                                panic!("conflict resolving {}", a);
                            }
                        },
                        // TODO?
                        _ => ()
                    }
                },

                DepExpression::And(ref mut and_list) => {
                    // recurse on this list of expressions
                    if unit_propagation_helper(and_list, assignments, assign) {
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

                DepExpression::Or(ref mut or_list) => {
                    if unit_propagation_helper(or_list, assignments, false) {
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
                let expr = match *(exprs.index_mut(i)).borrow_mut() {
                    DepExpression::And(ref mut and_list) => and_list.index(0).clone(),
                    DepExpression::Or(ref mut or_list)   => or_list.index(0).clone(),
                    _ => unreachable!()
                };
                expr_ = expr;
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
