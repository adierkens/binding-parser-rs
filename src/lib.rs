mod utils;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    io::{Error, ErrorKind},
    result::Result,
};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn parse(raw_binding: &str) -> JsValue {
    let result = parse_binding(raw_binding);

    match result {
        Ok(nodes) => {
            return JsValue::from_serde(&ParserResult::Success(ParserSuccess::new(nodes))).unwrap()
        }
        Err(e) => {
            return JsValue::from_serde(&ParserResult::Error(ParserError::new(e.to_string())))
                .unwrap();
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct PathNode {
    path: Vec<AnyNode>,
}

impl PathNode {
    fn new(path: Vec<AnyNode>) -> PathNode {
        PathNode { path }
    }
}

impl From<Vec<AnyNode>> for PathNode {
    fn from(path: Vec<AnyNode>) -> PathNode {
        PathNode::new(path)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct QueryNode {
    key: AnyNode,
    value: Option<AnyNode>,
}

impl QueryNode {
    fn new(key: AnyNode, value: Option<AnyNode>) -> QueryNode {
        QueryNode { key, value }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ValueNodeValue {
    String(String),
    Number(f32),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ValueNode {
    value: ValueNodeValue,
}

impl From<&str> for ValueNode {
    fn from(value: &str) -> Self {
        ValueNode {
            value: ValueNodeValue::String(value.to_string()),
        }
    }
}

impl From<String> for ValueNode {
    fn from(value: String) -> Self {
        ValueNode {
            value: ValueNodeValue::String(value.to_string()),
        }
    }
}

impl From<f32> for ValueNode {
    fn from(value: f32) -> Self {
        ValueNode {
            value: ValueNodeValue::Number(value),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ExpressionNode {
    value: String,
}

impl From<ValueNode> for ExpressionNode {
    fn from(value: ValueNode) -> Self {
        ExpressionNode {
            value: match value.value {
                ValueNodeValue::String(s) => s,
                ValueNodeValue::Number(n) => n.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ConcatenatedNode {
    value: Vec<ConcatableNode>,
}

impl From<Vec<ConcatableNode>> for ConcatenatedNode {
    fn from(value: Vec<ConcatableNode>) -> Self {
        ConcatenatedNode { value }
    }
}

impl From<Vec<AnyNode>> for ConcatenatedNode {
    fn from(value: Vec<AnyNode>) -> Self {
        let mut nodes: Vec<ConcatableNode> = Vec::new();

        for node in value {
            match node {
                AnyNode::Path(node) => nodes.push(ConcatableNode::Path(*node)),
                AnyNode::Value(node) => nodes.push(ConcatableNode::Value(*node)),
                AnyNode::Expression(node) => nodes.push(ConcatableNode::Expression(*node)),
                _ => panic!("ConcatenatedNode can only contain ConcatableNodes"),
            }
        }

        ConcatenatedNode::from(nodes)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ConcatenatedResult {
    Node(ConcatableNode),
    Concat(ConcatenatedNode),
}

impl ConcatenatedNode {
    fn new(value: Vec<ConcatableNode>) -> ConcatenatedResult {
        if value.len() == 1 {
            return ConcatenatedResult::Node(value[0].clone());
        }

        ConcatenatedResult::Concat(ConcatenatedNode { value })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ConcatableNode {
    Path(PathNode),
    Value(ValueNode),
    Expression(ExpressionNode),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum AnyNode {
    Path(Box<PathNode>),
    Query(Box<QueryNode>),
    Value(Box<ValueNode>),
    Expression(Box<ExpressionNode>),
    Concatenated(Box<ConcatenatedNode>),
}

impl From<PathNode> for AnyNode {
    fn from(path: PathNode) -> Self {
        AnyNode::Path(Box::new(path))
    }
}

impl From<QueryNode> for AnyNode {
    fn from(query: QueryNode) -> Self {
        AnyNode::Query(Box::new(query))
    }
}

impl From<ValueNode> for AnyNode {
    fn from(value: ValueNode) -> Self {
        AnyNode::Value(Box::new(value))
    }
}

impl From<ExpressionNode> for AnyNode {
    fn from(expression: ExpressionNode) -> Self {
        AnyNode::Expression(Box::new(expression))
    }
}

impl From<ConcatenatedNode> for AnyNode {
    fn from(concatenated: ConcatenatedNode) -> Self {
        AnyNode::Concatenated(Box::new(concatenated))
    }
}

pub type Path = Vec<AnyNode>;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ParserSuccess {
    path: Path,
    success: bool,
}

impl ParserSuccess {
    pub fn new(path: Path) -> ParserSuccess {
        ParserSuccess {
            path,
            success: true,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ParserError {
    error: String,
    success: bool,
}

impl ParserError {
    pub fn new(error: String) -> ParserError {
        ParserError {
            error,
            success: false,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ParserResult {
    Success(ParserSuccess),
    Error(ParserError),
}

const SEGMENT_SEPARATOR: char = '.';
const OPEN_CURL: char = '{';
const CLOSE_CURL: char = '}';
const OPEN_BRACKET: char = '[';
const CLOSE_BRACKET: char = ']';
const EQUALS: char = '=';
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const BACK_TICK: char = '`';

const SOME_SEGMENT_SEPARATOR: Option<char> = Some(SEGMENT_SEPARATOR);
const SOME_OPEN_CURL: Option<char> = Some(OPEN_CURL);
const SOME_CLOSE_CURL: Option<char> = Some(CLOSE_CURL);
const SOME_OPEN_BRACKET: Option<char> = Some(OPEN_BRACKET);
const SOME_CLOSE_BRACKET: Option<char> = Some(CLOSE_BRACKET);
const SOME_EQUALS: Option<char> = Some(EQUALS);
const SOME_SINGLE_QUOTE: Option<char> = Some(SINGLE_QUOTE);
const SOME_DOUBLE_QUOTE: Option<char> = Some(DOUBLE_QUOTE);
const SOME_BACK_TICK: Option<char> = Some(BACK_TICK);

fn is_identifier_char(c: Option<char>) -> bool {
    match c {
        Some(c) => c.is_alphanumeric() || c == '_' || c == '-' || c == '@',
        None => false,
    }
}

struct ParsingState {
    binding: String,
    current_index: usize,
    current_char: Option<char>,
}

impl ParsingState {
    fn new(binding: &str) -> ParsingState {
        ParsingState {
            binding: binding.to_string(),
            current_index: 1,
            current_char: binding.chars().nth(0),
        }
    }

    fn next(&mut self, expected: Option<char>) -> Result<Option<char>, Error> {
        if expected.is_some() && self.current_char != expected {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Expected '{}' but found '{}'",
                    expected.unwrap(),
                    self.current_char.unwrap()
                ),
            ));
        }

        self.current_index += 1;
        self.current_char = self.binding.chars().nth(self.current_index as usize);

        Ok(self.current_char)
    }

    fn whitespace(&mut self) -> Result<(), Error> {
        while self.current_char.is_some() && self.current_char.unwrap().is_whitespace() {
            self.next(None)?;
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<Option<ValueNode>, Error> {
        if !is_identifier_char(self.current_char) {
            return Ok(None);
        }

        let mut value = self.current_char.unwrap().to_string();

        while self.next(None).is_ok() {
            if !is_identifier_char(self.current_char) {
                break;
            }
            value.push(self.current_char.unwrap());
        }

        if value.len() == 0 {
            return Ok(None);
        }
        Ok(Some(ValueNode::from(value)))
    }

    fn expression(&mut self) -> Result<Option<ExpressionNode>, Error> {
        if self.current_char != SOME_OPEN_CURL {
            return Ok(None);
        }
        self.next(SOME_OPEN_CURL)?;
        self.whitespace()?;
        let value = self.identifier()?;
        self.whitespace()?;
        self.next(SOME_CLOSE_CURL)?;

        Ok(Some(ExpressionNode::from(value.unwrap())))
    }

    fn nested_path(&mut self) -> Result<Option<PathNode>, Error> {
        if self.current_char == SOME_OPEN_CURL {
            self.next(SOME_OPEN_CURL)?;

            if self.current_char == SOME_OPEN_CURL {
                self.next(SOME_OPEN_CURL)?;

                let path_node = self.parse_path()?;

                self.next(SOME_CLOSE_CURL)?;
                self.next(SOME_CLOSE_CURL)?;

                return Ok(Some(path_node));
            }
        }

        return Ok(None);
    }

    fn simple_segment(&mut self) -> Result<Option<AnyNode>, Error> {
        let nested_path = self.nested_path()?;

        if nested_path.is_some() {
            return Ok(Some(AnyNode::from(nested_path.unwrap())));
        }

        let expression = self.expression()?;
        if expression.is_some() {
            return Ok(Some(AnyNode::from(expression.unwrap())));
        }

        let identifier = self.identifier()?;
        if identifier.is_some() {
            return Ok(Some(AnyNode::from(identifier.unwrap())));
        }

        Ok(None)
    }

    fn segment(&mut self) -> Result<Option<AnyNode>, Error> {
        let mut segments: Vec<AnyNode> = Vec::new();
        let mut next_segment = self.simple_segment()?;

        while next_segment.is_some() {
            segments.push(next_segment.unwrap());
            next_segment = self.simple_segment()?;
        }

        if segments.len() == 0 {
            return Ok(None);
        }

        if segments.len() == 1 {
            return Ok(Some(segments.remove(0)));
        }

        Ok(Some(AnyNode::from(ConcatenatedNode::from(segments))))
    }

    fn regex(&mut self, pattern_match: Regex) -> Result<Option<ValueNode>, Error> {
        if !pattern_match.is_match(self.current_char.unwrap().to_string().as_str()) {
            return Ok(None);
        }

        let mut value = self.current_char.unwrap().to_string();

        while self.next(None).is_ok() {
            if !pattern_match.is_match(self.current_char.unwrap().to_string().as_str()) {
                break;
            }

            value.push(self.current_char.unwrap());
        }

        return Ok(None);
    }

    fn optionally_quoted_segment(&mut self) -> Result<Option<AnyNode>, Error> {
        self.whitespace()?;

        if self.current_char != SOME_SINGLE_QUOTE || self.current_char == SOME_DOUBLE_QUOTE {
            let is_single_quoted = self.current_char == SOME_SINGLE_QUOTE;
            self.next(if is_single_quoted {
                SOME_SINGLE_QUOTE
            } else {
                SOME_DOUBLE_QUOTE
            })?;

            lazy_static! {}

            let ID_REGEX: Regex = Regex::new(r#"[^'"]+"#).unwrap();
            let id = self.regex(ID_REGEX)?;

            self.next(if is_single_quoted {
                SOME_SINGLE_QUOTE
            } else {
                SOME_DOUBLE_QUOTE
            })?;

            return Ok(Some(AnyNode::from(id.unwrap())));
        }

        Ok(None)
    }

    fn equals(&mut self) -> Result<bool, Error> {
        if self.current_char != SOME_EQUALS {
            return Ok(false);
        }

        while self.current_char == SOME_EQUALS {
            self.next(None)?;
        }

        Ok(true)
    }

    fn parse_bracket(&mut self) -> Result<Option<AnyNode>, Error> {
        if self.current_char == SOME_OPEN_BRACKET {
            self.next(SOME_OPEN_BRACKET)?;
            self.whitespace()?;
            let mut value = self.optionally_quoted_segment()?;

            if value.is_some() {
                self.whitespace()?;

                if self.equals()? {
                    self.whitespace()?;
                    let second = self.optionally_quoted_segment()?;
                    value = Some(AnyNode::from(QueryNode {
                        key: value.unwrap(),
                        value: second,
                    }));
                    self.whitespace()?;
                }
            } else {
                return Err(Error::new(ErrorKind::InvalidData, "Expected identifier"));
            }

            if value.is_some() {
                self.next(SOME_CLOSE_BRACKET)?;
            }

            return Ok(value);
        }

        return Ok(None);
    }

    fn parse_segment_and_brackets(&mut self) -> Result<Option<Vec<AnyNode>>, Error> {
        let mut parsed: Vec<AnyNode> = Vec::new();

        let first_segment = self.segment()?;
        if first_segment.is_some() {
            parsed.push(first_segment.unwrap());

            let mut bracket_segment = self.parse_bracket()?;

            while bracket_segment.is_some() {
                parsed.push(bracket_segment.unwrap());
                bracket_segment = self.parse_bracket()?;
            }
        }

        return Ok(Some(parsed));
    }

    fn parse_path(&mut self) -> Result<PathNode, Error> {
        let mut parts: Vec<AnyNode> = Vec::new();

        let mut next_segment = self.parse_segment_and_brackets()?;

        while next_segment.is_some() {
            let mut unwrapped = next_segment.unwrap();

            parts.append(&mut unwrapped);

            if self.current_char.is_none() || self.current_char == SOME_CLOSE_CURL {
                break;
            }

            if unwrapped.len() == 0 && self.current_char.is_some() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Unexpected character: '{}'", self.current_char.unwrap()),
                ));
            }

            self.next(SOME_SEGMENT_SEPARATOR)?;
            next_segment = self.parse_segment_and_brackets()?;
        }

        Ok(PathNode::from(parts))
    }
}

pub fn parse_binding(raw_binding: &str) -> Result<Vec<AnyNode>, String> {
    let mut parsing_state = ParsingState::new(raw_binding);

    let result = parsing_state.parse_path();

    match result {
        Ok(path) => Ok(path.path),
        Err(e) => Err(e.to_string()),
    }
}
