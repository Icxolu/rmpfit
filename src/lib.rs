/// Definition of a parameter constraint structure
pub struct MPPar {
    pub fixed: bool,
    pub limited_low: bool,
    pub limited_up: bool,
    pub limit_low: f64,
    pub limit_up: f64,
    /// Step size for finite difference
    pub step: f64,
    /// Relative step size for finite difference
    pub rel_step: f64,
    /// Sidedness of finite difference derivative
    pub side: MPSide,
}

impl ::std::default::Default for MPPar {
    fn default() -> Self {
        MPPar {
            fixed: false,
            limited_low: false,
            limited_up: false,
            limit_low: 0.0,
            limit_up: 0.0,
            step: 0.0,
            rel_step: 0.0,
            side: MPSide::Auto,
        }
    }
}

/// Sidedness of finite difference derivative
#[derive(Copy, Clone)]
pub enum MPSide {
    /// one-sided derivative computed automatically
    Auto,
    /// one-sided derivative (f(x+h) - f(x)  )/h
    Right,
    /// one-sided derivative (f(x)   - f(x-h))/h
    Left,
    /// two-sided derivative (f(x+h) - f(x-h))/(2*h)
    Both,
    /// user-computed analytical derivatives
    User,
}

/// Definition of MPFIT configuration structure
pub struct MPConfig {
    /// Relative chi-square convergence criterion  Default: 1e-10
    pub ftol: f64,
    /// Relative parameter convergence criterion   Default: 1e-10
    pub xtol: f64,
    /// Orthogonality convergence criterion        Default: 1e-10
    pub gtol: f64,
    /// Finite derivative step size                Default: f64::EPSILON
    pub epsfcn: f64,
    /// Initial step bound                         Default: 100.0
    pub step_factor: f64,
    /// Range tolerance for covariance calculation Default: 1e-14
    pub covtol: f64,
    /// Maximum number of iterations.  If maxiter == 0,
    /// then basic error checking is done, and parameter
    /// errors/covariances are estimated based on input
    /// parameter values, but no fitting iterations are done.
    pub max_iter: usize,
    /// Maximum number of function evaluations, or 0 for no limit
    /// Default: 0 (no limit)
    pub max_fev: usize,
    /// Default: true
    pub n_print: bool,
    /// Scale variables by user values?
    /// true = yes, user scale values in diag;
    /// false = no, variables scaled internally (Default)
    pub do_user_scale: bool,
    /// Disable check for infinite quantities from user?
    /// true = perform check
    /// false = do not perform check (Default)
    pub no_finite_check: bool,
}

impl ::std::default::Default for MPConfig {
    fn default() -> Self {
        MPConfig {
            ftol: 1e-10,
            xtol: 1e-10,
            gtol: 1e-10,
            epsfcn: f64::EPSILON,
            step_factor: 100.0,
            covtol: 1e-14,
            max_iter: 0,
            max_fev: 0,
            n_print: true,
            do_user_scale: false,
            no_finite_check: false,
        }
    }
}

/// MP Fit errors
pub enum MPError {
    /// General input parameter error
    Input,
    /// User function produced non-finite values
    Nan,
    /// No user data points were supplied
    Empty,
    /// No free parameters
    NoFree,
    /// Initial values inconsistent with constraints
    InitBounds,
    /// Initial constraints inconsistent
    Bounds,
    /// Not enough degrees of freedom
    DoF,
}

/// Potential success status
pub enum MPSuccess {
    /// Convergence in chi-square value
    Chi,
    /// Convergence in parameter value
    Par,
    /// Convergence in both chi-square and parameter
    Both,
    /// Convergence in orthogonality
    Dir,
    /// Maximum number of iterations reached
    MaxIter,
    /// ftol is too small; no further improvement
    Ftol,
    /// xtol is too small; no further improvement
    Xtol,
    /// gtol is too small; no further improvement
    Gtol,
}

// MP Fit Result
pub enum MPResult {
    Success(MPSuccess, MPStatus),
    Error(MPError),
}

