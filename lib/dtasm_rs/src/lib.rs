// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

pub mod interface;
pub use dtasm_base::{types,model_description,errors};

use dtasm_abi::dtasm_generated::dtasm_api as DTAPI;
use dtasm_abi::dtasm_generated::dtasm_types as DTT;
use dtasm_abi::dtasm_generated::dtasm_model_description as DTMD;
use flatbuffers as FB;

use libc::{c_void,size_t,malloc,free};
use once_cell::sync::Lazy;

use std::slice;
use std::collections::HashMap;
use std::sync::Mutex;

use dtasm_base::model_conversion::{convert_model_description,collect_var_types};
use dtasm_base::types::{DtasmVarType,DtasmVarValues};

extern "Rust" {
    static mut SIM_MODULE: Lazy<Box<dyn interface::DtasmIf + Sync + Send>>;
}

static VARTYPES: Lazy<Mutex<HashMap<i32, DtasmVarType>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static MDBYTES: Lazy<Mutex<&[u8]>> = Lazy::new(|| Mutex::new(&[]));
static FBBUILDER: Lazy<Mutex<FB::FlatBufferBuilder>> = Lazy::new(|| Mutex::new(FB::FlatBufferBuilder::new_with_capacity(4096)));

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

    let mut md_bytes = MDBYTES.lock().unwrap();

    if md_bytes.len() == 0 {
        *md_bytes = unsafe { SIM_MODULE.get_model_description().unwrap() };

        let md_dtasm = DTMD::get_root_as_model_description(*md_bytes);
        let md = convert_model_description(&md_dtasm);

        let mut var_types = VARTYPES.lock().unwrap();
        *var_types = collect_var_types(&md);
    }

    if md_bytes.len() > max_len as usize {
        return md_bytes.len() as u32;
    }
    else
    {
        bytes[..md_bytes.len()].copy_from_slice(*md_bytes);
    }

    return md_bytes.len() as u32;
}

#[no_mangle]
extern "C" fn init(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{
    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };
    let init_req = FB::get_root::<DTAPI::InitReq>(in_bytes);

    let mut init_vals_sim = DtasmVarValues::new();

    let init_vals = init_req.init_values().unwrap();
    init_vals.bool_vals().map(|bools| for boolean in bools.iter() {
        let id = boolean.id();
        let val = boolean.val();
        init_vals_sim.bool_values.insert(id, val);
    });
    init_vals.real_vals().map(|reals| for real in reals.iter() {
        let id = real.id();
        let val = real.val();
        init_vals_sim.real_values.insert(id, val);
    });
    init_vals.int_vals().map(|integers| for integer in integers.iter() {
        let id = integer.id();
        let val = integer.val();
        init_vals_sim.int_values.insert(id, val);
    });
    init_vals.string_vals().map(|strings| for string in strings.iter() {
        let id = string.id();
        let val = string.val().unwrap().to_string();
        init_vals_sim.string_values.insert(id, val);
    });

    let md_bytes = MDBYTES.lock().unwrap();
    let md_dtasm = DTMD::get_root_as_model_description(*md_bytes);
    let md = convert_model_description(&md_dtasm);
    
    let init_res =
    unsafe { 
         SIM_MODULE.initialize(&md,
             &init_vals_sim, 
            init_req.starttime(), 
            match init_req.endtime_set() {
                true => Some(init_req.endtime()),
                false => None
            }, 
            match init_req.tolerance_set() {
                true => Some(init_req.tolerance()),
                false => None
            },
            init_req.loglevel_limit().into(), 
            init_req.check_consistency()
        )
    };

    let ret_val: u32;
    let mut fb_builder = FBBUILDER.lock().unwrap();
    {
        let status_res = DTAPI::StatusRes::create(&mut fb_builder, &DTAPI::StatusResArgs{
            status: match init_res {
                Err(_err) => DTT::Status::Error, 
                Ok(status) => status.into()
            }
        });

        fb_builder.finish(status_res, None);
        let buf = fb_builder.finished_data(); 

        let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

        if buf.len() <= out_max_len as usize {
            bytes[..buf.len()].copy_from_slice(buf);
        }

        ret_val = buf.len() as u32;
    }

    fb_builder.reset();
    ret_val
}

