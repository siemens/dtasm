mod types;
mod dpend;

use std::sync::Mutex;

use once_cell::sync::Lazy;

use dtasm_rs::interface::DtasmIf;
use dtasm_rs::model_description::{ModelDescription};
use dtasm_rs::types::{DoStepResponse, DtasmVarValues, GetValuesResponse, Status};
use dtasm_rs::errors::DtasmError;

use types::{DpState,DpVar,create_var_maps,create_default_vals};

// TODO: encapsulate this into custom macro, e.g.
// dtasm_module!(DpendMod)
#[no_mangle]
pub static mut SIM_MODULE: Lazy<Box<dyn DtasmIf + Sync + Send>> = Lazy::new(|| Box::new(DpendMod));

static DP_STATE: Lazy<Mutex<DpState>> = Lazy::new(|| Mutex::new(DpState::new()));


pub struct DpendMod;

impl DtasmIf for DpendMod {
    fn get_model_description(&mut self) -> Option<&'static [u8]> {
        Some(include_bytes!("../../dpend/target/modelDescription.fb"))
    }

    fn initialize(&mut self, md: &ModelDescription, initial_vals: &dtasm_rs::types::DtasmVarValues, tmin: f64, _tmax: Option<f64>, 
        _tol: Option<f64>, _log_level: dtasm_rs::types::LogLevel, _check: bool) -> Result<Status, DtasmError> {
        
        let mut state = DP_STATE.lock().unwrap();
        state.t = tmin;

        create_var_maps(&md, &mut state.var_maps);
        create_default_vals(&md, &mut state);
        
        for (id, val) in &initial_vals.real_values {
            let dp_var = state.var_maps.map_id_var.get(&id).unwrap().clone();
            state.var_values.insert(dp_var, *val);
        };
        
        if state.var_values.contains_key(&DpVar::M1){
            state.params.m1 = *state.var_values.get(&DpVar::M1).unwrap();
        }
    
        if state.var_values.contains_key(&DpVar::M2){
            state.params.m2 = *state.var_values.get(&DpVar::M2).unwrap();
        }
    
        if state.var_values.contains_key(&DpVar::L1){
            state.params.l1 = *state.var_values.get(&DpVar::L1).unwrap();
        }
    
        if state.var_values.contains_key(&DpVar::L2){
            state.params.l2 = *state.var_values.get(&DpVar::L2).unwrap();
        }

        Ok(Status::OK)
    }

    fn get_values(&self, var_ids: &Vec<i32>) -> Result<GetValuesResponse, DtasmError> {
        let state = DP_STATE.lock().unwrap();

        let mut var_vals = DtasmVarValues::new();
        let id_vals = &mut var_vals.real_values;

        for id in var_ids {
            if !state.var_maps.map_id_var.contains_key(&id) {
                return Err(DtasmError::UnknownVariableId(*id));
            }

            let var = state.var_maps.map_id_var[&id];
            let val = state.var_values[&var];
            id_vals.insert(*id, val);
        }

        Ok(GetValuesResponse{
            current_time: state.t, 
            status: Status::OK, 
            values: var_vals
        })
    }

    fn set_values(&mut self, _input_vals: &dtasm_rs::types::DtasmVarValues) -> Result<dtasm_rs::types::Status, dtasm_rs::errors::DtasmError> {
        let mut state = DP_STATE.lock().unwrap();

        for id in _input_vals.real_values.keys(){
            let var = state.var_maps.map_id_var[id];
            let val = _input_vals.real_values[id];
            match var {
                DpVar::A1 => { state.var_values.insert(DpVar::A1, val); },
                DpVar::A2 => { state.var_values.insert(DpVar::A2, val); },
                _ => ()
            }
        }

        Ok(dtasm_rs::types::Status::OK)
    }

    fn do_step(&mut self, current_time: f64, timestep: f64) -> Result<dtasm_rs::types::DoStepResponse, dtasm_rs::errors::DtasmError> {
    
        let mut state = DP_STATE.lock().unwrap();
        // if (state.t - (float)currentTime > 1.0e-5)
        // {
        //     // printf("Supplied timestep does not match internal state");
        //     exit(1);
        // }
    
        let mut dp_state = dpend::DpendState {
            t: current_time, 
            th1: state.var_values[&DpVar::TH1], 
            th2: state.var_values[&DpVar::TH2], 
            w1: state.var_values[&DpVar::W1], 
            w2: state.var_values[&DpVar::W2]
        };
    
        let params = dpend::DpendParams {
            m1: state.params.m1, 
            m2: state.params.m2, 
            l1: state.params.l1, 
            l2: state.params.l2
        };
    
        let input = dpend::DpendInput {
            dt: timestep, 
            a1: state.var_values[&DpVar::A1],
            a2: state.var_values[&DpVar::A2]
        };
    
        // println!("Before: {:?}", st);
        dpend::dp_step(&params, &mut dp_state, &input);
        // println!("After: {:?}", st);
    
        state.t = dp_state.t;
        state.var_values.insert(DpVar::TH1, dp_state.th1);
        state.var_values.insert(DpVar::TH2, dp_state.th2);
        state.var_values.insert(DpVar::W1, dp_state.w1);
        state.var_values.insert(DpVar::W2, dp_state.w2);

        let do_step_res = DoStepResponse {
            status: Status::OK, 
            updated_time: state.t
        }; 

        Ok(do_step_res)
    }
} 