/// Definition of results structure, for when fit completes
pub struct MPStatus {
    /// Final chi^2
    pub best_norm: f64,
    /// Starting value of chi^2
    pub orig_norm: f64,
    /// Number of iterations
    pub n_iter: usize,
    /// Number of function evaluations
    pub n_fev: usize,
    /// Total number of parameters
    pub n_par: usize,
    /// Number of free parameters
    pub n_free: usize,
    /// Number of pegged parameters
    pub n_pegged: usize,
    /// Number of residuals (= num. of data points)
    pub n_func: usize,
    /// Final residuals nfunc-vector
    pub resid: Vec<f64>,
    /// Final parameter uncertainties (1-sigma) npar-vector
    pub xerror: Vec<f64>,
    /// Final parameter covariance matrix npar x npar array
    pub covar: Vec<f64>,
}

pub trait MPFitter {
    fn eval(&self, params: &[f64], deviates: &mut [f64]);

    fn number_of_points(&self) -> usize;
}

/// (f64::MIN_POSITIVE * 1.5).sqrt() * 10
const MP_RDWARF: f64 = 1.8269129289596699e-153;
/// f64::MAX.sqrt() * 0.1
const MP_RGIANT: f64 = 1.3407807799935083e+153;

struct MPFit<'a> {
    m: usize,
    npar: usize,
    nfree: usize,
    ifree: Vec<usize>,
    fvec: Vec<f64>,
    nfev: usize,
    xnew: Vec<f64>,
    x: Vec<f64>,
    xall: &'a [f64],
    qtf: Vec<f64>,
    fjack: Vec<f64>,
    side: Vec<MPSide>,
    step: Vec<f64>,
    dstep: Vec<f64>,
}

impl<'a> MPFit<'a> {
    fn new(m: usize, xall: &[f64]) -> Option<MPFit> {
        let npar = xall.len();
        if m == 0 {
            None
        } else {
            Some(MPFit {
                m,
                npar,
                nfree: 0,
                ifree: vec![],
                fvec: vec![0.; m],
                nfev: 1,
                xnew: vec![0.; npar],
                x: vec![],
                xall: &xall,
                qtf: vec![],
                fjack: vec![],
                side: Vec::with_capacity(npar),
                step: Vec::with_capacity(npar),
                dstep: Vec::with_capacity(npar),
            })
        }
    }

    ///    function enorm
    ///
    ///    given an n-vector x, this function calculates the
    ///    euclidean norm of x.
    ///
    ///    the euclidean norm is computed by accumulating the sum of
    ///    squares in three different sums. the sums of squares for the
    ///    small and large components are scaled so that no overflows
    ///    occur. non-destructive underflows are permitted. underflows
    ///    and overflows do not occur in the computation of the unscaled
    ///    sum of squares for the intermediate components.
    ///    the definitions of small, intermediate and large components
    ///    depend on two constants, rdwarf and rgiant. the main
    ///    restrictions on these constants are that rdwarf**2 not
    ///    underflow and rgiant**2 not overflow. the constants
    ///    given here are suitable for every known computer.
    ///    the function statement is
    ///    double precision function enorm(n,x)
    ///    where
    ///
    ///    n is a positive integer input variable.
    ///
    ///    x is an input array of length n.
    ///
    ///    subprograms called
    ///
    ///    fortran-supplied ... dabs,dsqrt
    ///
    ///    argonne national laboratory. minpack project. march 1980.
    ///    burton s. garbow, kenneth e. hillstrom, jorge j. more
    fn enorm(&self) -> f64 {
        let mut s1 = 0.;
        let mut s2 = 0.;
        let mut s3 = 0.;
        let mut x1max = 0.;
        let mut x3max = 0.;
        let agiant = MP_RGIANT / self.m as f64;
        for val in &self.fvec {
            let xabs = val.abs();
            if xabs > MP_RDWARF && xabs < agiant {
                // sum for intermediate components.
                s2 += xabs * xabs;
            } else if xabs > MP_RDWARF {
                // sum for large components.
                if xabs > x1max {
                    let temp = x1max / xabs;
                    s1 = 1.0 + s1 * temp * temp;
                    x1max = xabs;
                } else {
                    let temp = xabs / x1max;
                    s1 += temp * temp;
                }
            } else if xabs > x3max {
                // sum for small components.
                let temp = x3max / xabs;
                s3 = 1.0 + s3 * temp * temp;
                x3max = xabs;
            } else if xabs != 0.0 {
                let temp = xabs / x3max;
                s3 += temp * temp;
            }
        }
        // calculation of norm.
        if s1 != 0.0 {
            x1max * (s1 + (s2 / x1max) / x1max).sqrt()
        } else if s2 != 0.0 {
            if s2 >= x3max {
                s2 * (1.0 + (x3max / s2) * (x3max * s3))
            } else {
                x3max * ((s2 / x3max) + (x3max * s3))
            }
            .sqrt()
        } else {
            x3max * s3.sqrt()
        }
    }

