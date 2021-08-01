// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use dtasmtime::model_description as MD;
use libc::{c_char, size_t};
use std::ptr;

#[repr(C)]
pub struct DtasmCapabilities {
    pub can_handle_variable_step_size: bool,
    pub can_reset_step: bool,
    pub can_interpolate_inputs: bool,
}

#[repr(C)]
pub struct DtasmModelInfo {
    pub name: *mut c_char,
    pub id: *mut c_char,
    pub description: *mut c_char,
    pub generation_tool: *mut c_char,
    pub generation_date_time: *mut c_char,
    pub name_delimiter: *mut c_char,
    pub capabilities: DtasmCapabilities,
}

#[repr(C)]
pub struct DtasmExperimentInfo {
    pub time_step_min: f64,
    pub time_step_max: f64,
    pub time_step_default: f64,
    pub start_time_default: f64,
    pub end_time_default: f64,
    pub time_unit: *mut c_char,
}

impl Default for DtasmExperimentInfo {
    fn default() -> Self {
        DtasmExperimentInfo {
            time_unit: ptr::null_mut(),
            start_time_default: f64::default(),
            end_time_default: f64::default(),
            time_step_default: f64::default(),
            time_step_max: f64::default(),
            time_step_min: f64::default(),
        }
    }
}

#[repr(C)]
pub enum DtasmVariableType {
    DtasmReal,
    DtasmInt,
    DtasmBool,
    DtasmString,
}

impl From<MD::VariableType> for DtasmVariableType {
    fn from(var_type: MD::VariableType) -> Self {
        match var_type {
            MD::VariableType::DtasmReal => DtasmVariableType::DtasmReal,
            MD::VariableType::DtasmBool => DtasmVariableType::DtasmBool,
            MD::VariableType::DtasmInt => DtasmVariableType::DtasmInt,
            MD::VariableType::DtasmString => DtasmVariableType::DtasmString,
        }
    }
}

#[repr(C)]
pub enum DtasmCausalityType {
    Local,
    Parameter,
    Input,
    Output,
}

impl From<MD::CausalityType> for DtasmCausalityType {
    fn from(caus: MD::CausalityType) -> Self {
        match caus {
            MD::CausalityType::Input => DtasmCausalityType::Input,
            MD::CausalityType::Output => DtasmCausalityType::Output,
            MD::CausalityType::Local => DtasmCausalityType::Local,
            MD::CausalityType::Parameter => DtasmCausalityType::Parameter,
        }
    }
}

#[repr(C)]
pub struct DtasmVariableValue {
    pub real_val: f64,
    pub int_val: i32,
    pub bool_val: bool,
    pub string_val: *mut c_char,
}

impl Default for DtasmVariableValue {
    fn default() -> Self {
        DtasmVariableValue {
            string_val: ptr::null_mut(),
            bool_val: bool::default(),
            int_val: i32::default(),
            real_val: f64::default(),
        }
    }
}

#[repr(C)]
pub struct DtasmModelVariable {
    pub id: i32,
    pub name: *mut c_char,
    pub value_type: DtasmVariableType,
    pub description: *mut c_char,
    pub unit: *mut c_char,
    pub causality: DtasmCausalityType,
    pub derivative_of_id: i32,
    pub default: DtasmVariableValue,
    pub has_default: bool
}

#[repr(C)]
pub struct DtasmModelDescription {
    pub model: DtasmModelInfo,
    pub experiment: DtasmExperimentInfo,
    pub has_experiment: bool,
    pub variables: *mut DtasmModelVariable,
    pub n_variables: size_t,
}

#[repr(C)]
pub enum DtasmStatus {
    DtasmOK,
    DtasmWarning,
    DtasmDiscard,
    DtasmError,
    DtasmFatal,
}

impl From<dtasm_base::types::Status> for DtasmStatus {
    fn from(st: dtasm_base::types::Status) -> Self {
        match st {
            dtasm_base::types::Status::OK => DtasmStatus::DtasmOK,
            dtasm_base::types::Status::Warning => DtasmStatus::DtasmWarning,
            dtasm_base::types::Status::Discard => DtasmStatus::DtasmDiscard,
            dtasm_base::types::Status::Error => DtasmStatus::DtasmError,
        }
    }
}

#[repr(C)]
pub enum DtasmLogLevel {
    DtasmLogError,
    DtasmLogWarn,
    DtasmLogInfo
}

impl Into<dtasm_base::types::LogLevel> for DtasmLogLevel {
    fn into(self) -> dtasm_base::types::LogLevel {
        match self {
            DtasmLogLevel::DtasmLogError => dtasm_base::types::LogLevel::Error,
            DtasmLogLevel::DtasmLogWarn => dtasm_base::types::LogLevel::Warn,
            DtasmLogLevel::DtasmLogInfo => dtasm_base::types::LogLevel::Info,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct DtasmVarValues {
    pub real_values: *mut f64,
    pub real_ids: *mut i32,
    pub n_reals: i32,
    pub int_values: *mut i32,
    pub int_ids: *mut i32,
    pub n_ints: i32,
    pub bool_values: *mut bool,
    pub bool_ids: *mut i32,
    pub n_bools: i32,
    pub string_values: *mut *mut c_char,
    pub string_ids: *mut i32,
    pub n_strings: i32,
}

#[repr(C)]
pub struct DtasmGetValuesResponse {
    pub status: DtasmStatus,
    pub current_time: f64,
    pub values: DtasmVarValues,
}

#[repr(C)]
pub struct DtasmDoStepResponse {
    pub status: DtasmStatus,
    pub updated_time: f64,
}
