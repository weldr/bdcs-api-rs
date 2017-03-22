use depclose::*;

use rusqlite::Connection;
use std::collections::HashMap;
use std::ops::Index;
use std::ops::IndexMut;
use std::rc::Rc;

pub fn solve_dependencies(conn: &Connection, exprs: &mut Vec<Rc<DepCell<DepExpression>>>) -> Result<Vec<i64>, String> {
    let mut assignments = HashMap::new();

    unit_propagation(exprs, &mut assignments);

    // FIXME:  For now, only return results if unit_propagation was able to do everything.  This
    // should handle most basic cases.  More complicated cases will require real dependency
    // solving.
    if exprs.is_empty() {
        // Take the DepAtom -> bool hash map and convert it to just a list of i64.  We only care
        // about the GroupId for packages that will be installed.
        let results = assignments.into_iter().filter_map(|x| match x {
            (DepAtom::GroupId(i), true) => Some(i),
            _ => None
        }).collect();

        return Ok(results);
    } else {
        return Err(String::from("Unsolved expressions"));
    }
}

fn unit_propagation(exprs: &mut Vec<Rc<DepCell<DepExpression>>>, assignments: &mut HashMap<DepAtom, bool>) -> bool {
    unit_propagation_helper(exprs, assignments, true, &mut 0)
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
//
// change_count is incremented every time a change is made to the assignments hash, and is compared
// to the marker field in DepCell. This way if a given Rc<DepCell<DepExpression>> has already been
// processed since the last assignments change, it does not need to be processed again.
fn unit_propagation_helper(exprs: &mut Vec<Rc<DepCell<DepExpression>>>, assignments: &mut HashMap<DepAtom, bool>, assign: bool, change_count: &mut i64) -> bool {
    let mut ever_changed = false;

    loop {
        let mut indices_to_remove = Vec::new();
        let mut indices_to_replace = Vec::new();
        let mut changed = false;

        for (i, ref mut val) in exprs.iter().enumerate() {
            if val.marker.get() == *change_count {
                continue;
            }

            match *(val.borrow_mut()) {
                DepExpression::Atom(ref a) => {
                    if !assignments.contains_key(&a) {
                        if assign {
                            assignments.insert(a.clone(), true);
                            *change_count = *change_count + 1;
                            changed = true;
                            indices_to_remove.push(i);
                        }
                    } else if assignments.get(&a) == Some(&true) {
                        changed = true;
                        *change_count = *change_count + 1;
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
                                    *change_count = *change_count + 1;
                                    indices_to_remove.push(i);
                                }
                            } else if assignments.get(&a) == Some(&false) {
                                changed = true;
                                *change_count = *change_count + 1;
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
                    if unit_propagation_helper(and_list, assignments, assign, change_count) {
                        changed = true;
                        *change_count = *change_count + 1;
                    }

                    // if there's only one thing left in the list, the And is actually just that
                    // thing.
                    if and_list.len() == 1 {
                        indices_to_replace.push(i);
                        changed = true;
                        *change_count = *change_count + 1;
                    } else if and_list.is_empty() {
                        indices_to_remove.push(i);
                        changed = true;
                        *change_count = *change_count + 1;
                    }
                },

                DepExpression::Or(ref mut or_list) => {
                    // For or, check if there's only one thing first, so we don't waste time
                    // processing the child with assign=false just to (potentially) redo it with
                    // assign=true
                    if or_list.len() == 1 {
                        indices_to_replace.push(i);
                        *change_count = *change_count + 1;
                        changed = true;
                    } else if or_list.is_empty() {
                        indices_to_remove.push(i);
                        changed = true;
                        *change_count = *change_count + 1;
                    } else if unit_propagation_helper(or_list, assignments, false, change_count) {
                        changed = true;
                        *change_count = *change_count + 1;
                    }
                }
            }

            if changed {
                val.marker.set(*change_count);
            }
        }

        for i in indices_to_replace {
            let expr = match *(exprs.index_mut(i)).borrow_mut() {
                DepExpression::And(ref mut and_list) => and_list.index(0).clone(),
                DepExpression::Or(ref mut or_list)   => or_list.index(0).clone(),
                _ => unreachable!()
            };
            exprs.remove(i);
            exprs.insert(i, expr);
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
