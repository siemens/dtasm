// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use dtasm_abi::dtasm_generated::dtasm_types as DTT;
use dtasm_abi::dtasm_generated::dtasm_model_description as DTMD;
use crate::model_description as MD;
use crate::types::DtasmVarType;

pub fn convert_model_description(md: &DTMD::ModelDescription) -> MD::ModelDescription {
    MD::ModelDescription {
        model: convert_model_info(&md.model()), 
        experiment: convert_experiment(md.experiment()), 
        variables: convert_variables(&md)
    }
}

pub fn convert_model_info(mi: &DTMD::ModelInfo) -> MD::ModelInfo {
    MD::ModelInfo {
        id: String::from(mi.id().unwrap_or_default()), 
        name: String::from(mi.name()),
        description: String::from(mi.description().unwrap_or_default()),
        generation_tool: String::from(mi.generation_tool().unwrap_or_default()),
        generation_date_time: String::from(mi.generation_datetime().unwrap_or_default()),
        name_delimiter: String::from(mi.name_delimiter().unwrap_or_default()),
        capabilities: convert_capabilities(&mi.capabilities().unwrap())
    }
}

pub fn convert_capabilities(cap: &DTMD::Capabilities) -> MD::Capabilities {
    MD::Capabilities {
        can_handle_variable_step_size: cap.can_handle_variable_step_size(),
        can_reset_step: cap.can_reset_step(),
        can_interpolate_inputs: cap.can_handle_variable_step_size()
    }
}

pub fn convert_experiment(exp: Option<DTMD::ExperimentInfo>) -> Option<MD::ExperimentInfo> {
    Some(MD::ExperimentInfo {
        time_step_min: exp?.timestep_min(),
        time_step_max: exp?.timestep_max(),
        time_step_default: exp?.timestep_default(), 
        start_time_default: exp?.starttime_default(),
        end_time_default: exp?.endtime_default(), 
        time_unit: String::from(exp?.time_unit().unwrap_or_default())
    })
}

pub fn convert_variables(md: &DTMD::ModelDescription) -> Vec<MD::ModelVariable> {
    let variables = md.variables();

    let mut vars: Vec<MD::ModelVariable> = Vec::new();
    for var in &variables {
        vars.push(
            MD::ModelVariable {
                id: var.id(),
                name: String::from(var.name()),
                causality: convert_causality(var.causality()), 
                value_type: convert_value_type(var.value_type()), 
                description: String::from(var.description().unwrap_or_default()),
                unit: String::from(var.unit().unwrap_or_default()),
                derivative_of_id: var.derivative_of_id(),
                default: convert_variable_value(var.default())
            }
        );
    }

    vars
}

pub fn convert_causality(caus: DTMD::CausalityType) -> MD::CausalityType {
    match caus {
        DTMD::CausalityType::input => MD::CausalityType::Input,
        DTMD::CausalityType::local => MD::CausalityType::Local,
        DTMD::CausalityType::parameter => MD::CausalityType::Parameter,
        DTMD::CausalityType::output => MD::CausalityType::Output,
        _ => MD::CausalityType::Parameter,
    }
}

pub fn convert_value_type(val_type: DTT::VariableType) -> MD::VariableType {
    match val_type {
        DTT::VariableType::DtasmReal => MD::VariableType::DtasmReal,
        DTT::VariableType::DtasmInt => MD::VariableType::DtasmInt,
        DTT::VariableType::DtasmBool => MD::VariableType::DtasmBool, 
        DTT::VariableType::DtasmString => MD::VariableType::DtasmString,
        _ => MD::VariableType::DtasmString,
    }
}

pub fn convert_variable_value(value: Option<DTT::VariableValue>) -> Option<MD::VariableValue> {
    Some(MD::VariableValue {
        real_val: value?.real_val(), 
        int_val: value?.int_val(),
        bool_val: value?.bool_val(), 
        string_val: String::from(value?.string_val().unwrap_or_default())
    })
}

pub fn collect_var_types(md: &MD::ModelDescription) -> HashMap<i32, DtasmVarType> {
    let model_vars = &md.variables;
    let mut var_types: HashMap<i32, DtasmVarType> = HashMap::new();

    for model_var in model_vars.iter() {
        var_types.insert(model_var.id,
            DtasmVarType {
                name: model_var.name.clone(), 
                causality: model_var.causality.clone(),
                value_type: model_var.value_type.clone(),
                default: model_var.default.clone()
            });
    }

    var_types
}

pub fn add_var_types(md: &MD::ModelDescription, var_types: &mut HashMap<i32, DtasmVarType>) {
    let model_vars = &md.variables;

    for model_var in model_vars.iter() {
        var_types.insert(model_var.id,
            DtasmVarType {
                name: model_var.name.clone(), 
                causality: model_var.causality.clone(),
                value_type: model_var.value_type.clone(),
                default: model_var.default.clone()
            });
    }
}