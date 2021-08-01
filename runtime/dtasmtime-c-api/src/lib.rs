// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use std::{ffi::{CStr, CString}, iter::FromIterator};
use std::{mem, mem::ManuallyDrop};
use std::ptr;
use std::path::PathBuf;
use std::collections::HashMap;

use libc::{c_char};

use dtasmtime::{model_description as MD, runtime::{Engine, Module, Instance}};
use dtasm_base::types::{DtasmVarValues as VarValues};

pub mod c_types;
use c_types::*;


#[no_mangle]
pub extern "C" fn dtasmtime_engine_new() -> *mut Engine {
    let engine = Engine::new().expect("Could not create dtasmtime engine");

    Box::into_raw(Box::new(engine))
}

#[no_mangle]
pub extern "C" fn dtasmtime_engine_free(ptr: *mut Engine) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn dtasmtime_module_new(filepath: *const c_char, 
    eng_ptr: *mut Engine) -> *mut Module<'static> {
    
    let engine = unsafe {
        assert!(!eng_ptr.is_null(), "Invalid module received");
        &mut *eng_ptr
    };

    let c_path = unsafe {
        assert!(!filepath.is_null());
        CStr::from_ptr(filepath)
    };

    let path = c_path.to_str().unwrap();
    let pathbuf = PathBuf::from(path);

    let module = Module::new(pathbuf, engine).expect("Could not load dtasm module");
    Box::into_raw(Box::new(module))
}

