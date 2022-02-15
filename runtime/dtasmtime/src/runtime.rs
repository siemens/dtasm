// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use std::convert::identity; 
use std::error::Error;
use std::path::PathBuf;
use std::io::{Read, Write};
use std::collections::HashMap;

use flatbuffers as FB;
use wasmtime as WT;
use wasmtime_wasi as WTW;

use dtasm_abi::dtasm_generated::dtasm_api as DTAPI;
use dtasm_abi::dtasm_generated::dtasm_types as DTT;
use dtasm_abi::dtasm_generated::dtasm_model_description as DTMD;

use crate::errors::DtasmtimeError;
use DtasmtimeError::DtasmError as DTERR; 
use dtasm_base::model_conversion::convert_model_description;
use dtasm_base::model_description as MD;
use dtasm_base::types::{DtasmVarType,DtasmVarValues,LogLevel,Status,GetValuesResponse,DoStepResponse};
use dtasm_base::errors::DtasmError;

type In1Out1T = WT::TypedFunc<i32,i32>;
type In1Out0T = WT::TypedFunc<i32,()>;
type In2Out1T = WT::TypedFunc<(i32,i32),i32>;
type In4Out1T = WT::TypedFunc<(i32,i32,i32,i32),i32>;

const WASM_PAGE_SIZE: u64 = 65536;
const FB_BUILDER_SIZE: usize = 32768;
const BASE_MEM_SIZE: i32 = 2048;

/// dtasm interface functions
static DTASM_EXPORTS: [&str; 8] = [
    "memory", 
    "alloc", 
    "dealloc", 
    "getModelDescription", 
    "init", 
    "getValues", 
    "setValues",
    "doStep"];

/// Engine for executing modules
pub struct Engine {
    wt_engine: WT::Engine, 
    wt_linker: WT::Linker<WTW::WasiCtx>,
}

impl Engine {
    pub fn new() -> Result<Engine, Box<dyn Error>> {
        let engine = WT::Engine::default();
        let mut linker = WT::Linker::new(&engine);
        WTW::add_to_linker(&mut linker, |s| s)?;

        Ok(Engine {
            wt_engine: engine,
            wt_linker: linker
        })
    }
}

/// Represents a dtasm module in memory
pub struct Module<'a> {
    wt_module: WT::Module,
    dtasm_engine: &'a Engine
}

impl Module<'_> {
    /// Loads a module from bytestream; note that the module needs to be tied to an engine at this point
    pub fn new(file: PathBuf, engine: &Engine) -> Result<Module, DtasmtimeError> {
        let module = WT::Module::from_file(&engine.wt_engine, file)?;

        for name in DTASM_EXPORTS.iter() {
            if module.get_export(name).is_none() {
                return Err(DTERR(DtasmError::MissingDtasmExport(name.to_string())));
            }
        }

        // TODO: ensure that exports have expected signature

        Ok(Module {
            wt_module: module, 
            dtasm_engine: engine
        })
    }

    /// Create an instance of the module
    pub fn instantiate(&mut self) -> Result<Instance, DtasmtimeError> {
        let wasi = WTW::WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        let mut store = WT::Store::new(&self.dtasm_engine.wt_engine, wasi);
        let wt_instance = self.dtasm_engine.wt_linker.instantiate(&mut store, &self.wt_module)?;

        let reactor_init = wt_instance
            .get_func(&mut store, "_initialize");
        let memory = wt_instance
            .get_memory(&mut store, "memory")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("memory".to_string())))?;
        let alloc = wt_instance
            .get_func(&mut store, "alloc")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("alloc".to_string())))?
            .typed::<i32,i32,_>(&mut store)?;
        let dealloc = wt_instance
            .get_func(&mut store, "dealloc")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("dealloc".to_string())))?
            .typed::<i32,(),_>(&store)?;
        let get_model_description = wt_instance
            .get_func(&mut store, "getModelDescription")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("getModelDescription".to_string())))?
            .typed::<(i32,i32),i32,_>(&store)?;
        let init = wt_instance
            .get_func(&mut store, "init")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("init".to_string())))?
            .typed::<(i32,i32,i32,i32),i32,_>(&store)?;
        let get_values = wt_instance
            .get_func(&mut store, "getValues")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("getValues".to_string())))?
            .typed::<(i32,i32,i32,i32),i32,_>(&store)?;
        let set_values = wt_instance
            .get_func(&mut store, "setValues")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("setValues".to_string())))?
            .typed::<(i32,i32,i32,i32),i32,_>(&store)?;
        let do_step = wt_instance
            .get_func(&mut store, "doStep")
            .ok_or(DTERR(DtasmError::MissingDtasmExport("doStep".to_string())))?
            .typed::<(i32,i32,i32,i32),i32,_>(&store)?;

        Ok(Instance {
            memory, 
            store: store,
            reactor_init_fn: reactor_init,
            alloc_fn: alloc, 
            dealloc_fn: dealloc, 
            get_md_fn: get_model_description, 
            init_fn: init,
            get_values_fn: get_values,
            set_values_fn: set_values,
            do_step_fn: do_step,
            var_types: HashMap::new(),
            md: None, 
            builder: FB::FlatBufferBuilder::with_capacity(FB_BUILDER_SIZE)
        })
    }
}

