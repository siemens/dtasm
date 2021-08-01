// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

#[derive(Debug,Clone)]
pub struct ModelDescription {
    pub model: ModelInfo, 
    pub variables: Vec<ModelVariable>,
    pub experiment: Option<ExperimentInfo>
}

#[derive(Debug,Clone)]
pub struct ModelInfo {
    pub name: String, 
    pub id: String,
    pub description: String, 
    pub generation_tool: String,
    pub generation_date_time: String, 
    pub name_delimiter: String, 
    pub capabilities: Capabilities
}

#[derive(Debug,Clone)]
pub struct Capabilities {
    pub can_handle_variable_step_size: bool,
    pub can_reset_step: bool,
    pub can_interpolate_inputs: bool
}

#[derive(Debug,Clone)]
pub struct ExperimentInfo {
    pub time_step_min: f64,
    pub time_step_max: f64, 
    pub time_step_default: f64, 
    pub start_time_default: f64,
    pub end_time_default: f64, 
    pub time_unit: String
}

#[derive(Debug,Clone)]
pub struct ModelVariable {
    pub id: i32, 
    pub name: String, 
    pub value_type: VariableType, 
    pub description: String, 
    pub unit: String, 
    pub causality: CausalityType, 
    pub derivative_of_id: i32, 
    pub default: Option<VariableValue>
}

#[derive(Debug,Clone)]
pub struct VariableValue {
    pub real_val: f64,
    pub int_val: i32,
    pub bool_val: bool,
    pub string_val: String
}

#[derive(Debug,Clone,Eq,PartialEq,Copy)]
pub enum VariableType {
    DtasmReal, 
    DtasmInt, 
    DtasmBool,
    DtasmString, 
}

#[derive(Debug,Clone,Eq,PartialEq,Copy)]
pub enum CausalityType {
    Local,
    Parameter,
    Input, 
    Output
}

