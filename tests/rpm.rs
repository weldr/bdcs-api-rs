//! Tests for the rpm module

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

extern crate bdcs;

use bdcs::rpm::{self, EVR, ReqOperator, Requirement, vercmp};
use std::cmp::Ordering;

#[test]
fn test_evr_ord() {
    fn reverse_ord(o: Ordering) -> Ordering {
        match o {
            Ordering::Greater => Ordering::Less,
            Ordering::Less    => Ordering::Greater,
            Ordering::Equal   => Ordering::Equal
        }
    }

    let evr_test_cases = vec![
        (EVR {epoch: None, version: String::from("1.0"), release: String::from("1")},    EVR {epoch: None, version: String::from("1.0"), release: String::from("1")}, Ordering::Equal),
        (EVR {epoch: Some(0), version: String::from("1.0"), release: String::from("1")}, EVR {epoch: None, version: String::from("1.0"), release: String::from("1")}, Ordering::Equal),
        (EVR {epoch: Some(1), version: String::from("1.0"), release: String::from("1")}, EVR {epoch: None, version: String::from("1.0"), release: String::from("1")}, Ordering::Greater),
        (EVR {epoch: None, version: String::from("1.0"), release: String::from("1")},    EVR {epoch: None, version: String::from("1.1"), release: String::from("1")}, Ordering::Less),
        (EVR {epoch: None, version: String::from("1.0"), release: String::from("1")},    EVR {epoch: None, version: String::from("1.0"), release: String::from("2")}, Ordering::Less),

        // from hawkey's tests/test_subject.c
        (EVR {epoch: Some(8), version: String::from("3.6.9"), release: String::from("11.fc100")}, EVR {epoch: Some(3),  version: String::from("3.6.9"), release: String::from("11.fc100")}, Ordering::Greater),
        (EVR {epoch: Some(8), version: String::from("3.6.9"), release: String::from("11.fc100")}, EVR {epoch: Some(11), version: String::from("3.6.9"), release: String::from("11.fc100")}, Ordering::Less),
        (EVR {epoch: Some(8), version: String::from("3.6.9"), release: String::from("11.fc100")}, EVR {epoch: Some(8),  version: String::from("7.0"),   release: String::from("11.fc100")}, Ordering::Less),
        (EVR {epoch: Some(8), version: String::from("3.6.9"), release: String::from("11.fc100")}, EVR {epoch: Some(8),  version: String::from(""),      release: String::from("11.fc100")}, Ordering::Greater),
        (EVR {epoch: Some(8), version: String::from(""),      release: String::from("11.fc100")}, EVR {epoch: Some(8),  version: String::from(""),      release: String::from("11.fc100")}, Ordering::Equal)
    ];

    for (e1, e2, result) in evr_test_cases {
        // Test both the ordering and the reverse
        assert_eq!(e1.cmp(&e2), result);
        assert_eq!(e2.cmp(&e1), reverse_ord(result));

        // Test that Eq works
        match result {
            Ordering::Equal => assert_eq!(e1, e2),
            _               => assert!(e1 != e2)
        };
    };
}

#[test]
fn test_evr_format() {
    let show_test_cases = vec![
        (EVR {epoch: None, version: String::from("1.0"), release: String::from("1")},    "1.0-1"),
        (EVR {epoch: Some(0), version: String::from("1.0"), release: String::from("1")}, "0:1.0-1"),
        (EVR {epoch: Some(1), version: String::from("1.0"), release: String::from("1")}, "1:1.0-1"),

        (EVR {epoch: Some(8), version: String::from("3.6.9"), release: String::from("11.fc100")}, "8:3.6.9-11.fc100"),

        // empty versions aren't allowed on the parse side, but make sure they at least don't blow up
        (EVR {epoch: None, version: String::from(""), release: String::from("")},                 ""),
        (EVR {epoch: Some(8), version: String::from(""), release: String::from("")},              "8:"),
        (EVR {epoch: None, version: String::from(""), release: String::from("11.fc100")},         "-11.fc100"),
        (EVR {epoch: Some(8), version: String::from(""), release: String::from("11.fc100")},      "8:-11.fc100"),
        (EVR {epoch: None, version: String::from("3.6.9"), release: String::from("")},            "3.6.9"),
        (EVR {epoch: Some(8), version: String::from("3.6.9"), release: String::from("")},         "8:3.6.9"),
    ];

    for (e1, s) in show_test_cases {
        assert_eq!(format!("{}", e1), s);
    };
}