/// Represents an instance of a loaded dtasm module
pub struct Instance {
    memory: WT::Memory, 
    store: WT::Store<WTW::WasiCtx>,
    reactor_init_fn: Option<WT::Func>,
    alloc_fn: In1Out1T, 
    dealloc_fn: In1Out0T, 
    get_md_fn: In2Out1T, 
    init_fn: In4Out1T,
    get_values_fn: In4Out1T,
    do_step_fn: In4Out1T,
    set_values_fn: In4Out1T,
    var_types: HashMap<i32, DtasmVarType>,
    md: Option<MD::ModelDescription>, 
    builder: FB::FlatBufferBuilder<'static>
}

impl Instance {
    /// Retrieve the model description of this module by calling the `getModelDescription` 
    /// export
    pub fn get_model_description(&mut self) -> Result<MD::ModelDescription, DtasmtimeError> {

        // if model description was already loaded, return it from cache
        match &self.md {
            None => {}, 
            Some(mod_desc) => {
                return Ok(mod_desc.clone());
            }
        } 

        let mut size = BASE_MEM_SIZE;
        let mut mem = self.alloc_fn.call(&mut self.store, size)?;
        let mut size_out = self.get_md_fn.call(&mut self.store, (mem, size))?;

        while size_out > size {
            self.dealloc_fn.call(&mut self.store,mem)?;
            size *= 2;
            mem = self.alloc_fn.call(&mut self.store, size)?;

            size_out = self.get_md_fn.call(&mut self.store, (mem, size))?;
        }

        let bytes = &self.memory.data(&mut self.store)[mem as usize..(mem+size_out) as usize];
   
        let model_desc_fb = DTMD::root_as_model_description(bytes).unwrap();
        let md = convert_model_description(&model_desc_fb);
        self.md = Some(md.clone());
        self.var_types = Instance::collect_var_types(&md)?;

        self.dealloc_fn.call(&mut self.store, mem)?;
   
        Ok(md)
    }

