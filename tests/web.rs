//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::assert_eq;

use binding_parser_rs::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn assert_results_equal(expected: Path, result: Path) {
    assert_eq!(result.len(), expected.len());

    for (i, node) in result.iter().enumerate() {
        assert_eq!(node, &expected[i]);
    }
}

#[wasm_bindgen_test]
fn basic_empty() {
    let result = parse_binding("").expect("parse_binding failed");
    let expected: Path = vec![];
    assert_results_equal(expected, result);
}

#[wasm_bindgen_test]
fn basic_single() {
    let result = parse_binding("foo").expect("parse_binding failed");
    let expected: Path = vec![AnyNode::from(ValueNode::from("foo"))];

    assert_results_equal(expected, result);
}

#[wasm_bindgen_test]
fn basic_double() {
    let result = parse_binding("foo.bar").expect("parse_binding failed");
    let expected: Path = vec![
        AnyNode::from(ValueNode::from("foo")),
        AnyNode::from(ValueNode::from("bar")),
    ];
    assert_results_equal(expected, result);
}