#[cfg(test)]
mod test_evr_parse {
    use rpm::EVR;
    fn parse_evr(s: &str) -> EVR {
        match s.parse::<EVR>() {
            Ok(evr)  => evr,
            Err(err) => panic!("Failed to parse {}: {}", s, err)
        }
    }

    #[test]
    fn good_tests() {
        let parse_test_cases = vec![
            ("1.0-11.fc100",   EVR {epoch: None, version: String::from("1.0"), release: String::from("11.fc100")}),
            ("0:1.0-11.fc100", EVR {epoch: Some(0), version: String::from("1.0"), release: String::from("11.fc100")}),
            ("8:1.0-11.fc100", EVR {epoch: Some(8), version: String::from("1.0"), release: String::from("11.fc100")}),
            ("1.0",            EVR {epoch: None,    version: String::from("1.0"), release: String::from("")}),
            ("8:1.0",          EVR {epoch: Some(8), version: String::from("1.0"), release: String::from("")}),
        ];

        for (s, e1) in parse_test_cases {
            assert!(e1.eq(&parse_evr(s)));
        };
    }

    #[test]
    #[should_panic]
    fn missing_epoch() {
        parse_evr(":1.0-11.fc100");
    }

    #[test]
    #[should_panic]
    fn missing_version() {
        parse_evr("0:-11.fc100");
    }

    #[test]
    #[should_panic]
    fn missing_release() {
        parse_evr("0:1.0-");
    }

    #[test]
    #[should_panic]
    fn invalid_epoch_1() {
        // can't be negative
        parse_evr("-1:1.0-100.fc11");
    }

    #[test]
    #[should_panic]
    fn invalid_epoch_2() {
        // non numeric
        parse_evr("A:1.0-100.fc11");
    }

    #[test]
    #[should_panic]
    fn invalid_epoch_3() {
        // overflow u32
        parse_evr("8589934592:1.0-100.fc11");
    }

    #[test]
    #[should_panic]
    fn invalid_version_1() {
        parse_evr("0:1.0:0-100.fc11");
    }

    #[test]
    #[should_panic]
    fn invalid_version_2() {
        parse_evr("0:1.0&0-100.fc11");
    }

    #[test]
    #[should_panic]
    fn invalid_version_3() {
        parse_evr("0:1.0🌮0-100.fc11");
    }

    #[test]
    #[should_panic]
    fn invalid_release_1() {
        parse_evr("0:1.0-100.fc:11");
    }

    #[test]
    #[should_panic]
    fn invalid_release_2() {
        parse_evr("0:1.0-100.fc&11");
    }

    #[test]
    #[should_panic]
    fn invalid_release_3() {
        parse_evr("0:1.0-100.fc🌮11");
    }
}

#[test]
fn test_operator_display() {
    assert_eq!(format!("{}", ReqOperator::GreaterThanEqual), ">=");
    assert_eq!(format!("{}", ReqOperator::GreaterThan),      ">");
    assert_eq!(format!("{}", ReqOperator::EqualTo),          "=");
    assert_eq!(format!("{}", ReqOperator::LessThanEqual),    "<=");
    assert_eq!(format!("{}", ReqOperator::LessThan),         "<");
}

#[cfg(test)]
mod test_reqoperator_parse {
    use rpm::ReqOperator;
    fn parse_reqoperator(s: &str) -> ReqOperator {
        match s.parse() {
            Ok(evr)  => evr,
            Err(err) => panic!("Failed to parse {}: {}", s, err)
        }
    }

    #[test]
    fn good_tests() {
        assert_eq!(parse_reqoperator(">="), ReqOperator::GreaterThanEqual);
        assert_eq!(parse_reqoperator(">"),  ReqOperator::GreaterThan);
        assert_eq!(parse_reqoperator("="),  ReqOperator::EqualTo);
        assert_eq!(parse_reqoperator("<="), ReqOperator::LessThanEqual);
        assert_eq!(parse_reqoperator("<"),  ReqOperator::LessThan);
    }

    #[test]
    #[should_panic]
    fn empty_test() {
        parse_reqoperator("");
    }

    #[test]
    #[should_panic]
    fn extra_data() {
        parse_reqoperator(">=🌮");
    }

    #[test]
    #[should_panic]
    fn invalid_data() {
        parse_reqoperator("🌮");
    }
}