    /// Initialize the instance with the given initial values and simulation parameters
    ///
    /// * `initial_vals` - initial values for the state variables
    /// * `tmin` - initial time where simulation starts
    /// * `tmax` - final time of the simulation
    /// * `tol` - relative tolerance for numerical solver
    /// * `log_level` - maximal level at which log messages should be reported
    /// * `check` - whether to check validity of buffers (not currently implemented)
    pub fn initialize(&mut self, initial_vals: &DtasmVarValues, tmin: f64, tmax: Option<f64>, 
        tol: Option<f64>, log_level: LogLevel, check: bool) -> Result<Status, DtasmtimeError>{
        // TODO: Check if state valid

        let md = &self.md.as_ref().ok_or(DTERR(DtasmError::InvalidCallingOrder))?;
        
        // if _initialize is exported, call it now to initialize WASI reactor
        match &self.reactor_init_fn {
            None => (),
            Some(f) => {
                f.call(&mut self.store, &[], &mut [])?;
            }
        }
        
        let fb_log = match log_level {
            LogLevel::Info => DTT::LogLevel::Info,
            LogLevel::Warn => DTT::LogLevel::Warn,
            LogLevel::Error => DTT::LogLevel::Error,
        };

        let mut var_values = DtasmVarValues::new();

        // collect all initial values that are explicitly set and check their types
        for (id, val) in &initial_vals.real_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmReal { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.real_values.insert(*id, *val);
        }
        for (id, val) in &initial_vals.int_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmInt { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.int_values.insert(*id, *val);
        }
        for (id, val) in &initial_vals.bool_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmBool { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.bool_values.insert(*id, *val);
        }
        for (id, val) in &initial_vals.string_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmString { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.string_values.insert(*id, val.clone());
        }

        // build up the init request message
        let model_id = self.builder.create_string(&md.model.id);

        let mut real_offs: Vec<flatbuffers::WIPOffset<DTT::RealVal>> = Vec::new();
        for (id, val) in &var_values.real_values {
            real_offs.push(DTT::RealVal::create(&mut self.builder, &DTT::RealValArgs{
                id: *id,
                val: *val
            }));
        }
        let real_vals = self.builder.create_vector(&real_offs);

        let mut int_offs: Vec<flatbuffers::WIPOffset<DTT::IntVal>> = Vec::new();
        for (id, val) in &var_values.int_values {
            int_offs.push(DTT::IntVal::create(&mut self.builder, &DTT::IntValArgs{
                id: *id,
                val: *val
            }));
        }
        let int_vals = self.builder.create_vector(&int_offs);

        let mut bool_offs: Vec<flatbuffers::WIPOffset<DTT::BoolVal>> = Vec::new();
        for (id, val) in &var_values.bool_values {
            bool_offs.push(DTT::BoolVal::create(&mut self.builder, &DTT::BoolValArgs{
                id: *id,
                val: *val
            }));
        }
        let bool_vals = self.builder.create_vector(&bool_offs);

        let mut string_offs: Vec<flatbuffers::WIPOffset<DTT::StringVal>> = Vec::new();
        for (id, val) in &var_values.string_values {
            let val_str = self.builder.create_string(val);
            string_offs.push(DTT::StringVal::create(&mut self.builder, &DTT::StringValArgs{
                id: *id,
                val: Some(val_str)
            }));
        }
        let string_vals = self.builder.create_vector(&string_offs);

        let scalar_vals = DTT::VarValues::create(&mut self.builder, &DTT::VarValuesArgs{
            real_vals: Some(real_vals), 
            int_vals: Some(int_vals),
            bool_vals: Some(bool_vals),
            string_vals: Some(string_vals)
        });

        let init_req = DTAPI::InitReq::create(&mut self.builder, &DTAPI::InitReqArgs{
            id: Some(model_id), 
            starttime: tmin, 
            endtime: match tmax {Some(v) => v, None => 0.0}, 
            endtime_set: match tmax {Some(_v) => true, None => false}, 
            tolerance: match tol {Some(v) => v, None => 0.0}, 
            tolerance_set: match tol {Some(_v) => true, None => false}, 
            loglevel_limit: fb_log, 
            check_consistency: check, 
            init_values: Some(scalar_vals)
        });
        self.builder.finish(init_req, None);

        let init_req_buf = self.builder.finished_data(); 
        let init_req_len = init_req_buf.len();
        let init_req_ptr = self.alloc_fn.call(&mut self.store, init_req_len as i32)? as usize;

        // copy buffer into allocated position in linear memory
        self.memory.data_mut(&mut self.store)[init_req_ptr..init_req_ptr+init_req_len]
            .copy_from_slice(init_req_buf);

        // return value is status only, should fit into 64 bytes
        let size = 64;
        let init_res_ptr = self.alloc_fn.call(&mut self.store, size)? as usize;
        let size_out = self.init_fn.call(&mut self.store, (init_req_ptr as i32, init_req_len as i32, init_res_ptr as i32, size))?;

        if size_out > size { return Err(DTERR(DtasmError::DtasmInternalError(format!("Unexpected size returned from init request: {}", size_out)))); }

        let res_bytes = &self.memory.data(&mut self.store)[init_res_ptr..init_res_ptr+(size_out as usize)];

        let init_res = FB::root::<DTAPI::StatusRes>(res_bytes).unwrap();

        let status_res = init_res.status().into();
        
        self.dealloc_fn.call(&mut self.store, init_req_ptr as i32)?;
        self.dealloc_fn.call(&mut self.store, init_res_ptr as i32)?;
        self.builder.reset();

        Ok(status_res)
    }

    /// Retrieve values of the output and state variables in the current timestep. 
    /// 
    /// * `var_ids` - vector of variable ids for which values shall be retrieved
    pub fn get_values(&mut self, var_ids: &Vec<i32>) -> Result<GetValuesResponse, DtasmtimeError> {
        // TODO: Check state

        // check if all requested var ids are valid
        for id in var_ids.iter() {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].causality == MD::CausalityType::Input {
                return Err(DTERR(DtasmError::VariableCausalityMismatch(MD::CausalityType::Input,*id))); 
            }
        }

