use crate::ast::{
    //Body,
    //Expression,
    //Property,
    Module,
};
use crate::mem::manager::Dealloc;

#[derive(Default)]
pub struct Analyzer<D: Dealloc> {
    module: Module<D>,
}

impl<D: Dealloc> Analyzer<D> {
    /// Push a character to the analyzer; the result is a count of errors.
    fn push(&mut self, _c: char, _line: usize, _col: usize) -> usize {
        0
    }
}
/*
struct AnalyzerState {
    line: usize,
    col: usize,
    errors: usize,
}

impl AnalyzerState {

}
*/
