// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

#ifndef simModule_modelInfo_h
#define simModule_modelInfo_h

#include <string>
#include <map>
#include <vector>

namespace simModule 
{
    struct ModelInfo 
    {
        std::string name; 
        std::string guid;
        int toleranceDefined;
        double tolerance;
        double tStart;
        int endTimeDefined;
        double tEnd;
        bool debugLog;
    };

    struct VarSelector
    {
        std::vector<unsigned int> realIDs;
        std::vector<unsigned int> intIDs;
        std::vector<unsigned int> boolIDs;
        std::vector<unsigned int> stringIDs;
    };

    struct VarValues
    {
        std::vector<double> realValues;
        std::vector<int> intValues;
        std::vector<bool> boolValues;
        std::vector<std::string> stringValues;
    };
}

#endif /* simModule_modelInfo_h*/