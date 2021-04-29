// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use std::f64::consts::PI;

#[derive(Default,Debug)]
pub struct DpendState 
{
    pub t: f64,
    pub th1: f64,
    pub th2: f64,
    pub w1: f64, 
    pub w2: f64
}

#[derive(Default,Debug)]
pub struct DpendParams
{
    pub m1: f64,
    pub m2: f64, 
    pub l1: f64,
    pub l2: f64
}

#[derive(Default)]
pub struct DpendInput
{
    pub dt: f64, 
    pub a1: f64, 
    pub a2: f64
}

const N: usize = 4;
const G: f64 = 9.81;

// fn main() {
//     let args = Cli::from_args();

//     println!("{}, {}, {}, {}, {}", args.t0, args.tmax, args.th10, args.th20, args.n);

//     run(args.t0, args.tmax, args.th10, 0.0, args.th20, 0.0, args.n);

//     println!("Done.");
// }

#[no_mangle]
extern "C"
fn run(tmin: f64, tmax: f64, th10: f64, w10: f64, th20: f64, w20: f64, nstep: i32)
{
    let h = (tmax - tmin) / (nstep as f64 - 1.0);

    let inp = DpendInput {
        dt: h, 
        a1: 0.0, 
        a2: 0.0
    };

    let params = DpendParams {
        l1: 1.0, 
        l2: 1.0, 
        m1: 1.0, 
        m2: 1.0
    };

    let mut st = DpendState {
        t: tmin, 
        th1: th10 * PI / 180.0,
        th2: th20 * PI / 180.0, 
        w1: w10 * PI / 180.0,
        w2: w20 * PI / 180.0
    };

    println!("{} {} {} {} {}", st.t, st.th1, st.w1, st.th2, st.w2);
    
    for _ in 0..nstep-1 {
        dp_step(&params, &mut st, &inp);

        println!("{} {} {} {} {}", st.t, st.th1, st.w1, st.th2, st.w2);
    }
}



pub fn dp_step(param: &DpendParams, state: &mut DpendState, input: &DpendInput)
{
    let mut yin: [f64; N] = [0.0; N];
    let mut yout: [f64; N] = [0.0; N];
    let mut uin: [f64; 2] = [0.0; 2];
    let t: f64 = state.t;
    let h: f64 = input.dt;

    yin[0] = state.th1;
    yin[1] = state.w1;
    yin[2] = state.th2;
    yin[3] = state.w2;

    uin[0] = input.a1;
    uin[1] = input.a2;

    runge_kutta(param, t, yin, uin, &mut yout, h);

    state.th1 = yout[0];
    state.w1 = yout[1];
    state.th2 = yout[2];
    state.w2 = yout[3];

    state.t = t+h;
}


fn derivs(p: &DpendParams, _xin: f64, yin: [f64; N], u_in: [f64; 2], dydx: &mut [f64; N])
{
    /* function to fill array of derivatives dydx at xin */

    let den1: f64; 
    let den2: f64;
    let del: f64;

    dydx[0] = yin[1];

    del = yin[2] - yin[0];
    den1 = (p.m1 + p.m2) * p.l1 - p.m2 * p.l1 * del.cos() * del.cos();
    dydx[1] = ((p.m2 * p.l1 * yin[1] * yin[1] * del.sin() * del.cos() 
        + p.m2 * G * yin[2].sin() * del.cos() 
        + p.m2 * p.l2 * yin[3] * yin[3] * del.sin() 
        - (p.m1 + p.m2) * G * yin[0].sin()) / den1) + u_in[0];

    dydx[2] = yin[3];

    den2 = (p.l2 / p.l1) * den1;
    dydx[3] = ((-p.m2 * p.l2 * yin[3] * yin[3] * del.sin() * del.cos() 
        + (p.m1 + p.m2) * G * yin[0].sin() * del.cos() 
        - (p.m1 + p.m2) * p.l1 * yin[1] * yin[1] * del.sin() 
        - (p.m1 + p.m2) * G * yin[2].sin()) / den2) + u_in[1];
}


fn runge_kutta(param: &DpendParams, xin: f64, yin: [f64; N], u_in: [f64; 2], yout: &mut [f64; N], h: f64) {
    /* fourth order Runge-Kutta - see e.g. Numerical Recipes */

    let hh: f64;
    let xh: f64;
    let mut dydx: [f64; N] = [0.0; N];
    let mut dydxt: [f64; N] = [0.0; N];
    let mut yt: [f64; N] = [0.0; N];
    let mut k1: [f64; N] = [0.0; N];
    let mut k2: [f64; N] = [0.0; N];
    let mut k3: [f64; N] = [0.0; N];
    let mut k4: [f64; N] = [0.0; N];

    hh = 0.5 * h;
    xh = xin + hh;

    derivs(param, xin, yin, u_in, &mut dydx); /* first step */
    for i in 0..k1.len() {
        k1[i] = h * dydx[i];
        yt[i] = yin[i] + 0.5 * k1[i];
    }

    derivs(param, xh, yt, u_in, &mut dydxt); /* second step */
    for i in 0..k2.len() {
        k2[i] = h * dydxt[i];
        yt[i] = yin[i] + 0.5 * k2[i];
    }

    derivs(param, xh, yt, u_in, &mut dydxt); /* third step */
    for i in 0..k3.len() {
        k3[i] = h * dydxt[i];
        yt[i] = yin[i] + k3[i];
    }

    derivs(param, xin + h, yt, u_in, &mut dydxt); /* fourth step */
    for i in 0..k4.len() {
        k4[i] = h * dydxt[i];
        yout[i] = yin[i] + k1[i] / 6. + k2[i] / 3. + k3[i] / 3. + k4[i] / 6.;
    }
}