#[test]
fn test_reqoperator_cmp() {
    assert!(ReqOperator::GreaterThanEqual == Ordering::Greater);
    assert!(ReqOperator::GreaterThanEqual == Ordering::Equal);
    assert!(ReqOperator::GreaterThanEqual != Ordering::Less);

    assert!(ReqOperator::GreaterThan == Ordering::Greater);
    assert!(ReqOperator::GreaterThan != Ordering::Equal);
    assert!(ReqOperator::GreaterThan != Ordering::Less);

    assert!(ReqOperator::EqualTo != Ordering::Greater);
    assert!(ReqOperator::EqualTo == Ordering::Equal);
    assert!(ReqOperator::EqualTo != Ordering::Less);

    assert!(ReqOperator::LessThanEqual != Ordering::Greater);
    assert!(ReqOperator::LessThanEqual == Ordering::Equal);
    assert!(ReqOperator::LessThanEqual == Ordering::Less);

    assert!(ReqOperator::LessThan != Ordering::Greater);
    assert!(ReqOperator::LessThan != Ordering::Equal);
    assert!(ReqOperator::LessThan == Ordering::Less);
}

#[test]
fn test_requirement_format() {
    // assume if one operator works they all work
    let format_test_cases = vec![
        (Requirement {name: String::from("libthing"), expr: None}, "libthing"),
        (Requirement {name: String::from("libthing"), expr: Some((ReqOperator::GreaterThanEqual, EVR {epoch: None, version: String::from("1.0"), release: String::from("1")}))}, "libthing >= 1.0-1"),
    ];

    for (r, s) in format_test_cases {
        assert_eq!(format!("{}", r), s);
    }
}

#[test]
fn test_requirement_parse() {
    let parse_test_cases = vec![
        (Requirement {name: String::from("libthing"), expr: None}, "libthing"),
        (Requirement {name: String::from("libthing"), expr: Some((ReqOperator::GreaterThanEqual, EVR {epoch: None, version: String::from("1.0"), release: String::from("1")}))}, "libthing >= 1.0-1"),
    ];

    for (r, s) in parse_test_cases {
        assert_eq!(s.parse::<Requirement>().unwrap(), r);
    }
}