        // build get_values request message
        let var_ids_fb = self.builder.create_vector(&var_ids);

        let req = DTAPI::GetValuesReq::create(&mut self.builder, &DTAPI::GetValuesReqArgs{
            ids: Some(var_ids_fb)
        });
        self.builder.finish(req, None);

        let getval_req_buf = self.builder.finished_data();
        let getval_req_len = getval_req_buf.len();
        let getval_req_ptr = self.alloc_fn.call(&mut self.store, getval_req_len as i32)? as usize;

        self.memory.data_mut(&mut self.store)[getval_req_ptr..getval_req_ptr+getval_req_len]
            .copy_from_slice(getval_req_buf);
    
        let mut size = BASE_MEM_SIZE;
        let mut getval_res_ptr = self.alloc_fn.call(&mut self.store,  size)? as usize;
        let mut size_out = self.get_values_fn.call(&mut self.store, (getval_req_ptr as i32, getval_req_len as i32, getval_res_ptr as i32, size))?;
        
        while size_out > size {
            self.dealloc_fn.call(&mut self.store, getval_res_ptr as i32)?;
            size *= 2;
            getval_res_ptr = self.alloc_fn.call(&mut self.store, size)? as usize;
    
            size_out = self.get_values_fn.call(&mut self.store, (getval_req_ptr as i32, getval_req_len as i32, getval_res_ptr as i32, size))?;
        }
    
        let res_bytes = &self.memory.data(&mut self.store)[getval_res_ptr..getval_res_ptr+size_out as usize];
    
        let getvalues_res = FB::root::<DTAPI::GetValuesRes>(res_bytes).unwrap();
        let var_values = Instance::extract_vals(&getvalues_res, &self.var_types)?;
        let current_time = getvalues_res.current_time();
        let status = getvalues_res.status().into();

        self.dealloc_fn.call(&mut self.store, getval_req_ptr as i32)?;
        self.dealloc_fn.call(&mut self.store, getval_res_ptr as i32)?;
        self.builder.reset();

