//! Provides a Gaussian reconstruction filter

use std::f32;

use film::filter::Filter;

pub struct Gaussian {
    w: f32,
    h: f32,
    inv_w: f32,
    inv_h: f32,
    alpha: f32,
    exp_x: f32,
    exp_y: f32
}

impl Gaussian {
    pub fn new(w: f32, h: f32, alpha: f32) -> Gaussian {
        Gaussian { w: w, h: h, inv_w: 1.0 / w, inv_h: 1.0 / h,
            alpha: alpha, exp_x: f32::exp(-alpha * w * w),
            exp_y: f32::exp(-alpha * h * h)
        }
    }
    fn gaussian_1d(&self, x: f32, e: f32) -> f32 {
        f32::max(0.0, f32::exp(-self.alpha * x * x) - e)
    }
}

impl Filter for Gaussian {
    fn weight(&self, x: f32, y: f32) -> f32 {
        self.gaussian_1d(x, self.exp_x) * self.gaussian_1d(y, self.exp_y)
    }
}