#[test]
fn satisfies_tests() {
    // provides, requires, true/false
    let test_cases = vec![
        ("no", "match", false),

        ("thing",          "thing",          true),
        ("thing",          "thing >= 1.0-1", true),
        ("thing >= 1.0-1", "thing",          true),

        ("thing = 1.0-1",  "thing = 1.0-1",  true),
        ("thing = 1.0-1",  "thing >= 1.0-1", true),
        ("thing = 1.0-1",  "thing > 1.0-1",  false),
        ("thing = 1.0-1",  "thing < 1.0-1",  false),
        ("thing = 1.0-1",  "thing <= 1.0-1", true),

        ("thing = 1.0",    "thing = 1.0-9",   true),
        ("thing = 1.0",    "thing < 1.0-9",   true),
        ("thing = 1.0",    "thing <= 1.0-9",  true),
        ("thing = 1.0",    "thing <= 1.0-9",  true),
        ("thing = 1.0",    "thing = 1.0-9",   true),

        ("thing <= 1.0",   "thing = 1.0-9",   true),
        ("thing <= 1.0",   "thing < 1.0-9",   true),
        ("thing <= 1.0",   "thing <= 1.0-9",  true),
        ("thing <= 1.0",   "thing <= 1.0-9",  true),
        ("thing <= 1.0",   "thing = 1.0-9",   true),

        ("thing >= 1.0",   "thing = 1.0-9",   true),
        ("thing >= 1.0",   "thing < 1.0-9",   true),
        ("thing >= 1.0",   "thing <= 1.0-9",  true),
        ("thing >= 1.0",   "thing <= 1.0-9",  true),
        ("thing >= 1.0",   "thing = 1.0-9",   true),

        ("thing = 1.0-9",  "thing = 1.0-9",   true),
        ("thing < 1.0-9",  "thing = 1.0-9",   false),
        ("thing <= 1.0-9", "thing = 1.0-9",   true),
        ("thing > 1.0-9",  "thing = 1.0-9",   false),
        ("thing >= 1.0-9", "thing = 1.0-9",   true),

        ("thing >= 1.0-1", "thing = 1.0-1",  true),
        ("thing >= 1.0-1", "thing >= 1.0-1", true),
        ("thing >= 1.0-1", "thing > 1.0-1",  true),
        ("thing >= 1.0-1", "thing < 1.0-1",  false),
        ("thing >= 1.0-1", "thing <= 1.0-1", true),

        ("thing > 1.0-1",  "thing = 1.0-1",  false),
        ("thing > 1.0-1",  "thing >= 1.0-1", true),
        ("thing > 1.0-1",  "thing > 1.0-1",  true),
        ("thing > 1.0-1",  "thing < 1.0-1",  false),
        ("thing > 1.0-1",  "thing <= 1.0-1", false),

        ("thing < 1.0-1",  "thing = 1.0-1",  false),
        ("thing < 1.0-1",  "thing >= 1.0-1", false),
        ("thing < 1.0-1",  "thing > 1.0-1",  false),
        ("thing < 1.0-1",  "thing < 1.0-1",  true),
        ("thing < 1.0-1",  "thing <= 1.0-1", true),

        ("thing <= 1.0-1", "thing = 1.0-1",  true),
        ("thing <= 1.0-1", "thing >= 1.0-1", true),
        ("thing <= 1.0-1", "thing > 1.0-1",  false),
        ("thing <= 1.0-1", "thing < 1.0-1",  true),
        ("thing <= 1.0-1", "thing <= 1.0-1", true),

        ("thing = 9.0",    "thing = 1.0-1",  false),
        ("thing = 9.0",    "thing >= 1.0-1", true),
        ("thing = 9.0",    "thing > 1.0-1",  true),
        ("thing = 9.0",    "thing <= 1.0-1", false),
        ("thing = 9.0",    "thing < 1.0-1",  false),

        ("thing < 9.0",    "thing = 1.0-1",  true),
        ("thing < 9.0",    "thing >= 1.0-1", true),
        ("thing < 9.0",    "thing > 1.0-1",  true),
        ("thing < 9.0",    "thing <= 1.0-1", true),
        ("thing < 9.0",    "thing < 1.0-1",  true),

        ("thing <= 9.0",   "thing = 1.0-1",  true),
        ("thing <= 9.0",   "thing >= 1.0-1", true),
        ("thing <= 9.0",   "thing > 1.0-1",  true),
        ("thing <= 9.0",   "thing <= 1.0-1", true),
        ("thing <= 9.0",   "thing < 1.0-1",  true),

        ("thing > 9.0",    "thing = 1.0-1",  false),
        ("thing > 9.0",    "thing >= 1.0-1", true),
        ("thing > 9.0",    "thing > 1.0-1",  true),
        ("thing > 9.0",    "thing <= 1.0-1", false),
        ("thing > 9.0",    "thing < 1.0-1",  false),

        ("thing >= 9.0",   "thing = 1.0-1",  false),
        ("thing >= 9.0",   "thing >= 1.0-1", true),
        ("thing >= 9.0",   "thing > 1.0-1",  true),
        ("thing >= 9.0",   "thing <= 1.0-1", false),
        ("thing >= 9.0",   "thing < 1.0-1",  false),

        ("thing = 1.0",    "thing = 9.0-1",  false),
        ("thing = 1.0",    "thing >= 9.0-1", false),
        ("thing = 1.0",    "thing > 9.0-1",  false),
        ("thing = 1.0",    "thing <= 9.0-1", true),
        ("thing = 1.0",    "thing < 9.0-1",  true),

        ("thing < 1.0",    "thing = 9.0-1",  false),
        ("thing < 1.0",    "thing >= 9.0-1", false),
        ("thing < 1.0",    "thing > 9.0-1",  false),
        ("thing < 1.0",    "thing <= 9.0-1", true),
        ("thing < 1.0",    "thing < 9.0-1",  true),

        ("thing <= 1.0",   "thing = 9.0-1",  false),
        ("thing <= 1.0",   "thing >= 9.0-1", false),
        ("thing <= 1.0",   "thing > 9.0-1",  false),
        ("thing <= 1.0",   "thing <= 9.0-1", true),
        ("thing <= 1.0",   "thing < 9.0-1",  true),

        ("thing >= 1.0",   "thing = 9.0-1",  true),
        ("thing >= 1.0",   "thing >= 9.0-1", true),
        ("thing >= 1.0",   "thing > 9.0-1",  true),
        ("thing >= 1.0",   "thing <= 9.0-1", true),
        ("thing >= 1.0",   "thing < 9.0-1",  true),

        ("thing > 1.0",    "thing = 9.0-1",  true),
        ("thing > 1.0",    "thing >= 9.0-1", true),
        ("thing > 1.0",    "thing > 9.0-1",  true),
        ("thing > 1.0",    "thing <= 9.0-1", true),
        ("thing > 1.0",    "thing < 9.0-1",  true),
    ];

    for (s1, s2, result) in test_cases {
        let r1:Requirement = s1.parse().unwrap();
        let r2:Requirement = s2.parse().unwrap();
        if r1.satisfies(&r2) != result {
            panic!("Failed to satisfy: Provides: {}, Requires: {}, result: {}", r1, r2, result);
        }
    }
}

