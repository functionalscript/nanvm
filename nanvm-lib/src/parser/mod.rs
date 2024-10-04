// Analyzer is a temporary name of a refactored parser that will be merged with the main parser.
// It uses src/ast APIs.
#![allow(clippy::module_inception)]
pub mod analyzer;
pub mod any_state;
pub mod parser;
pub mod path;
pub mod shared;
