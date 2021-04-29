/* solve_dpend.c
 *
 * Example code to solve double pendulum ODEs using fourth order 
 * Runge-Kutta. 
 *
 * Parameters are passed in at the command line:
 * 
 * $./solve_dpend TMIN TMAX TH10 W10 TH20 W20 NSTEP > pendulum.txt 
 *
 * where TMIN and TMAX are the starting and ending times (in seconds), 
 * TH10 and TH20 are the initial angles (degrees), and W10 and W20 
 * are the initial angular velocities (degrees per second), and 
 * NSTEP is the number of integrations steps. This example illustrates 
 * using redirection to write the results to file in a file
 * pendulum.txt. Note that there is no checking for accuracy, so the
 * user needs to choose a suitable NSTEP. Also angles written to file
 * are in radians.
 *
 * As an example, the data for the first animated gif on the web page 
 * may be generated with   
 *
 * $./solve_dpend 0.0 10.0 90.0 0.00 -10.0 0.0 1000 > outfile.txt
 *
 * (only every fifth frame is used in the animation).
 * 
 * M.S. Wheatland, 2004
 *
 */

#include <stdlib.h>
#include <math.h>
#include "dpend.h"

#define WASM_EXPORT __attribute__((used)) __attribute__((visibility("default")))

/* hardwired parameters */

#define PI 3.14159265
#define N 4    /* number of equations to be solved */
#define G 9.8  /* acc'n due to gravity, in m/s^2 */
//#define L1 1.0 /* length of pendulum 1 in m */
//#define L2 1.0 /* length of pendulum 2 in m */
//#define M1 1.0 /* mass of pendulum 1 in kg */
//#define M2 1.0 /* mass of pendulum 2 in kg */

void runge_kutta(const dpend_params* param, double xin, double yin[], double yout[], double h);
void derivs(const dpend_params* param,double xin, double yin[], double dydx[]);

void dp_step(const dpend_params* param, dpend_state* state, dpend_input* in)
{

    double yin[N], yout[N];
    double t = state->t;
    double h = in->dt;

    yin[0] = state->th1;
    yin[1] = state->w1;
    yin[2] = state->th2;
    yin[3] = state->w2;

    runge_kutta(param, t, yin, yout, h);

    state->th1 = yout[0];
    state->w1 = yout[1];
    state->th2 = yout[2];
    state->w2 = yout[3];

    state->t = t+h;
}


void derivs(const dpend_params* p, double xin, double yin[], double dydx[])
{
    /* function to fill array of derivatives dydx at xin */

    double den1, den2, del;

    dydx[0] = yin[1];

    del = yin[2] - yin[0];
    den1 = (p->m1 + p->m2) * p->l1 - p->m2 * p->l1 * cos(del) * cos(del);
    dydx[1] = (p->m2 * p->l1 * yin[1] * yin[1] * sin(del) * cos(del) + p->m2 * G * sin(yin[2]) * cos(del) + p->m2 * p->l2 * yin[3] * yin[3] * sin(del) - (p->m1 + p->m2) * G * sin(yin[0])) / den1;

    dydx[2] = yin[3];

    den2 = (p->l2 / p->l1) * den1;
    dydx[3] = (-p->m2 * p->l2 * yin[3] * yin[3] * sin(del) * cos(del) + (p->m1 + p->m2) * G * sin(yin[0]) * cos(del) - (p->m1 + p->m2) * p->l1 * yin[1] * yin[1] * sin(del) - (p->m1 + p->m2) * G * sin(yin[2])) / den2;

    return;
}


void runge_kutta(const dpend_params* param, double xin, double yin[], double yout[], double h)
{
    /* fourth order Runge-Kutta - see e.g. Numerical Recipes */

    int i;
    double hh, xh, dydx[N], dydxt[N], yt[N], k1[N], k2[N], k3[N], k4[N];

    hh = 0.5 * h;
    xh = xin + hh;

    derivs(param, xin, yin, dydx); /* first step */
    for (i = 0; i < N; i++)
    {
        k1[i] = h * dydx[i];
        yt[i] = yin[i] + 0.5 * k1[i];
    }

    derivs(param, xh, yt, dydxt); /* second step */
    for (i = 0; i < N; i++)
    {
        k2[i] = h * dydxt[i];
        yt[i] = yin[i] + 0.5 * k2[i];
    }

    derivs(param, xh, yt, dydxt); /* third step */
    for (i = 0; i < N; i++)
    {
        k3[i] = h * dydxt[i];
        yt[i] = yin[i] + k3[i];
    }

    derivs(param, xin + h, yt, dydxt); /* fourth step */
    for (i = 0; i < N; i++)
    {
        k4[i] = h * dydxt[i];
        yout[i] = yin[i] + k1[i] / 6. + k2[i] / 3. + k3[i] / 3. + k4[i] / 6.;
    }

    return;
}
