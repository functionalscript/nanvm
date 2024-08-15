use super::DataType;
use crate::ast::Module;
use crate::common::default::default;
use crate::mem::manager::Dealloc;
use crate::tokenizer::{create_transition_maps, TokenizerState, TransitionMaps};

#[derive(Default)]
pub struct AnalyzerParameters {
    data_type: DataType,
}

#[derive(Default)]
pub enum AnalyzerDiagnostic {
    #[default]
    OK,
    // TODO: add error, warning diagnostics.
}

pub struct AnalyzerResults<D: Dealloc> {
    pub module: Module<D>,
    pub diagnostics: Vec<AnalyzerDiagnostic>,
}

pub struct AnalyzerState<D: Dealloc> {
    parameters: AnalyzerParameters,
    tokenizer_state: TokenizerState,
    tokenizer_maps: TransitionMaps,
    diagnostics_len: usize,
    // TODO: add line number, column number tracking fields (needed for diagnostics).
    module: Module<D>,
    diagnostics: Vec<AnalyzerDiagnostic>,
}

impl<D: Dealloc> AnalyzerState<D> {
    /// Creates a new analyzer staring state. The caller should check `diagnostics` for errors
    /// immediately after creation (since `parameters` value can be inconsistent).
    pub fn new(parameters: AnalyzerParameters) -> Self {
        Self {
            parameters,
            tokenizer_state: default(),
            tokenizer_maps: create_transition_maps(),
            module: default(),
            diagnostics: default(),
            diagnostics_len: 0,
        }
    }

    /// Updates analyzer state with a next input character; the result is the increment in the count
    /// of `diagnostics`. It's up to the caller to check what was added at the end of `diagnostics`
    ///  - are there any fatal errors, from the point of view of the current parsing session?
    fn push_mut(&mut self, c: char) -> usize {
        for _token in self.tokenizer_state.push_mut(c, &self.tokenizer_maps) {
            // TODO: process the token.
        }
        let prior_diagnostics_len = self.diagnostics_len;
        self.diagnostics_len = self.diagnostics.len();
        self.diagnostics_len - prior_diagnostics_len
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
