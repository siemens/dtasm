#include "dtasm3.h"

#include <fstream>
#include <iterator>
#include <sstream>
#include <iostream>
#include <vector>


using namespace dtasm3;


void print_status(const DtasmStatus status, const std::string call) {
    switch (status){
        case DtasmOK:
            std::cout << call << " returned status: OK" << std::endl;
            break;
        case DtasmDiscard:
            std::cout << call << " returned status: Discard" << std::endl;
            break;
        case DtasmWarning:
            std::cout << call << " returned status: Warning" << std::endl;
            break;
        case DtasmError:
            std::cout << call << " returned status: Error" << std::endl;
            break;
    }
}


void print_var_names(const std::vector<std::string> &out_var_names) {
    std::cout << "t";
    for (auto it = out_var_names.begin(); it != out_var_names.end(); ++it) {
        std::cout << ";" << *it;
    }
    std::cout << std::endl;
}


void print_var_values(double t, 
    const std::vector<int32_t> &var_ids, 
    const std::vector<DtasmVariableType> &var_types,
    const DtasmVarValues &var_values) {

    std::cout << t;
    typedef std::vector<int32_t>::const_iterator int_iter;
    typedef std::vector<DtasmVariableType>::const_iterator type_iter;
    for (std::pair<int_iter, type_iter> i(var_ids.begin(), var_types.begin());
        i.first != var_ids.end();
        ++i.first, ++i.second) {

        switch (*i.second) {
            case DtasmReal: 
                std::cout << ";" << var_values.real_values.at(*i.first);
                break;
            case DtasmInt:
                std::cout << ";" << var_values.int_values.at(*i.first);
                break;
            case DtasmBool:
                std::cout << ";" << var_values.bool_values.at(*i.first);
                break;
            case DtasmString:
                std::cout << ";" << var_values.string_values.at(*i.first);
                break;
        }
    }
    std::cout << std::endl;
}


void check_status_ok(const DtasmStatus status, const std::string call) {
    if (status != DtasmOK) {
        std::stringstream errMsg;
        errMsg << "Non-ok status returned from doStep(): " << status;
        throw std::runtime_error(errMsg.str().c_str());
    }
}


int main(int argc, char *argv[]) {
    double tmin = 0.0;
    double tmax = 10.0;
    int n_steps = 100;
    
    if (argc <= 1) {
        printf("Usage: dtasm3 dtasm_module.wasm [tmin=0.0] [tmax=10.0] [n_steps=1000] \n");
        exit(0);
    }

    std::string wasm_path = argv[1];

    if (argc > 2) {
        tmin = atof(argv[2]);
    }

    if (argc > 3) {
        tmax = atof(argv[3]);
    }

    if (argc > 4) {
        n_steps = atoi(argv[4]);
    }

    double dt = (tmax-tmin)/n_steps;

    std::ifstream wasm_file(wasm_path, std::ios::binary | std::ifstream::in);
    if(!wasm_file.is_open()){
        std::cerr << "Wasm module could not be loaded: " << wasm_path;
        exit(1);
    }
    
    auto wasm_buf = std::make_shared<std::vector<uint8_t>>();
    wasm_file.unsetf(std::ios::skipws);
    std::copy(std::istream_iterator<uint8_t>(wasm_file), 
        std::istream_iterator<uint8_t>(),
        std::back_inserter(*wasm_buf));

    Environment env((size_t)(64 * 1024));
    Module mod = env.load_module(wasm_buf);
    Runtime rt = env.create_runtime(mod);
    auto model_desc = rt.get_model_description();

    DtasmModelInfo mi = model_desc.model;
    std::cout << "ID: " << mi.id << std::endl 
        << "Name: " << mi.name << std::endl 
        << "Description: " << mi.description << std::endl 
        << "Generating Tool: " << mi.generation_tool << std::endl;

    DtasmCapabilities cap = mi.capabilities;
    std::cout << " can_handle_variable_step_size: " << cap.can_handle_variable_step_size << std::endl
        << " can_interpolate_inputs: " << cap.can_interpolate_inputs << std::endl
        << " can_reset_step: " << cap.can_interpolate_inputs << std::endl;

    DtasmVarValues initial_vals;
    for (auto it = model_desc.variables.begin(); it != model_desc.variables.end(); ++it) {
        if (it->has_default) {
            switch (it->value_type) {
                case DtasmReal:
                    initial_vals.real_values[it->id] = it->default_.real_val;
                    break;
                case DtasmInt:
                    initial_vals.int_values[it->id] = it->default_.int_val;
                    break;
                case DtasmBool:
                    initial_vals.bool_values[it->id] = it->default_.bool_val;
                    break;
                case DtasmString:
                    initial_vals.string_values[it->id] = it->default_.string_val;
                    break;
            }
        }
    }
    auto init_status = rt.initialize(initial_vals, tmin, true, tmax, false, 0, DtasmLogInfo, false);
    print_status(init_status, "Init");

    std::vector<int32_t> out_var_ids;
    std::vector<std::string> out_var_names;
    std::vector<DtasmVariableType> out_var_types;
    for (auto it = model_desc.variables.begin(); it != model_desc.variables.end(); ++it) {
        if (it->causality == DtasmCausalityType::Output || it->causality == DtasmCausalityType::Local) {
            out_var_ids.push_back(it->id);
            out_var_names.push_back(it->name);
            out_var_types.push_back(it->value_type);
        }
    }

    DtasmVarValues set_vals_default;
    for (auto it = model_desc.variables.begin(); it != model_desc.variables.end(); ++it) {
        if (it->causality == DtasmCausalityType::Input && it->has_default) {
            switch (it->value_type) {
                case DtasmReal:
                    set_vals_default.real_values[it->id] = it->default_.real_val;
                    break;
                case DtasmInt:
                    set_vals_default.int_values[it->id] = it->default_.int_val;
                    break;
                case DtasmBool:
                    set_vals_default.bool_values[it->id] = it->default_.bool_val;
                    break;
                case DtasmString:
                    set_vals_default.string_values[it->id] = it->default_.string_val;
                    break;
            }
        }
    }

    DtasmGetValuesResponse res;
    auto get_values_status = rt.get_values(out_var_ids, res);
    check_status_ok(get_values_status, "GetValues");

    print_var_names(out_var_names);
    print_var_values(res.current_time, out_var_ids, out_var_types, res.values);

    double t = tmin;
    DtasmDoStepResponse do_step_res;
    DtasmStatus set_values_status;

    for (int i=0; i<n_steps; ++i) {
        do_step_res = rt.do_step(t, dt);
        check_status_ok(do_step_res.status, "DoStep");
        get_values_status = rt.get_values(out_var_ids, res);
        check_status_ok(get_values_status, "GetValues");
        print_var_values(res.current_time, out_var_ids, out_var_types, res.values);
        set_values_status = rt.set_values(set_vals_default);
        check_status_ok(set_values_status, "SetValues");
        t = res.current_time;
    }
}