    ///     subroutine fdjac2
    ///
    ///     this subroutine computes a forward-difference approximation
    ///     to the m by n jacobian matrix associated with a specified
    ///     problem of m functions in n variables.
    ///
    ///     the subroutine statement is
    ///
    ///	subroutine fdjac2(fcn,m,n,x,fvec,fjac,ldfjac,iflag,epsfcn,wa)
    ///
    ///     where
    ///
    ///	fcn is the name of the user-supplied subroutine which
    ///	  calculates the functions. fcn must be declared
    ///	  in an external statement in the user calling
    ///	  program, and should be written as follows.
    ///
    ///	  subroutine fcn(m,n,x,fvec,iflag)
    ///	  integer m,n,iflag
    ///	  double precision x(n),fvec(m)
    ///	  ----------
    ///	  calculate the functions at x and
    ///	  return this vector in fvec.
    ///	  ----------
    ///	  return
    ///	  end
    ///
    ///	  the value of iflag should not be changed by fcn unless
    ///	  the user wants to terminate execution of fdjac2.
    ///	  in this case set iflag to a negative integer.
    ///
    ///	m is a positive integer input variable set to the number
    ///	  of functions.
    ///
    ///	n is a positive integer input variable set to the number
    ///	  of variables. n must not exceed m.
    ///
    ///	x is an input array of length n.
    ///
    ///	fvec is an input array of length m which must contain the
    ///	  functions evaluated at x.
    ///
    ///	fjac is an output m by n array which contains the
    ///	  approximation to the jacobian matrix evaluated at x.
    ///
    ///	ldfjac is a positive integer input variable not less than m
    ///	  which specifies the leading dimension of the array fjac.
    ///
    ///	iflag is an integer variable which can be used to terminate
    ///	  the execution of fdjac2. see description of fcn.
    ///
    ///	epsfcn is an input variable used in determining a suitable
    ///	  step length for the forward-difference approximation. this
    ///	  approximation assumes that the relative errors in the
    ///	  functions are of the order of epsfcn. if epsfcn is less
    ///	  than the machine precision, it is assumed that the relative
    ///	  errors in the functions are of the order of the machine
    ///	  precision.
    ///
    ///	wa is a work array of length m.
    ///
    ///     subprograms called
    ///
    ///	user-supplied ...... fcn
    ///
    ///	minpack-supplied ... dpmpar
    ///
    ///	fortran-supplied ... dabs,dmax1,dsqrt
    ///
    ///     argonne national laboratory. minpack project. march 1980.
    ///     burton s. garbow, kenneth e. hillstrom, jorge j. more
    ///
    fn fdjack2(&self, config: &MPConfig) {
        let eps = config.epsfcn.max(f64::EPSILON).sqrt();
        // TODO: sides are not going to be used, probably clean them up after
        // TODO: probably analytical derivatives should be implemented at some point
        for j in 0..self.nfree {
            let free_p = self.ifree[j];
            let temp = self.x[free_p];
            let mut h = eps * temp.abs();
            if self.step.len() > free_p && self.step[free_p] > 0. {
                h = self.step[free_p];
            }
            if self.dstep.len() > free_p && self.dstep[free_p] > 0. {
                h = (self.dstep[free_p] * temp).abs();
            }
            if h == 0. {
                h = eps;
            }
        }
    }
}

