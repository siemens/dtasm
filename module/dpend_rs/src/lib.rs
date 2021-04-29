// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

mod dpend_types;
mod dpend;

use dtasm_abi::dtasm_generated::dtasm_api as DTAPI;
use dtasm_abi::dtasm_generated::dtasm_types as DTT;
use dtasm_abi::dtasm_generated::dtasm_model_description as DTMD;
use flatbuffers as FB;

use dpend_types::{DpVar,DpState,create_var_maps};

use lazy_static::lazy_static;

use std::slice;
use std::collections::HashMap;
use std::sync::Mutex;

use libc::{c_void,size_t,malloc,free};


static mut MD: Option<DTMD::ModelDescription> = None;

lazy_static! {
    static ref DP_STATE: Mutex<DpState> = {
        Mutex::new(DpState::new())
    };
}

#[no_mangle]
extern "C" fn alloc(size: size_t) -> *mut c_void {
    unsafe { malloc(size) }
}

#[no_mangle]
extern "C" fn dealloc(p: *mut c_void) {
    unsafe { free(p) }
}


#[no_mangle]
extern "C" fn getModelDescription(out_p: *mut u8, max_len: u32) -> u32 {
    assert!(!out_p.is_null());
    let bytes = unsafe { slice::from_raw_parts_mut(out_p, max_len as usize) };

    let md_bytes = include_bytes!("../../dpend/target/modelDescription.fb");
    unsafe { 
        MD = Some(DTMD::get_root_as_model_description(md_bytes));
    };

    if md_bytes.len() > max_len as usize {
        return md_bytes.len() as u32;
    }
    else
    {
        bytes[..md_bytes.len()].copy_from_slice(md_bytes);
    }

    return md_bytes.len() as u32;
}


#[no_mangle]
extern "C" fn init(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{
    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };

    let init_req = FB::get_root::<DTAPI::InitReq>(in_bytes);

    let md = unsafe { MD.expect("ModelDescription not initialized.") };

    let mut state = DP_STATE.lock().unwrap();

    state.t = init_req.starttime();

    create_var_maps(&md, &mut state.var_maps);
    create_default_vals(&md, &mut state);

    init_req.init_values()
        .and_then(|scals| scals.real_vals())
        .map(|reals| for real in reals.iter() {
            let id = real.id();
            let val = real.val();

            let dp_var = state.var_maps.map_id_var.get(&id).unwrap().clone();
            state.var_values.insert(dp_var, val);
        });

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

    let mut builder = FB::FlatBufferBuilder::new_with_capacity(4096);

    let status_res = DTAPI::StatusRes::create(&mut builder, &DTAPI::StatusResArgs{
        status: DTT::Status::OK
    });
    builder.finish(status_res, None);
    let buf = builder.finished_data(); 

    let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

    if buf.len() > out_max_len as usize {
        return buf.len() as u32;
    }
    else
    {
        bytes[..buf.len()].copy_from_slice(buf);
    }

    return buf.len() as u32;
}

#[no_mangle]
extern "C" fn getValues(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{
    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };
    let getvalues_req = FB::get_root::<DTAPI::GetValuesReq>(in_bytes);

    let state = DP_STATE.lock().unwrap();

    let mut get_ok = true;
    let mut id_vals: HashMap<i32, f64> = HashMap::new();

    let get_ids = getvalues_req.ids().expect("Get values request did not contain any variables.");

    for i in 0..get_ids.len() {
        let var_id = get_ids.get(i);
        if !state.var_maps.map_id_var.contains_key(&var_id) {
            get_ok = false;
            continue;
        }
        let var = state.var_maps.map_id_var[&var_id];
        let val = state.var_values[&var];
        id_vals.insert(var_id, val);
    }

    let mut builder = FB::FlatBufferBuilder::new_with_capacity(4096);
    let mut real_offs: Vec<flatbuffers::WIPOffset<DTT::RealVal>> = Vec::new();

    if get_ok {
        for (key, value) in id_vals {
            let real_val = DTT::RealVal::create(&mut builder, &DTT::RealValArgs{
                id: key,
                val: value
            });
            real_offs.push(real_val);
        }
    }

    let real_vals = builder.create_vector(&real_offs);
    let scalar_vals = DTT::VarValues::create(&mut builder, &DTT::VarValuesArgs{
        real_vals: Some(real_vals),
        ..Default::default()
    });

    let getvalues_res = DTAPI::GetValuesRes::create(&mut builder, &DTAPI::GetValuesResArgs{
        status: DTT::Status::OK, 
        current_time: state.t,
        values: Some(scalar_vals),
        ..Default::default()
    });

    builder.finish(getvalues_res, None);
    let buf = builder.finished_data(); // Of type `&[u8]`

    let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

    if buf.len() > out_max_len as usize {
        return buf.len() as u32;
    }
    else
    {
        bytes[..buf.len()].copy_from_slice(buf);
    }

    return buf.len() as u32;
}

