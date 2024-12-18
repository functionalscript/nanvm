use super::shared::DataType;
use crate::ast::Module;
use crate::common::default::default;
use crate::mem::manager::{Dealloc, Manager};
use crate::tokenizer::{create_transition_maps, JsonToken, TokenizerState, TransitionMaps};

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

pub struct AnalyzerState<M: Manager> {
    parameters: AnalyzerParameters,
    tokenizer_state: TokenizerState<M::Dealloc>,
    tokenizer_maps: TransitionMaps<M>,
    diagnostics_len: usize,
    // TODO: add line number, column number tracking fields (needed for diagnostics).
    module: Module<M::Dealloc>,
    diagnostics: Vec<AnalyzerDiagnostic>,
}

impl<M: Manager + 'static> AnalyzerState<M> {
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
    pub fn push_mut(&mut self, manager: M, c: char) -> usize {
        for token in self
            .tokenizer_state
            .push_mut(manager, c, &self.tokenizer_maps)
        {
            self.process_token(token);
        }
        let prior_diagnostics_len = self.diagnostics_len;
        self.diagnostics_len = self.diagnostics.len();
        self.diagnostics_len - prior_diagnostics_len
    }

    /// Completes the analysis.
    pub fn end(self) -> AnalyzerResults<M::Dealloc> {
        // TODO: in case the current state is not a valid end state, add an error to self.diagnostics.
        AnalyzerResults {
            module: self.module,
            diagnostics: self.diagnostics,
        }
    }

    fn process_token(&mut self, _token: JsonToken<M::Dealloc>) {}
}
