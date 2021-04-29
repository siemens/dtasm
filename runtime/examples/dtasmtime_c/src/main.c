// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
#include <math.h>

#include <sys/resource.h>

#include <dtasmtime-c-api.h>


int main(int argc, char *argv[]) {
    double tmin = 0.0;
    double tmax = 10.0;
    int n_steps = 1000;
    const char* dtasm_file;

    if (argc <= 1)
    {
        printf("Usage: dtasmtime-c dtasmModule.wasm [tmin=0.0] [tmax=10.0] [n_steps=1000] \n");
        exit(0);
    }

    dtasm_file = argv[1];

    if (argc > 2)
    {
        tmin = atof(argv[2]);
    }

    if (argc > 3)
    {
        tmax = atof(argv[3]);
    }

    if (argc > 4)
    {
        n_steps = atoi(argv[4]);
    }

    double dt = (tmax-tmin)/n_steps;
    
    printf("Creating engine... ");
    Engine *engine = dtasmtime_engine_new();
    printf("Ok.\n");

    printf("Creating Module... ");
    Module *module = dtasmtime_module_new(dtasm_file, engine);
    printf("Ok.\n");

    printf("Instantiating Module... ");
    Instance *inst = dtasmtime_module_instantiate(module);
    printf("Ok.\n");

    printf("Getting model description... \n");
    DtasmModelDescription md = dtasmtime_modeldescription_get(inst);

    DtasmModelInfo mi = md.model;
    printf(" ID: %s,\n Name: %s,\n Description: %s,\n GenTool: %s \n", mi.id, mi.name, mi.description, mi.generation_tool);

    DtasmCapabilities cap = mi.capabilities;
    printf(" can_handle_variable_step_size: %d\n", cap.can_handle_variable_step_size);
    printf(" can_interpolate_inputs: %d\n", cap.can_interpolate_inputs);
    printf(" can_reset_step: %d\n", cap.can_interpolate_inputs);

    if (md.has_experiment) 
    {
        DtasmExperimentInfo ei = md.experiment;
        printf(" Start time default: %f,\n End time default: %f,\n Timestep default: %f,\n Time unit: %s\n", 
            ei.start_time_default, 
            ei.end_time_default, 
            ei.time_step_default, 
            ei.time_unit);
    }

    printf("Variables: \n");
    DtasmModelVariable *var = md.variables;
    int var_count = md.n_variables;
    int real_init_count = 0;
    int output_state_count = 0;
   
    for (int i=0; i<var_count; i++)
    {
        printf(" Name: %s\n", var[i].name);
        printf(" Desc: %s\n", var[i].description);
        printf(" Id: %d\n", var[i].id);

        if (var[i].has_default)
        {
            switch (var[i].value_type){
                case DtasmReal:
                    real_init_count++;
                    printf(" Default value: %f\n", var[i].default_.real_val);
                    break;
            }
        }

        if (var[i].causality == Output || var[i].causality == Local)
            output_state_count++;
    }

    DtasmVarValues initial_vals;
    initial_vals.n_reals = real_init_count;
    initial_vals.real_ids = malloc(real_init_count * sizeof(int));
    initial_vals.real_values = malloc(real_init_count * sizeof(double));

//TODO: ints, bools and strings
    initial_vals.n_ints = 0;
    initial_vals.int_ids = NULL;
    initial_vals.int_values = NULL;

    initial_vals.n_bools = 0;
    initial_vals.bool_ids = NULL;
    initial_vals.bool_values = NULL;

    initial_vals.n_strings = 0;
    initial_vals.string_ids = NULL;
    initial_vals.string_values = NULL;

    int i_var = 0;
    for (int i=0; i<var_count; i++)
    {
        if (var[i].value_type == DtasmReal)
        {
            if (var[i].has_default)
            {
                initial_vals.real_ids[i_var] = var[i].id;
                initial_vals.real_values[i_var] = var[i].default_.real_val;
                i_var++;
            }
        }
    }

    printf("Calling initialize...\n");
    DtasmStatus st = dtasmtime_initialize(inst, initial_vals, tmin, true, tmax, false, 0.0, DtasmLogWarn, false);
    printf("Returned %d\n", st);

    free(initial_vals.real_ids);
    free(initial_vals.real_values);

    int32_t * req_ids = malloc(output_state_count * sizeof(int));
    int i_outvar = 0;
    for (int i=0; i<var_count; i++)
    {
        if (var[i].causality == Output || var[i].causality == Local)
        {
            req_ids[i_outvar] = var[i].id;
            i_outvar++;
        }
    }

    printf("Calling get values...\n");
    DtasmGetValuesResponse get_vals_res = dtasmtime_getvalues(inst, req_ids, output_state_count);
    printf("Received status: %d\n", get_vals_res.status);
    printf("Current time: %f\n", get_vals_res.current_time);

    for (int i = 0; i < get_vals_res.values.n_reals; ++i)
    {
        printf("Value for var id %d: %f\n", get_vals_res.values.real_ids[i], get_vals_res.values.real_values[i]);
    }

    printf("Freeing getvalues res... ");
    dtasmtime_getvalues_free(get_vals_res);
    printf("Ok.\n");

    double t = tmin;

    struct rusage r_usage;
    getrusage(RUSAGE_SELF,&r_usage);
    printf("Memory usage: %ld kilobytes\n", r_usage.ru_maxrss);

    for (int i=0; i<n_steps; ++i) {
        printf("Calling do_step...\n");
        DtasmDoStepResponse dostep_res = dtasmtime_dostep(inst, t, dt);
        printf("Returned %d, updated time %f\n", dostep_res.status, dostep_res.updated_time);

        get_vals_res = dtasmtime_getvalues(inst, req_ids, output_state_count);
        for (int i = 0; i < get_vals_res.values.n_reals; ++i) {
            printf("Value for var id %d: %f\n", get_vals_res.values.real_ids[i], get_vals_res.values.real_values[i]);
        }
        dtasmtime_getvalues_free(get_vals_res);

        getrusage(RUSAGE_SELF,&r_usage);
        printf("Memory usage: %ld kilobytes\n", r_usage.ru_maxrss);

        t = dostep_res.updated_time;
    }

    printf("Freeing model description... ");
    dtasmtime_modeldescription_free(md);
    printf("Ok.\n");

    printf("Freeing Instance... ");
    dtasmtime_instance_free(inst);
    printf("Ok.\n");

    printf("Freeing Module... ");
    dtasmtime_module_free(module);
    printf("Ok.\n");

    printf("Freeing Engine... ");
    dtasmtime_engine_free(engine);
    printf("Ok.\n");
}