        Ok(GetValuesResponse {status, current_time, values: var_values})
    }


    /// Set values of input variables for the next timestep
    ///
    /// * `input_vals`: Values for the input variables
    pub fn set_values(&mut self, input_vals: &DtasmVarValues) -> Result<Status, DtasmtimeError>{
        // TODO: check state

        // start with default values from model description
        let mut var_values = DtasmVarValues::new();

        // collect set values and check their existence and types
        for (id, val) in &input_vals.real_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].causality != MD::CausalityType::Input { 
                return Err(DTERR(DtasmError::VariableCausalityInvalidForSet(self.var_types[id].causality, *id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmReal { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.real_values.insert(*id, *val);
        }
        for (id, val) in &input_vals.int_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].causality != MD::CausalityType::Input { 
                return Err(DTERR(DtasmError::VariableCausalityInvalidForSet(self.var_types[id].causality, *id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmInt { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.int_values.insert(*id, *val);
        }
        for (id, val) in &input_vals.bool_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].causality != MD::CausalityType::Input { 
                return Err(DTERR(DtasmError::VariableCausalityInvalidForSet(self.var_types[id].causality, *id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmBool { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.bool_values.insert(*id, *val);
        }
        for (id, val) in &input_vals.string_values {
            if !self.var_types.contains_key(id) { 
                return Err(DTERR(DtasmError::UnknownVariableId(*id))); 
            }
            if self.var_types[id].causality != MD::CausalityType::Input { 
                return Err(DTERR(DtasmError::VariableCausalityInvalidForSet(self.var_types[id].causality, *id))); 
            }
            if self.var_types[id].value_type != MD::VariableType::DtasmString { 
                return Err(DTERR(DtasmError::VariableTypeMismatch(self.var_types[id].value_type, *id))); 
            }
            var_values.string_values.insert(*id, val.clone());
        }

        // build the setValues request message
        let mut real_offs: Vec<flatbuffers::WIPOffset<DTT::RealVal>> = Vec::new();
        for (id, val) in &var_values.real_values {
            real_offs.push(DTT::RealVal::create(&mut self.builder, &DTT::RealValArgs{
                id: *id,
                val: *val
            }));
        }
        let real_vals = self.builder.create_vector(&real_offs);

        let mut int_offs: Vec<flatbuffers::WIPOffset<DTT::IntVal>> = Vec::new();
        for (id, val) in &var_values.int_values {
            int_offs.push(DTT::IntVal::create(&mut self.builder, &DTT::IntValArgs{
                id: *id,
                val: *val
            }));
        }
        let int_vals = self.builder.create_vector(&int_offs);

        let mut bool_offs: Vec<flatbuffers::WIPOffset<DTT::BoolVal>> = Vec::new();
        for (id, val) in &var_values.bool_values {
            bool_offs.push(DTT::BoolVal::create(&mut self.builder, &DTT::BoolValArgs{
                id: *id,
                val: *val
            }));
        }
        let bool_vals = self.builder.create_vector(&bool_offs);

        let mut string_offs: Vec<flatbuffers::WIPOffset<DTT::StringVal>> = Vec::new();
        for (id, val) in &var_values.string_values {
            let val_str = self.builder.create_string(val);
            string_offs.push(DTT::StringVal::create(&mut self.builder, &DTT::StringValArgs{
                id: *id,
                val: Some(val_str)
            }));
        }
        let string_vals = self.builder.create_vector(&string_offs);

        let scalar_vals = DTT::VarValues::create(&mut self.builder, &DTT::VarValuesArgs{
            real_vals: Some(real_vals), 
            int_vals: Some(int_vals),
            bool_vals: Some(bool_vals),
            string_vals: Some(string_vals)
        });

        let set_vals_req = DTAPI::SetValuesReq::create(&mut self.builder, &DTAPI::SetValuesReqArgs{
            values: Some(scalar_vals),
        });
        self.builder.finish(set_vals_req, None);

        let set_req_buf = self.builder.finished_data(); 
        let set_req_len = set_req_buf.len();
        let set_req_ptr = self.alloc_fn.call(&mut self.store, set_req_len as i32)? as usize;

        // copy buffer into allocated position in linear memory
        self.memory.data_mut(&mut self.store)[set_req_ptr..set_req_ptr+set_req_len]
                .copy_from_slice(set_req_buf);

        // return value is status only, should fit into 64 bytes
        let size = 64;
        let set_res_ptr = self.alloc_fn.call(&mut self.store, size)? as usize;
        let size_out = self.set_values_fn.call(&mut self.store, (set_req_ptr as i32, set_req_len as i32, set_res_ptr as i32, size))?;

        if size_out > size { return Err(DTERR(DtasmError::DtasmInternalError(format!("Unexpected size returned from setValues request: {}", size_out)))); }

        let res_bytes = &self.memory.data(&mut self.store)[set_res_ptr..set_res_ptr+(size_out as usize)];

        let init_res = FB::root::<DTAPI::StatusRes>(res_bytes).unwrap();

        let status_res = init_res.status().into();
        
        self.dealloc_fn.call(&mut self.store, set_req_ptr as i32)?;
        self.dealloc_fn.call(&mut self.store, set_res_ptr as i32)?;
        self.builder.reset();

        Ok(status_res)
    }

    /// Simulate a time step
    ///
    /// * `current_time` - current time
    /// * `timestep` - step to calculate forward in time
    pub fn do_step(&mut self, current_time: f64, timestep: f64) -> Result<DoStepResponse, DtasmtimeError> {
        // TODO: Check correct state

        // build doStep request message
        let req = DTAPI::DoStepReq::create(&mut self.builder, &DTAPI::DoStepReqArgs{
            current_time: current_time, 
            timestep
        });
        self.builder.finish(req, None);

        let dostep_req_buf = self.builder.finished_data();
        let dostep_req_len = dostep_req_buf.len();
        let dostep_req_ptr = self.alloc_fn.call(&mut self.store, dostep_req_len as i32)? as usize;

        self.memory.data_mut(&mut self.store)[dostep_req_ptr..dostep_req_ptr+dostep_req_len]
            .copy_from_slice(dostep_req_buf);
    
        let size = BASE_MEM_SIZE;
        let dostep_res_ptr = self.alloc_fn.call(&mut self.store, size)? as usize;
        let size_out = self.do_step_fn.call(&mut self.store, (dostep_req_ptr as i32, dostep_req_len as i32, dostep_res_ptr as i32, size))?;
        
        if size_out > size { return Err(DTERR(DtasmError::DtasmInternalError(format!("Unexpected size returned from doStep request: {}", size_out)))); }
    
        let res_bytes = &self.memory.data(&mut self.store)[dostep_res_ptr..dostep_res_ptr+size_out as usize];
    
        let dostep_res = FB::root::<DTAPI::DoStepRes>(res_bytes).unwrap();
        let updated_time = dostep_res.updated_time();
        let status_res = dostep_res.status().into();
     
        self.dealloc_fn.call(&mut self.store, dostep_req_ptr as i32)?;
        self.dealloc_fn.call(&mut self.store, dostep_res_ptr as i32)?;
        self.builder.reset();

        Ok(DoStepResponse {status: status_res, updated_time})
    }

    fn extract_vals(&getvalues_res: &DTAPI::GetValuesRes, 
        map_id_var: &HashMap<i32, DtasmVarType>) -> Result<DtasmVarValues, DtasmError> {

        let mut var_vals = DtasmVarValues::new();
        
        let values = getvalues_res.values()
            .ok_or(DtasmError::DtasmInternalError("Invalid response received to getValues request: `values` field empty".to_string()))?;
        
        for real_val in values.real_vals().iter().flat_map(identity) {
            let id = real_val.id();
            let val = real_val.val();
    
            if !map_id_var.contains_key(&id){
                return Err(DtasmError::UnknownVariableId(id));
            }
            if map_id_var[&id].value_type != MD::VariableType::DtasmReal {
                return Err(DtasmError::VariableTypeMismatch(MD::VariableType::DtasmReal, id));
            }
            var_vals.real_values.insert(id, val);
        }

        for int_val in values.int_vals().iter().flat_map(identity) {
            let id = int_val.id();
            let val = int_val.val();
    
            if !map_id_var.contains_key(&id){
                return Err(DtasmError::UnknownVariableId(id));
            }
            if map_id_var[&id].value_type != MD::VariableType::DtasmInt {
                return Err(DtasmError::VariableTypeMismatch(MD::VariableType::DtasmInt, id));
            }
            var_vals.int_values.insert(id, val);
        }

        for bool_val in values.bool_vals().iter().flat_map(identity) {
            let id = bool_val.id();
            let val = bool_val.val();
    
            if !map_id_var.contains_key(&id){
                return Err(DtasmError::UnknownVariableId(id));
            }
            if map_id_var[&id].value_type != MD::VariableType::DtasmBool {
                return Err(DtasmError::VariableTypeMismatch(MD::VariableType::DtasmBool, id));
            }
            var_vals.bool_values.insert(id, val);
        }

        for str_val in values.string_vals().iter().flat_map(identity) {
            let id = str_val.id();
            let val = str_val.val().ok_or(DtasmError::InvalidVariableValue("None".to_string(), id))?;
    
            if !map_id_var.contains_key(&id){
                return Err(DtasmError::UnknownVariableId(id));
            }
            if map_id_var[&id].value_type != MD::VariableType::DtasmString {
                return Err(DtasmError::VariableTypeMismatch(MD::VariableType::DtasmString, id));
            }
            var_vals.string_values.insert(id, val.to_string());
        }

        Ok(var_vals)
    }


    fn collect_var_types(md: &MD::ModelDescription) -> Result<HashMap<i32, DtasmVarType>, DtasmtimeError> {
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

        Ok(var_types)
    }

    /// Load a serialized state from file into this instance
    pub fn load_state(&mut self, filepath: PathBuf) -> Result<(), DtasmtimeError>{
        let mut file = std::fs::File::open(filepath)?;

        let mut buffer = Vec::new();
        // read the whole file
        file.read_to_end(&mut buffer)?;

        let state_size = buffer.len() as u64;
        let mem_size = &self.memory.size(&mut self.store);

        if state_size > &self.memory.size(&mut self.store) * WASM_PAGE_SIZE {
            let add_pages = state_size  / WASM_PAGE_SIZE - mem_size;
            let old_size = &self.memory.grow(&mut self.store, add_pages)?;
            assert!(old_size == mem_size, "Memory sizing inconsistency detected");
        }

        let _ = &self.memory.data_mut(&mut self.store).copy_from_slice(&buffer[..]);

        Ok(())
    }

    /// Serialize the current state of the instance to a binary file
    pub fn save_state(&mut self, filepath: PathBuf) -> Result<(),DtasmtimeError>{
        let mut file = std::fs::File::create(filepath)?;

        file.write_all(&self.memory.data(&mut self.store))?;

        Ok(())
    }
}