#[test]
fn test_vercmp() {
    // These are from tests/rpmvercmp.at in the rpm source
    let vercmp_test_cases = vec![
        ("1.0", "1.0", Ordering::Equal),
        ("1.0", "2.0", Ordering::Less),
        ("2.0", "1.0", Ordering::Greater),

        ("2.0.1", "2.0.1", Ordering::Equal),
        ("2.0", "2.0.1", Ordering::Less),
        ("2.0.1", "2.0", Ordering::Greater),

        ("2.0.1a", "2.0.1a", Ordering::Equal),
        ("2.0.1a", "2.0.1", Ordering::Greater),
        ("2.0.1", "2.0.1a", Ordering::Less),

        ("5.5p1", "5.5p1", Ordering::Equal),
        ("5.5p1", "5.5p2", Ordering::Less),
        ("5.5p2", "5.5p1", Ordering::Greater),

        ("5.5p10", "5.5p10", Ordering::Equal),
        ("5.5p1", "5.5p10", Ordering::Less),
        ("5.5p10", "5.5p1", Ordering::Greater),

        ("10xyz", "10.1xyz", Ordering::Less),
        ("10.1xyz", "10xyz", Ordering::Greater),

        ("xyz10", "xyz10", Ordering::Equal),
        ("xyz10", "xyz10.1", Ordering::Less),
        ("xyz10.1", "xyz10", Ordering::Greater),

        ("xyz.4", "xyz.4", Ordering::Equal),
        ("xyz.4", "8", Ordering::Less),
        ("8", "xyz.4", Ordering::Greater),
        ("xyz.4", "2", Ordering::Less),
        ("2", "xyz.4", Ordering::Greater),

        ("5.5p2", "5.6p1", Ordering::Less),
        ("5.6p1", "5.5p2", Ordering::Greater),

        ("5.6p1", "6.5p1", Ordering::Less),
        ("6.5p1", "5.6p1", Ordering::Greater),

        ("6.0.rc1", "6.0", Ordering::Greater),
        ("6.0", "6.0.rc1", Ordering::Less),

        ("10b2", "10a1", Ordering::Greater),
        ("10a2", "10b2", Ordering::Less),
        ("1.0aa", "1.0aa", Ordering::Equal),
        ("1.0a", "1.0aa", Ordering::Less),
        ("1.0aa", "1.0a", Ordering::Greater),

        ("10.0001", "10.0001", Ordering::Equal),
        ("10.0001", "10.1", Ordering::Equal),
        ("10.1", "10.0001", Ordering::Equal),
        ("10.0001", "10.0039", Ordering::Less),
        ("10.0039", "10.0001", Ordering::Greater),

        ("4.999.9", "5.0", Ordering::Less),
        ("5.0", "4.999.9", Ordering::Greater),

        ("20101121", "20101121", Ordering::Equal),
        ("20101121", "20101122", Ordering::Less),
        ("20101122", "20101121", Ordering::Greater),

        ("2_0", "2_0", Ordering::Equal),
        ("2.0", "2_0", Ordering::Equal),
        ("2_0", "2.0", Ordering::Equal),

        ("a", "a", Ordering::Equal),
        ("a+", "a+", Ordering::Equal),
        ("a+", "a_", Ordering::Equal),
        ("a_", "a+", Ordering::Equal),
        ("+a", "+a", Ordering::Equal),
        ("+a", "_a", Ordering::Equal),
        ("_a", "+a", Ordering::Equal),
        ("+_", "+_", Ordering::Equal),
        ("_+", "+_", Ordering::Equal),
        ("_+", "_+", Ordering::Equal),
        ("+", "_", Ordering::Equal),
        ("_", "+", Ordering::Equal),

        ("1.0~rc1", "1.0~rc1", Ordering::Equal),
        ("1.0~rc1", "1.0", Ordering::Less),
        ("1.0", "1.0~rc1", Ordering::Greater),
        ("1.0~rc1", "1.0~rc2", Ordering::Less),
        ("1.0~rc2", "1.0~rc1", Ordering::Greater),
        ("1.0~rc1~git123", "1.0~rc1~git123", Ordering::Equal),
        ("1.0~rc1~git123", "1.0~rc1", Ordering::Less),
        ("1.0~rc1", "1.0~rc1~git123", Ordering::Greater)
    ];

    for (s1, s2, result) in vercmp_test_cases {
        assert_eq!(vercmp(&s1, &s2), result);
    };
}
