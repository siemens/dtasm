// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use crate::model_description::{VariableType, CausalityType};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DtasmError {
    #[error("Missing dtasm export symbol: `{0}`")]
    MissingDtasmExport(String), 
    #[error("Invalid invocation order")]
    InvalidCallingOrder,
    #[error("Unkown variable id requested: `{0}`")]
    UnknownVariableId(i32), 
    #[error("Unexpected variable type `{0:#?}` for requested variable id `{1}`")]
    VariableTypeMismatch(VariableType, i32), 
    #[error("Unexpected variable causality `{0:#?}` for requested variable id `{1}`")]
    VariableCausalityMismatch(CausalityType, i32), 
    #[error("Causality `{0:#?}` does not allow to set value for requested variable id `{1}`")]
    VariableCausalityInvalidForSet(CausalityType, i32), 
    #[error("Causality does not allow to set value for requested variable id `{0}`")]
    VariableInvalidForSet(i32), 
    #[error("Internal error in dtasm module: `{0}`")]
    DtasmInternalError(String), 
    #[error("Invalid variable value `{0}` for variable id `{1}`")]
    InvalidVariableValue(String, i32), 
    #[error("Not implemented: `{0}`")]
    NotImplementedError(String)
}