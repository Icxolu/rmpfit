# rmpfit

<!-- cargo-rdme start -->

Very simple pure Rust implementation of the
[CMPFIT](https://pages.physics.wisc.edu/~craigm/idl/cmpfit.html) library:
the Levenberg-Marquardt technique to solve the least-squares problem.

The code is mainly copied directly from CMPFIT almost without changing. The
original CMPFIT tests (Linear (free parameters), Quad (free and fixed
parameters), and Gaussian (free and fixed parameters) function) are
reproduced and passed.

Just a few rust-specific optimizations are done:
* Error handling using Result.
* A few loops are zipped to help the compiler optimize the code (no
  performance tests are done anyway).
* Using trait `Problem` to call the user code.
* Using Enum types if possible.

# Advantages
* Pure Rust.
* No external dependencies
  ([assert_approx_eq](https://docs.rs/assert_approx_eq/) just for testing).
* Internal Jacobian calculations.

# Disadvantages
* Sided, analytical or user provided derivates are not implemented.

# Usage Example
A user should implement trait `Problem` for its struct:
```rust
use assert_approx_eq::assert_approx_eq;
use rmpfit::{Problem, MPResult, fit};

struct Linear {
    x: Vec<f64>,
    y: Vec<f64>,
    ye: Vec<f64>,
}

impl Problem<2> for Linear {
    fn eval(&self, [a, b]: &[f64; 2], deviates: &mut [f64]) -> MPResult<()> {
        for (((d, &x), &y), &ye) in deviates
            .iter_mut()
            .zip(self.x.iter())
            .zip(self.y.iter())
            .zip(self.ye.iter())
            {
                let f = a + b * x;
                *d = (y - f) / ye;
            }
        Ok(())
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
// initializing input parameters
let mut init = [1., 1.];
let status = fit(&l, &mut init, Default::default(), Default::default()).unwrap();
assert_approx_eq!(init[0], 3.20996572); // actual 3.2
assert_approx_eq!(status.xerror[0], 0.02221018);
assert_approx_eq!(init[1], 1.77095420); // actual 1.78
assert_approx_eq!(status.xerror[1], 0.01893756);
```
`init` will afterwards contain the refined parameters of the fitting
function. If the user function fails to calculate residuals, it should
return `Error::Eval`

# Note
This is a fork of
[rmpfit](https://git.3lp.cx/dyadkin/rmpfit/src/branch/master) which changes
the api to use const generics arrays for the fit parameters instead of
slices. Limits are transformed into enums representing the
state. All structs have basic traits derived, Error
implements [Error](std::error::Error)

<!-- cargo-rdme end -->
