use crate::ast::Module;
use crate::common::default::default;
use crate::mem::manager::Dealloc;
use crate::tokenizer::{create_transition_maps, TokenizerState, TransitionMaps};
use super::DataType;

#[derive(Default)]
pub struct AnalyzerParameters {
    data_type: DataType,
}

#[derive(Default)]
pub enum AnalyzerDiagnostic {
    #[default]
    OK,
}

pub struct AnalyzerResults<D: Dealloc> {
    pub module: Module<D>,
    pub diagnostics: Vec<AnalyzerDiagnostic>,
}

pub struct AnalyzerState<D: Dealloc> {
    parameters: AnalyzerParameters,
    tokenizer_state: TokenizerState,
    tokenizer_maps: TransitionMaps,
    module: Module<D>,
    diagnostics: Vec<AnalyzerDiagnostic>,
}

impl<D: Dealloc> AnalyzerState<D> {
    pub fn new(parameters: AnalyzerParameters) -> Self {
        Self {
            parameters,
            tokenizer_state: default(),
            tokenizer_maps: create_transition_maps(),
            module: default(),
            diagnostics: default(),
        }
    }

    /// Updates analyzer state with a next input character; the result is the current count of errors.
    fn push_mut(&mut self, c: char) -> usize {
        for _token in self.tokenizer_state.push_mut(c, &self.tokenizer_maps) {
            // TODO: process the token
        }
        self.diagnostics.len()
    }

    /// Completes the analysis.
    fn end(self) -> AnalyzerResults<D> {
        // TODO: in case the current state is not a valid end state, add an error to self.diagnostics.
        AnalyzerResults {
            module: self.module,
            diagnostics: self.diagnostics,
        }
    }
}
