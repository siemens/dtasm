// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

#ifndef DPEND_H
#define DPEND_H

typedef struct dpend_state 
{
    double t;
    double th1;
    double th2;
    double w1;
    double w2;
} dpend_state;

typedef struct dpend_params
{
    double m1;
    double m2;
    double l1;
    double l2;
} dpend_params;

typedef struct dpend_input
{
    double dt;
    double a1;
    double a2;
} dpend_input;

#ifdef __cplusplus
extern "C" {
#endif
void dp_step(const dpend_params* param, dpend_state* state, dpend_input* in);
#ifdef __cplusplus
}
#endif

#endif