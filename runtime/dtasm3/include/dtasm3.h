#pragma once

#include <memory>
#include <vector>
#include <map>


namespace dtasm3
{
    typedef enum DtasmCausalityType {
        Local,
        Parameter,
        Input,
        Output,
    } DtasmCausalityType;


    typedef enum DtasmLogLevel {
        DtasmLogError,
        DtasmLogWarn,
        DtasmLogInfo,
    } DtasmLogLevel;


    typedef enum DtasmStatus {
        DtasmOK,
        DtasmWarning,
        DtasmDiscard,
        DtasmError,
        DtasmFatal,
    } DtasmStatus;


    typedef enum DtasmVariableType {
        DtasmReal,
        DtasmInt,
        DtasmBool,
        DtasmString,
    } DtasmVariableType;


    typedef struct DtasmCapabilities {
        bool can_handle_variable_step_size;
        bool can_reset_step;
        bool can_interpolate_inputs;
    } DtasmCapabilities;


    typedef struct DtasmModelInfo {
        std::string name;
        std::string id;
        std::string description;
        std::string generation_tool;
        std::string generation_date_time;
        std::string name_delimiter;
        DtasmCapabilities capabilities;
    } DtasmModelInfo;


    typedef struct DtasmExperimentInfo {
        double time_step_min;
        double time_step_max;
        double time_step_default;
        double start_time_default;
        double end_time_default;
        std::string time_unit;
    } DtasmExperimentInfo;


    typedef struct DtasmVariableValue {
        double real_val;
        int32_t int_val;
        bool bool_val;
        std::string string_val;
    } DtasmVariableValue;


    typedef struct DtasmModelVariable {
        int32_t id;
        std::string name;
        DtasmVariableType value_type;
        std::string description;
        std::string unit;
        DtasmCausalityType causality;
        int32_t derivative_of_id;
        DtasmVariableValue default_;
        bool has_default;
    } DtasmModelVariable;


    typedef struct DtasmModelDescription {
        DtasmModelInfo model;
        DtasmExperimentInfo experiment;
        bool has_experiment;
        std::vector<DtasmModelVariable> variables;
    } DtasmModelDescription;


    typedef struct DtasmVarValues {
        std::map<int32_t, double> real_values;
        std::map<int32_t, int> int_values;
        std::map<int32_t, bool> bool_values;
        std::map<int32_t, std::string> string_values;
    } DtasmVarValues;


    typedef struct DtasmGetValuesResponse {
        enum DtasmStatus status;
        double current_time;
        struct DtasmVarValues values;
    } DtasmGetValuesResponse;


    typedef struct DtasmDoStepResponse {
        DtasmStatus status;
        double updated_time;
    } DtasmDoStepResponse;


    class Runtime {
    private:
        class Impl;
        std::unique_ptr<Impl> _rt;

    protected:
        friend class Environment;
        Runtime(Impl &impl);
    
    public: 
        ~Runtime();
        Runtime(const Runtime &rt);
        DtasmModelDescription get_model_description();
        
        DtasmStatus initialize(
            const DtasmVarValues &initial_vals,
            double tmin,
            bool tmax_set,
            double tmax,
            bool tol_set,
            double tol,
            DtasmLogLevel log_level,
            bool check);

        DtasmStatus get_values(const std::vector<int32_t> &var_ids, 
            DtasmGetValuesResponse &res);
        DtasmStatus set_values(const DtasmVarValues &set_vals);
        DtasmDoStepResponse do_step(double t, double dt);
        void save_state(std::vector<uint8_t> &state_buffer);
        void load_state(const std::vector<uint8_t> &state_buffer);
    };


    class Module {
    private: 
        friend class Environment;
        class Impl;
        std::unique_ptr<Impl> _mod;

    protected: 
        Module(Impl &impl);

    public: 
        Module(const Module &mod);
        ~Module();
    };


    class Environment {
    private: 
        class Impl;    
        std::unique_ptr<Impl> _env;

    public: 
        Environment(size_t stack_size_bytes);
        ~Environment();
        Module load_module(const uint8_t* data, const size_t len);
        Module load_module(std::shared_ptr<std::vector<uint8_t>> data);
        Runtime create_runtime(Module &mod, int buffersize = 8192);
    };
}