#[no_mangle]
pub extern "C" fn dtasmtime_module_free(ptr: *mut Module) {
    if ptr.is_null() {
        return
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn dtasmtime_module_instantiate(mod_ptr: *mut Module) -> *mut Instance {
    let module = unsafe {
        assert!(!mod_ptr.is_null(), "Invalid module received");
        &mut *mod_ptr
    };

    let inst = module.instantiate().expect("Could not instantiate module");

    Box::into_raw(Box::new(inst))
}

#[no_mangle]
pub extern "C" fn dtasmtime_instance_free(inst_ptr: *mut Instance) {
    if inst_ptr.is_null() {
        return
    }
    unsafe {
        Box::from_raw(inst_ptr);
    }
}

#[no_mangle]
pub extern "C" fn dtasmtime_modeldescription_get(inst_ptr: *mut Instance) -> DtasmModelDescription {
    let inst = unsafe {
        assert!(!inst_ptr.is_null(), "Invalid instance received");
        &mut *inst_ptr
    };

    let md = inst.get_model_description().expect("Error reading model description");

    let exp_info =
        if let Some(ei) = &md.experiment {
            DtasmExperimentInfo{
                time_unit: CString::new(ei.time_unit.to_string()).unwrap().into_raw(), 
                end_time_default: ei.end_time_default, 
                start_time_default: ei.start_time_default, 
                time_step_default: ei.time_step_default, 
                time_step_max: ei.time_step_max, 
                time_step_min: ei.time_step_min
            }
        }
        else {
            Default::default()
        };

    let model_info = DtasmModelInfo{
            description: CString::new(md.model.description.to_string()).unwrap().into_raw(), 
            generation_date_time: CString::new(md.model.generation_date_time.to_string()).unwrap().into_raw(), 
            generation_tool: CString::new(md.model.generation_tool.to_string()).unwrap().into_raw(), 
            id: CString::new(md.model.id.to_string()).unwrap().into_raw(), 
            name: CString::new(md.model.name.to_string()).unwrap().into_raw(),
            name_delimiter: CString::new(md.model.name_delimiter.to_string()).unwrap().into_raw(), 
            capabilities: DtasmCapabilities {
                can_handle_variable_step_size: md.model.capabilities.can_handle_variable_step_size, 
                can_interpolate_inputs: md.model.capabilities.can_interpolate_inputs, 
                can_reset_step: md.model.capabilities.can_reset_step
            }
        };

    let n_vars = md.variables.len();

    let vars = get_model_variables_raw(&md);

    DtasmModelDescription {
        experiment: exp_info, 
        has_experiment: md.experiment.is_some(),
        model: model_info, 
        n_variables: n_vars, 
        variables: vars
    }
}

#[no_mangle]
pub extern "C" fn dtasmtime_modeldescription_free(md: DtasmModelDescription) {
    if md.has_experiment {
        unsafe {
            CString::from_raw(md.experiment.time_unit);
        }
    }

    unsafe {
        CString::from_raw(md.model.description);
        CString::from_raw(md.model.generation_date_time);
        CString::from_raw(md.model.generation_tool);
        CString::from_raw(md.model.id);
        CString::from_raw(md.model.name);
        CString::from_raw(md.model.name_delimiter);
    }

    let variables: Vec<DtasmModelVariable> = unsafe {
        Vec::from_raw_parts(md.variables, md.n_variables, md.n_variables)
    };

    for var in variables {
        unsafe {
            CString::from_raw(var.description);
            CString::from_raw(var.name);
            CString::from_raw(var.unit);
        }

        if var.has_default {
            unsafe {
                CString::from_raw(var.default.string_val);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn dtasmtime_initialize(inst_ptr: *mut Instance, initial_vals: DtasmVarValues, tmin: f64, tmax_set: bool, tmax: f64, 
    tol_set: bool, tol: f64, log_level: DtasmLogLevel, check: bool) -> DtasmStatus {
    let inst = unsafe {
        assert!(!inst_ptr.is_null(), "Invalid instance received");
        &mut *inst_ptr
    };

    let init_vals = cvarvalues_to_dtasmvarvalues(initial_vals);

    let tmax_opt = if tmax_set { Some(tmax) } else { None };
    let tol_opt = if tol_set { Some(tol) } else { None };

    inst.initialize(&init_vals, tmin, tmax_opt, tol_opt, log_level.into(), check).expect("initialize failed").into()
}

#[no_mangle]
pub extern "C" fn dtasmtime_setvalues(inst_ptr: *mut Instance, set_vals: DtasmVarValues) -> DtasmStatus {
    let set_values = cvarvalues_to_dtasmvarvalues(set_vals);

    let inst = unsafe {
        assert!(!inst_ptr.is_null(), "Invalid instance received");
        &mut *inst_ptr
    };

    inst.set_values(&set_values).expect("Set values failed").into()
}

#[no_mangle]
pub extern "C" fn dtasmtime_getvalues(inst_ptr: *mut Instance, var_ids: *mut i32, var_count: i32) -> DtasmGetValuesResponse {
    let inst = unsafe {
        assert!(!inst_ptr.is_null(), "Invalid instance received");
        &mut *inst_ptr
    };

    // clone pointed-to array in such a way that it's not freed when this function returns
    let ids = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(var_ids, var_count as usize, var_count as usize)
    }).clone());

    let res = inst.get_values(&ids).expect("Error on invoking instance's get_values");

    DtasmGetValuesResponse {
        current_time: res.current_time, 
        status: res.status.into(),
        values: get_var_values_raw(&res.values)
    }
}

#[no_mangle]
pub extern "C" fn dtasmtime_getvalues_free(get_values: DtasmGetValuesResponse) {
    drop(unsafe {
        Vec::from_raw_parts(get_values.values.real_ids, get_values.values.n_reals as usize, get_values.values.n_reals as usize)
    });
    drop(unsafe {
        Vec::from_raw_parts(get_values.values.real_values, get_values.values.n_reals as usize, get_values.values.n_reals as usize)
    });
    drop(unsafe {
        Vec::from_raw_parts(get_values.values.int_ids, get_values.values.n_ints as usize, get_values.values.n_ints as usize)
    });
    drop(unsafe {
        Vec::from_raw_parts(get_values.values.int_values, get_values.values.n_ints as usize, get_values.values.n_ints as usize)
    });
    drop(unsafe {
        Vec::from_raw_parts(get_values.values.bool_ids, get_values.values.n_bools as usize, get_values.values.n_bools as usize)
    });
    drop(unsafe {
        Vec::from_raw_parts(get_values.values.bool_values, get_values.values.n_bools as usize, get_values.values.n_bools as usize)
    });

    // TODO: string variables
}

#[no_mangle]
pub extern "C" fn dtasmtime_dostep(inst_ptr: *mut Instance, t: f64, dt: f64) -> DtasmDoStepResponse {
    let inst = unsafe {
        assert!(!inst_ptr.is_null(), "Invalid instance received");
        &mut *inst_ptr
    };

    let dostep_res = inst.do_step(t,dt).expect("Error on invoking instance's gedo_stept_values");
    DtasmDoStepResponse {
        status: dostep_res.status.into(),
        updated_time: dostep_res.updated_time
    }
}

fn cvarvalues_to_dtasmvarvalues(vals: DtasmVarValues) -> VarValues {
    // clone pointed-to arrays in such a way that they're not freed when this function returns
    let real_ids = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.real_ids, vals.n_reals as usize, vals.n_reals as usize)
    }).clone());

    let real_vals = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.real_values, vals.n_reals as usize, vals.n_reals as usize)
    }).clone());

    let real_varvals: HashMap<i32, f64> = real_ids.into_iter().zip(real_vals.into_iter()).collect();

    let int_ids = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.int_ids, vals.n_ints as usize, vals.n_ints as usize)
    }).clone());

    let int_vals = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.int_values, vals.n_ints as usize, vals.n_ints as usize)
    }).clone());

    let int_varvals: HashMap<i32, i32> = int_ids.into_iter().zip(int_vals.into_iter()).collect();

    let bool_ids = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.bool_ids, vals.n_bools as usize, vals.n_bools as usize)
    }).clone());

    let bool_vals = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.bool_values, vals.n_bools as usize, vals.n_bools as usize)
    }).clone());

    let bool_varvals: HashMap<i32, bool> = bool_ids.into_iter().zip(bool_vals.into_iter()).collect();

    let string_ids = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.string_ids, vals.n_strings as usize, vals.n_strings as usize)
    }).clone());

    let c_string_vals = ManuallyDrop::into_inner(ManuallyDrop::new(unsafe {
        Vec::from_raw_parts(vals.string_values, vals.n_strings as usize, vals.n_strings as usize)
    }).clone());
    
    let string_vals: Vec<String> = c_string_vals.iter().map(|&x| unsafe { CStr::from_ptr(x).to_str()
        .expect("String conversion error").to_string() }).collect();

    let string_varvals: HashMap<i32, String> = string_ids.into_iter().zip(string_vals.into_iter()).collect();

    VarValues {
        real_values: real_varvals,
        int_values: int_varvals,
        bool_values: bool_varvals,
        string_values: string_varvals,
    }
}


