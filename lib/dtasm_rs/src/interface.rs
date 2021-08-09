// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use dtasm_base::types::{DtasmVarValues,LogLevel,Status,GetValuesResponse,DoStepResponse};
use dtasm_base::model_description::ModelDescription;
use dtasm_base::errors::DtasmError;

pub trait DtasmIf: Sync {
    fn get_model_description(&mut self) -> Option<&'static [u8]>;

    fn initialize(&mut self, md: &ModelDescription, initial_vals: &DtasmVarValues, tmin: f64, tmax: Option<f64>, 
        tol: Option<f64>, log_level: LogLevel, check: bool) -> Result<Status, DtasmError>;

    fn get_values(&self, var_ids: &Vec<i32>) -> Result<GetValuesResponse, DtasmError>;

    fn set_values(&mut self, input_vals: &DtasmVarValues) -> Result<Status, DtasmError>;

    fn do_step(&mut self, current_time: f64, timestep: f64) -> Result<DoStepResponse, DtasmError>;
}