pub fn mpfit<T: MPFitter>(
    f: T,
    xall: &mut [f64],
    params: Option<&[MPPar]>,
    config: &MPConfig,
) -> MPResult {
    let mut fit = match MPFit::new(f.number_of_points(), xall) {
        None => return MPResult::Error(MPError::Empty),
        Some(v) => v,
    };
    match &params {
        None => {
            fit.nfree = fit.npar;
            fit.ifree = (0..fit.npar).collect();
        }
        Some(pars) => {
            if pars.len() == 0 {
                return MPResult::Error(MPError::Empty);
            }
            for (i, p) in pars.iter().enumerate() {
                if !p.fixed {
                    fit.nfree += 1;
                    fit.ifree.push(i);
                }
                fit.side.push(p.side);
                fit.step.push(p.step);
                fit.dstep.push(p.rel_step);
            }
            if fit.nfree == 0 {
                return MPResult::Error(MPError::NoFree);
            }
        }
    };
    if fit.m < fit.nfree {
        return MPResult::Error(MPError::DoF);
    }
    f.eval(fit.xall, &mut fit.fvec);
    let fnorm = fit.enorm();
    let orig_norm = fnorm * fnorm;
    fit.xnew.copy_from_slice(fit.xall);
    fit.x = Vec::with_capacity(fit.nfree);
    for i in 0..fit.nfree {
        fit.x.push(fit.xall[fit.ifree[i]]);
    }
    // Initialize Levenberg-Marquardt parameter and iteration counter
    let par = 0.0;
    let iter = 1;
    fit.qtf = vec![0.; fit.nfree];
    fit.fjack = vec![0.; fit.m * fit.nfree];
    loop {
        for i in 0..fit.nfree {
            fit.xnew[fit.ifree[i]] = fit.x[i];
        }
        // Calculate the Jacobian matrix
        fit.fdjack2(&config);
        break;
    }
    MPResult::Success(
        MPSuccess::Both,
        MPStatus {
            best_norm: 0.0,
            orig_norm: 0.0,
            n_iter: 0,
            n_fev: 0,
            n_par: 0,
            n_free: 0,
            n_pegged: 0,
            n_func: 0,
            resid: vec![],
            xerror: vec![],
            covar: vec![],
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::{mpfit, MPFitter};

    #[test]
    fn linear() {
        struct Linear {
            x: Vec<f64>,
            y: Vec<f64>,
            ye: Vec<f64>,
        };

        impl MPFitter for Linear {
            fn eval(&self, params: &[f64], deviates: &mut [f64]) {
                for (((d, x), y), ye) in deviates
                    .iter_mut()
                    .zip(self.x.iter())
                    .zip(self.y.iter())
                    .zip(self.ye.iter())
                {
                    let f = params[0] + params[1] * *x;
                    *d = (*y - f) / *ye;
                }
            }

            fn number_of_points(&self) -> usize {
                self.x.len()
            }
        }
        let l = Linear {
            x: vec![
                -1.7237128E+00,
                1.8712276E+00,
                -9.6608055E-01,
                -2.8394297E-01,
                1.3416969E+00,
                1.3757038E+00,
                -1.3703436E+00,
                4.2581975E-02,
                -1.4970151E-01,
                8.2065094E-01,
            ],
            y: vec![
                1.9000429E-01,
                6.5807428E+00,
                1.4582725E+00,
                2.7270851E+00,
                5.5969253E+00,
                5.6249280E+00,
                0.787615,
                3.2599759E+00,
                2.9771762E+00,
                4.5936475E+00,
            ],
            ye: vec![0.07; 10],
        };
        let mut init = [1., 1.];
        let _ = mpfit(l, &mut init, None, &Default::default());
    }
}
