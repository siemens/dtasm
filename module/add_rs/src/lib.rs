// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

mod types;

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::format;

use std::sync::Mutex;

use once_cell::sync::{OnceCell,Lazy};

use dtasm_rs::interface::DtasmIf;
use dtasm_rs::model_description::{ModelDescription};
use dtasm_rs::types::{DoStepResponse, DtasmVarValues, GetValuesResponse, Status};
use dtasm_rs::errors::DtasmError;

use types::{AddState, AddVar};

// TODO: encapsulate this into custom macro, e.g.
// dtasm_module!(DpendMod)
#[no_mangle]
pub static mut SIM_MODULE: Lazy<Box<dyn DtasmIf + Sync + Send>> = Lazy::new(|| Box::new(AddMod));
static ADD_STATE: OnceCell<Mutex<AddState>> = OnceCell::new();


pub struct AddMod;

impl DtasmIf for AddMod {
    fn get_model_description(&mut self) -> Option<&'static [u8]> {
        Some(include_bytes!("../target/modelDescription.fb"))
    }

    fn initialize(&mut self, md: &ModelDescription, _initial_vals: &dtasm_rs::types::DtasmVarValues, tmin: f64, _tmax: Option<f64>, 
        _tol: Option<f64>, _log_level: dtasm_rs::types::LogLevel, _check: bool) -> Result<Status, DtasmError> {
        
        ADD_STATE.set(Mutex::new(AddState::new())).unwrap();
        let mut state = ADD_STATE.get().unwrap().lock().unwrap();

        state.t = tmin;

        types::create_var_maps(&md, &mut state.var_maps);
        
        Ok(Status::OK)
    }

    fn get_values(&self, var_ids: &Vec<i32>) -> Result<GetValuesResponse, DtasmError> {
        let state = ADD_STATE.get().unwrap().lock().unwrap();

        let mut var_vals = DtasmVarValues::new();

        for id in var_ids {
            if !state.var_maps.map_id_var.contains_key(&id) {
                return Err(DtasmError::UnknownVariableId(*id));
            }

            let var = state.var_maps.map_id_var[&id];
            match var {
                AddVar::IO => { 
                    var_vals.int_values.insert(*id, state.int_values[&var]);
                }
                AddVar::RO => {
                    var_vals.real_values.insert(*id, state.real_values[&var]);
                }
                AddVar::BO => {
                    var_vals.bool_values.insert(*id, state.bool_values[&var]);
                }
                AddVar::SO => {
                    var_vals.string_values.insert(*id, state.string_values[&var].clone());
                }
                _ => { return Err(DtasmError::UnknownVariableId(*id)); }
            }
        }

        Ok(GetValuesResponse{
            current_time: state.t, 
            status: Status::OK, 
            values: var_vals
        })
    }

    fn set_values(&mut self, input_vals: &dtasm_rs::types::DtasmVarValues) -> Result<dtasm_rs::types::Status, dtasm_rs::errors::DtasmError> {
        let mut state = ADD_STATE.get().unwrap().lock().unwrap();

        for id in input_vals.real_values.keys(){
            let var = state.var_maps.map_id_var[id];
            let val = input_vals.real_values[id];
            state.real_values.insert(var, val);
        }

        for id in input_vals.int_values.keys(){
            let var = state.var_maps.map_id_var[id];
            let val = input_vals.int_values[id];
            state.int_values.insert(var, val);
        }

        for id in input_vals.bool_values.keys(){
            let var = state.var_maps.map_id_var[id];
            let val = input_vals.bool_values[id];
            state.bool_values.insert(var, val);
        }

        for id in input_vals.string_values.keys(){
            let var = state.var_maps.map_id_var[id];
            let val = input_vals.string_values[id].clone();
            state.string_values.insert(var, val);
        }

        Ok(dtasm_rs::types::Status::OK)
    }

    fn do_step(&mut self, _current_time: f64, timestep: f64) -> Result<dtasm_rs::types::DoStepResponse, dtasm_rs::errors::DtasmError> {
    
        let mut state = ADD_STATE.get().unwrap().lock().unwrap();

        let r_out = state.real_values[&AddVar::RI1] + state.real_values[&AddVar::RI2];
        state.real_values.insert(AddVar::RO, r_out);

        let i_out = state.int_values[&AddVar::II1] + state.int_values[&AddVar::II2];
        state.int_values.insert(AddVar::IO, i_out);

        let b_out = state.bool_values[&AddVar::BI1] && state.bool_values[&AddVar::BI2];
        state.bool_values.insert(AddVar::BO, b_out);

        let s_out = format!("{}{}", state.string_values[&AddVar::SI1], state.string_values[&AddVar::SI2]);
        state.string_values.insert(AddVar::SO, s_out);

        state.t += timestep;
        
        let do_step_res = DoStepResponse {
            status: Status::OK, 
            updated_time: state.t
        }; 

        Ok(do_step_res)
    }
} 

