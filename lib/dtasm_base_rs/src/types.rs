// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use crate::model_description as MD;
use dtasm_abi::dtasm_generated::dtasm_types as DTT;

use std::collections::HashMap;

pub struct DtasmVarType {
    pub name: String, 
    pub value_type: MD::VariableType,
    pub causality: MD::CausalityType,
    pub default: Option<MD::VariableValue>
}

#[derive(Debug,Clone)]
pub struct DtasmVarValues {
    pub real_values: HashMap<i32, f64>,
    pub int_values: HashMap<i32, i32>,
    pub bool_values: HashMap<i32, bool>,
    pub string_values: HashMap<i32, String>,
}

impl DtasmVarValues{
    pub fn new() -> DtasmVarValues {
        DtasmVarValues {
            real_values: HashMap::new(),
            int_values: HashMap::new(),
            bool_values: HashMap::new(),
            string_values: HashMap::new()
        }
    }
}

#[derive(Debug,Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info
}

impl From<DTT::LogLevel> for LogLevel {
    fn from(ll: DTT::LogLevel) -> Self {
        match ll {
            DTT::LogLevel::Info => LogLevel::Info, 
            DTT::LogLevel::Warn => LogLevel::Warn, 
            DTT::LogLevel::Error => LogLevel::Error, 
            _ => LogLevel::Error, 
        }
    }
}

#[derive(Debug,Clone)]
pub enum Status {
    OK,
    Warning,
    Discard,
    Error
}

impl From<DTT::Status> for Status {
    fn from(st: DTT::Status) -> Self {
        match st {
            DTT::Status::OK => Status::OK, 
            DTT::Status::Warning => Status::Warning, 
            DTT::Status::Discard => Status::Discard, 
            DTT::Status::Error => Status::Error,
            _ => Status::Error,
        }
    }
}

impl Into<DTT::Status> for Status {
    fn into(self) -> DTT::Status {
        match self {
            Status::OK => DTT::Status::OK, 
            Status::Warning => DTT::Status::Warning, 
            Status::Discard => DTT::Status::Discard, 
            Status::Error => DTT::Status::Error
        }
    }
}

/// Response from a call to retrieve values of variables. 
///
/// * `status` - Status after the last time step computation.
/// * `current_time` - Current internal time of the instance.
/// * `values` - Current values of the requested variables.
#[derive(Debug,Clone)]
pub struct GetValuesResponse {
    pub status: Status, 
    pub current_time: f64,
    pub values: DtasmVarValues
}

#[derive(Debug,Clone)]
pub struct DoStepResponse {
    pub status: Status, 
    pub updated_time: f64
}