fn get_var_values_raw(vals: &VarValues) -> DtasmVarValues {
    let mut real_ids = ManuallyDrop::new(Vec::from_iter(vals.real_values.keys().cloned()).into_boxed_slice());
    let mut real_values = ManuallyDrop::new(Vec::from_iter(vals.real_values.values().cloned()).into_boxed_slice());
    let mut int_ids = ManuallyDrop::new(Vec::from_iter(vals.int_values.keys().cloned()).into_boxed_slice());
    let mut int_values = ManuallyDrop::new(Vec::from_iter(vals.int_values.values().cloned()).into_boxed_slice());
    let mut bool_ids = ManuallyDrop::new(Vec::from_iter(vals.bool_values.keys().cloned()).into_boxed_slice());
    let mut bool_values = ManuallyDrop::new(Vec::from_iter(vals.bool_values.values().cloned()).into_boxed_slice());

    DtasmVarValues {
        real_ids: real_ids.as_mut_ptr(),
        real_values: real_values.as_mut_ptr(),
        int_ids: int_ids.as_mut_ptr(),
        int_values: int_values.as_mut_ptr(),
        bool_ids: bool_ids.as_mut_ptr(),
        bool_values: bool_values.as_mut_ptr(),
        // TODO: strings are more complicated
        string_ids: ptr::null_mut(), 
        string_values: ptr::null_mut(), 
        n_reals: real_values.len() as i32,
        n_bools: bool_values.len() as i32,
        n_ints: int_values.len() as i32,
        n_strings: 0
    }
}


fn get_model_variables_raw(md: &MD::ModelDescription) -> *mut DtasmModelVariable {
    let mut vec_vars: Vec<DtasmModelVariable> = Vec::new();
    for var in &md.variables {
        let c_var_default = 
            if let Some(def) = &var.default {
                DtasmVariableValue {
                    bool_val: def.bool_val, 
                    real_val: def.real_val, 
                    int_val: def.int_val, 
                    string_val: CString::new(def.string_val.to_string()).unwrap().into_raw()
                }
            }
            else {
                DtasmVariableValue::default()
            };

        let c_var = DtasmModelVariable {
            causality: var.causality.into(), 
            derivative_of_id: var.derivative_of_id, 
            description: CString::new(var.description.to_string()).unwrap().into_raw(), 
            id: var.id, 
            name: CString::new(var.name.to_string()).unwrap().into_raw(), 
            unit: CString::new(var.unit.to_string()).unwrap().into_raw(), 
            value_type: var.value_type.into(), 
            default: c_var_default,
            has_default: var.default.is_some()
        };

        vec_vars.push(c_var);
    }

    vec_vars.shrink_to_fit();
    assert!(vec_vars.len() == vec_vars.capacity());
    let ptr = vec_vars.as_mut_ptr();
    mem::forget(vec_vars);

    ptr
}