#[no_mangle]
extern "C" fn setValues(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{
    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };
    let set_req = FB::get_root::<DTAPI::SetValuesReq>(in_bytes);

    let mut state = DP_STATE.lock().unwrap();

    set_req.values()
        .and_then(|set_vals| set_vals.real_vals())
        .map(|reals| for real in reals.iter() {
            let id = real.id();
            let val = real.val();

            let dp_var = state.var_maps.map_id_var.get(&id).unwrap().clone();
            if dp_var != DpVar::A1 && dp_var != DpVar::A2 {
                println!("Received set request for invalid variable {:?}, ignoring", dp_var);
            }
            else {
                state.var_values.insert(dp_var, val);
            }
        });

    let mut builder = FB::FlatBufferBuilder::new_with_capacity(4096);

    let status_res = DTAPI::StatusRes::create(&mut builder, &DTAPI::StatusResArgs{
        status: DTT::Status::OK
    });
    builder.finish(status_res, None);
    let buf = builder.finished_data(); 

    let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

    if buf.len() > out_max_len as usize {
        return buf.len() as u32;
    }
    else
    {
        bytes[..buf.len()].copy_from_slice(buf);
    }

    return buf.len() as u32;
}



#[no_mangle]
extern "C" fn doStep(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{
    // println!("Received doStep req: in_p {:p}, in_len {}, out_p {:p}, out_max_len {}", in_p, in_len, out_p, out_max_len);

    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };
    let dostep_req = FB::get_root::<DTAPI::DoStepReq>(in_bytes);

    let mut state = DP_STATE.lock().unwrap();

    let current_time = dostep_req.current_time();
    let step = dostep_req.timestep();

    // if (state.t - (float)currentTime > 1.0e-5)
    // {
    //     // printf("Supplied timestep does not match internal state");
    //     exit(1);
    // }

    let mut st = dpend::DpendState {
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
        dt: step, 
        a1: state.var_values[&DpVar::A1],
        a2: state.var_values[&DpVar::A2]
    };

    // println!("Before: {:?}", st);
    dpend::dp_step(&params, &mut st, &input);
    // println!("After: {:?}", st);

    state.t = st.t;
    state.var_values.insert(DpVar::TH1, st.th1);
    state.var_values.insert(DpVar::TH2, st.th2);
    state.var_values.insert(DpVar::W1, st.w1);
    state.var_values.insert(DpVar::W2, st.w2);

    let mut builder = FB::FlatBufferBuilder::new_with_capacity(4096);

    let dostep_res = DTAPI::DoStepRes::create(&mut builder, &DTAPI::DoStepResArgs{
        status: DTT::Status::OK,
        updated_time: st.t
    });
    builder.finish(dostep_res, None);
    let buf = builder.finished_data(); // Of type `&[u8]`

    let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

    if buf.len() > out_max_len as usize {
        return buf.len() as u32;
    }
    else
    {
        bytes[..buf.len()].copy_from_slice(buf);
    }

    return buf.len() as u32;
}

fn create_default_vals(dt_md: &DTMD::ModelDescription,
    dp_state: &mut DpState) {

    let mod_desc_scalars = dt_md.variables();
    let var_maps = &dp_state.var_maps;
    let default_vals = &mut dp_state.var_defaults;
    let init_vals = &mut dp_state.var_values;

    for i in 0..mod_desc_scalars.len() {
        let scalar_var = mod_desc_scalars.get(i);
        let dp_var = var_maps.map_id_var.get(&scalar_var.id())
            .expect(&format!("Unknown variable {}", scalar_var.name()))
            .clone();

        match scalar_var.default() {
            None => {}, 
            Some(def) => {
                default_vals.insert(dp_var, def.real_val());
                init_vals.insert(dp_var, def.real_val());
            }
        }
    }
}