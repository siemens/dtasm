// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

extern crate alloc;
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::String;

use dtasm_rs::model_description::ModelDescription;

#[derive(Eq, PartialOrd, Ord, PartialEq, Clone, Copy, Debug)]
pub enum AddVar {RI1, RI2, RO, II1, II2, IO, BI1, BI2, BO, SI1, SI2, SO}

#[derive(Debug)]
pub struct AddVarMaps
{
    pub map_id_var: BTreeMap<i32, AddVar>
}

impl AddVarMaps {
    pub fn new() -> AddVarMaps {
        AddVarMaps {
            map_id_var: BTreeMap::new(), 
        }
    }
}

#[derive(Debug)]
pub struct AddState 
{
    pub t: f64,

    pub var_maps: AddVarMaps,
    pub real_values: BTreeMap<AddVar, f64>,
    pub int_values: BTreeMap<AddVar, i32>,
    pub bool_values: BTreeMap<AddVar, bool>,
    pub string_values: BTreeMap<AddVar, String>,
}

unsafe impl Sync for AddState {}

impl AddState {
    pub fn new() -> AddState {
        let mut add_state = AddState {
            t: 0.0, 

            var_maps: AddVarMaps::new(),
            real_values: BTreeMap::new(),
            int_values: BTreeMap::new(),
            bool_values: BTreeMap::new(),
            string_values: BTreeMap::new(),
        };

        add_state.real_values.insert(AddVar::RI1, 0.0);
        add_state.real_values.insert(AddVar::RI2, 0.0);
        add_state.real_values.insert(AddVar::RO, 0.0);

        add_state.int_values.insert(AddVar::II1, 0);
        add_state.int_values.insert(AddVar::II2, 0);
        add_state.int_values.insert(AddVar::IO, 0);

        add_state.bool_values.insert(AddVar::BI1, false);
        add_state.bool_values.insert(AddVar::BI2, false);
        add_state.bool_values.insert(AddVar::BO, false);

        add_state.string_values.insert(AddVar::SI1, "".to_owned());
        add_state.string_values.insert(AddVar::SI2, "".to_owned());
        add_state.string_values.insert(AddVar::SO, "".to_owned());

        add_state
    }
}

pub fn create_var_maps(dt_md: &ModelDescription,
    var_maps: &mut AddVarMaps) {
    let md_vars = &dt_md.variables;

    for model_var in md_vars {
        let name = &model_var.name;
        let id = model_var.id;

        let add_var: AddVar;

        if name == "real_in1"{
            add_var = AddVar::RI1;
        }
        else if name == "real_in2"{
            add_var = AddVar::RI2;
        }
        else if name == "real_out"{
            add_var = AddVar::RO;
        }
        else if name == "int_in1"{
            add_var = AddVar::II1;
        }
        else if name == "int_in2"{
            add_var = AddVar::II2;
        }
        else if name == "int_out"{
            add_var = AddVar::IO;
        }
        else if name == "bool_in1"{
            add_var = AddVar::BI1;
        }
        else if name == "bool_in2"{
            add_var = AddVar::BI2;
        }
        else if name == "bool_out"{
            add_var = AddVar::BO;
        }
        else if name == "string_in1"{
            add_var = AddVar::SI1;
        }
        else if name == "string_in2"{
            add_var = AddVar::SI2;
        }
        else if name == "string_out"{
            add_var = AddVar::SO;
        }
        else {
            continue;
        }

        var_maps.map_id_var.insert(id, add_var);
    }
}
