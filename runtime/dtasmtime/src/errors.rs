// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT
use thiserror::Error;
use dtasm_base::errors;

#[derive(Error, Debug)]
pub enum DtasmtimeError {
    #[error(transparent)]
    ModuleError(#[from] anyhow::Error),
    #[error(transparent)]
    ModuleTrapError(#[from] wasmtime::Trap),
    #[error(transparent)]
    DtasmError(#[from] errors::DtasmError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
