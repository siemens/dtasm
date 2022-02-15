use std::{collections::HashMap, path::PathBuf};

use dtasmtime::{runtime::{Engine, Instance, Module}, types::{DtasmVarValues, LogLevel}};
use dtasm_base::model_description as MD;

use float_cmp::approx_eq;
use rstest::{fixture, rstest};


struct DtasmFixture {
    inst: Instance,
    map_name_id: HashMap<String, i32>,
    out_ids: Vec<i32>
}

#[fixture]
fn fix() -> DtasmFixture {
    let mut add_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    add_path.push("tests");
    add_path.push("assets");
    add_path.push("add_rs.wasm");

    if !std::path::Path::new(&add_path).exists() {
        panic!("add_rs.wasm not found - did you compile dtasm modules in release mode?");
    }

    let engine = Engine::new().expect("Could not instantiate dtasm engine");
    let mut dtasm_module = Module::new(add_path, &engine).expect("Could not instantiate dtasm module");
    let mut inst = dtasm_module.instantiate().expect("Instantiate failed!");
    let md = inst.get_model_description().expect("Get Model Description failed!");

    let init_vals = DtasmVarValues::new();

    let _init_res = inst.initialize(&init_vals, 0.0, None, None, LogLevel::Warn, true)
        .expect("Failed to initialize add_rs.wasm");

    let mut map_name_id: HashMap<String, i32> = HashMap::new();
    let mut out_ids: Vec<i32> = Vec::new();

    for variable in &md.variables {
        map_name_id.insert(variable.name.to_string(), variable.id);

        if variable.causality == MD::CausalityType::Output {
            out_ids.push(variable.id);
        }
    }

    let fixture = DtasmFixture { 
        inst, 
        map_name_id, 
        out_ids };
    
    fixture
}

#[rstest]
fn it_adds_two_reals(mut fix: DtasmFixture) {
    let mut input_vals = DtasmVarValues::new();
    let real_input1_id = fix.map_name_id["real_in1"];
    let real_input2_id = fix.map_name_id["real_in2"];

    input_vals.real_values.insert(real_input1_id, -7.34);
    input_vals.real_values.insert(real_input2_id, 10.73);

    fix.inst.set_values(&input_vals).expect("Could not set input values");
    let _dostep_res = fix.inst.do_step(0.0,0.02).expect("DoStep failed");
    let get_vals = fix.inst.get_values(&fix.out_ids).expect("Error in get values");

    let out_id = fix.map_name_id["real_out"];
    let result_val = get_vals.values.real_values[&out_id];

    assert!( approx_eq!(f64, result_val, 3.39, ulps = 2) );
}

#[rstest]
fn it_adds_two_ints(mut fix: DtasmFixture) {
    let mut input_vals = DtasmVarValues::new();
    let int_input1_id = fix.map_name_id["int_in1"];
    let int_input2_id = fix.map_name_id["int_in2"];

    input_vals.int_values.insert(int_input1_id, -23456);
    input_vals.int_values.insert(int_input2_id, 634533);

    fix.inst.set_values(&input_vals).expect("Could not set input values");
    let _dostep_res = fix.inst.do_step(0.0,0.02).expect("DoStep failed");
    let get_vals = fix.inst.get_values(&fix.out_ids).expect("Error in get values");

    let out_id = fix.map_name_id["int_out"];
    let result_val = get_vals.values.int_values[&out_id];

    assert_eq!(result_val, 611077);
}

#[rstest]
fn it_ands_two_bools(mut fix: DtasmFixture) {
    let mut input_vals = DtasmVarValues::new();
    let bool_input1_id = fix.map_name_id["bool_in1"];
    let bool_input2_id = fix.map_name_id["bool_in2"];

    input_vals.bool_values.insert(bool_input1_id, true);
    input_vals.bool_values.insert(bool_input2_id, true);

    fix.inst.set_values(&input_vals).expect("Could not set input values");
    let _dostep_res = fix.inst.do_step(0.0,0.02).expect("DoStep failed");
    let get_vals = fix.inst.get_values(&fix.out_ids).expect("Error in get values");

    let out_id = fix.map_name_id["bool_out"];
    let result_val = get_vals.values.bool_values[&out_id];

    assert_eq!(result_val, true);
}

#[rstest]
fn it_concats_two_strings(mut fix: DtasmFixture) {
    let mut input_vals = DtasmVarValues::new();
    let str_input1_id = fix.map_name_id["string_in1"];
    let str_input2_id = fix.map_name_id["string_in2"];

    input_vals.string_values.insert(str_input1_id, "hello".to_string());
    input_vals.string_values.insert(str_input2_id, " world".to_string());

    fix.inst.set_values(&input_vals).expect("Could not set input values");
    let _dostep_res = fix.inst.do_step(0.0,0.02).expect("DoStep failed");
    let get_vals = fix.inst.get_values(&fix.out_ids).expect("Error in get values");

    let out_id = fix.map_name_id["string_out"];
    let result_val = &get_vals.values.string_values[&out_id];

    assert_eq!(*result_val, "hello world".to_string());
}