#[no_mangle]
extern "C" fn getValues(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{    
    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };
    let getvalues_req = FB::get_root::<DTAPI::GetValuesReq>(in_bytes);

    let get_ids = getvalues_req.ids().expect("Get values request did not contain any variables.");
    let mut get_var_ids: Vec<i32> = vec!();

    for i in 0..get_ids.len() {
        get_var_ids.push(get_ids.get(i));
    }

    let get_values_res =
        unsafe { 
            SIM_MODULE.get_values(&get_var_ids).expect("Error on calling get_values: ")
        };

    let ret_val: u32;
    let mut fb_builder = FBBUILDER.lock().unwrap();
    {
        let mut real_offs: Vec<flatbuffers::WIPOffset<DTT::RealVal>> = Vec::new();
        let real_vals = get_values_res.values.real_values;
        for (key, value) in real_vals {
            let real_val = DTT::RealVal::create(&mut fb_builder, &DTT::RealValArgs{
                id: key,
                val: value
            });
            real_offs.push(real_val);
        }
        let real_vals_fb = fb_builder.create_vector(&real_offs);

        let mut int_offs: Vec<flatbuffers::WIPOffset<DTT::IntVal>> = Vec::new();
        let int_vals = get_values_res.values.int_values;
        for (key, value) in int_vals {
            let int_val = DTT::IntVal::create(&mut fb_builder, &DTT::IntValArgs{
                id: key,
                val: value
            });
            int_offs.push(int_val);
        }
        let int_vals_fb = fb_builder.create_vector(&int_offs);

        let mut bool_offs: Vec<flatbuffers::WIPOffset<DTT::BoolVal>> = Vec::new();
        let bool_vals = get_values_res.values.bool_values;
        for (key, value) in bool_vals {
            let bool_val = DTT::BoolVal::create(&mut fb_builder, &DTT::BoolValArgs{
                id: key,
                val: value
            });
            bool_offs.push(bool_val);
        }
        let bool_vals_fb = fb_builder.create_vector(&bool_offs);

        let mut string_offs: Vec<flatbuffers::WIPOffset<DTT::StringVal>> = Vec::new();
        let str_vals = get_values_res.values.string_values;
        for (id, val) in str_vals {
            let val_str = fb_builder.create_string(&val);
            let str_val = DTT::StringVal::create(&mut fb_builder, &DTT::StringValArgs{
                id: id,
                val: Some(val_str)
            });
            string_offs.push(str_val);
        }
        let str_vals_fb = fb_builder.create_vector(&string_offs);

        let scalar_vals = DTT::VarValues::create(&mut fb_builder, &DTT::VarValuesArgs{
            real_vals: Some(real_vals_fb), 
            int_vals: Some(int_vals_fb),
            bool_vals: Some(bool_vals_fb),
            string_vals: Some(str_vals_fb)
        });

        let get_values_res_fb = DTAPI::GetValuesRes::create(&mut fb_builder, &DTAPI::GetValuesResArgs{
            current_time: get_values_res.current_time,
            values: Some(scalar_vals), 
            status: get_values_res.status.into()
        });

        fb_builder.finish(get_values_res_fb, None);
        let buf = fb_builder.finished_data(); 

        let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

        if buf.len() <= out_max_len as usize {
            bytes[..buf.len()].copy_from_slice(buf);
        }

        ret_val = buf.len() as u32;
    }

    fb_builder.reset();
    ret_val
}

#[no_mangle]
extern "C" fn setValues(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{
    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };
    let set_req = FB::get_root::<DTAPI::SetValuesReq>(in_bytes);

    let mut set_vals_sim = DtasmVarValues::new();

    let set_vals = set_req.values().unwrap();
    set_vals.bool_vals().map(|bools| for boolean in bools.iter() {
        let id = boolean.id();
        let val = boolean.val();
        set_vals_sim.bool_values.insert(id, val);
    });
    set_vals.real_vals().map(|reals| for real in reals.iter() {
        let id = real.id();
        let val = real.val();
        set_vals_sim.real_values.insert(id, val);
    });
    set_vals.int_vals().map(|integers| for integer in integers.iter() {
        let id = integer.id();
        let val = integer.val();
        set_vals_sim.int_values.insert(id, val);
    });
    set_vals.string_vals().map(|strings| for string in strings.iter() {
        let id = string.id();
        let val = string.val().unwrap().to_string();
        set_vals_sim.string_values.insert(id, val);
    });

    let set_vals_res =
    unsafe { 
         SIM_MODULE.set_values(&set_vals_sim)
    };

    let ret_val: u32;
    let mut fb_builder = FBBUILDER.lock().unwrap();
    {
        let status_res = DTAPI::StatusRes::create(&mut fb_builder, &DTAPI::StatusResArgs{
            status: match set_vals_res {
                Err(_err) => DTT::Status::Error, 
                Ok(status) => status.into()
            }
        });

        fb_builder.finish(status_res, None);
        let buf = fb_builder.finished_data(); 

        let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

        if buf.len() <= out_max_len as usize {
            bytes[..buf.len()].copy_from_slice(buf);
        }

        ret_val = buf.len() as u32;
    }

    fb_builder.reset();
    ret_val
}

#[no_mangle]
extern "C" fn doStep(in_p: *const u8, in_len: u32, out_p: *mut u8, out_max_len: u32) -> u32
{
    let in_bytes = unsafe { slice::from_raw_parts(in_p, in_len as usize) };
    let dostep_req = FB::get_root::<DTAPI::DoStepReq>(in_bytes);

    let current_time = dostep_req.current_time();
    let step = dostep_req.timestep();

    let do_step_res =
    unsafe { 
        SIM_MODULE.do_step(current_time, step).expect("Error on calling do_step: ")
    };

    let ret_val: u32;
    let mut fb_builder = FBBUILDER.lock().unwrap();
    {
        let do_step_res_fb = DTAPI::DoStepRes::create(&mut fb_builder, &DTAPI::DoStepResArgs{
            status: do_step_res.status.into(),
            updated_time: do_step_res.updated_time
        });

        fb_builder.finish(do_step_res_fb, None);
        let buf = fb_builder.finished_data(); 

        let bytes = unsafe { slice::from_raw_parts_mut(out_p, out_max_len as usize) };

        if buf.len() <= out_max_len as usize {
            bytes[..buf.len()].copy_from_slice(buf);
        }

        ret_val = buf.len() as u32;
    }

    fb_builder.reset();
    ret_val
}
