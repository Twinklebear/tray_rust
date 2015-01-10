//! Provides the Fresnel term trait and implementations for conductors and dielectric materials

use std::num::Float;

use film::Colorf;
use linalg;

/// Compute the Fresnel term for a dielectric material given the incident and transmission
/// angles and refractive indices
fn dielectric(cos_i: f32, cos_t: f32, eta_i: f32, eta_t: f32) -> Colorf {
	let r_par = (eta_t * cos_i - eta_i * cos_t) / (eta_t * cos_i + eta_i * cos_t);
	let r_perp = (eta_i * cos_i - eta_t * cos_t) / (eta_i * cos_i + eta_t * cos_t);
	Colorf::broadcast((r_par * r_par + r_perp * r_perp) * 0.5)
}
/// Compute the Fresnel term for a conductor given the incident angle and the material properties
fn conductor(cos_i: f32, eta: &Colorf, k: &Colorf) -> Colorf {
	let mut a = (*eta * *eta + *k * *k) * cos_i * cos_i;
    let white = Colorf::broadcast(1.0);
	let r_par = (a - *eta * cos_i * 2.0 + white) / (a + *eta * cos_i * 2.0 + white);
	a = *eta * *eta + *k * *k;
    let cos_col = Colorf::broadcast(cos_i * cos_i);
	let r_perp = (a - *eta * cos_i * 2.0 + cos_col) / (a + *eta * cos_i * 2.0 + cos_col);
	//These are actually r_par^2 and r_perp^2, so don't square here
	(r_par + r_perp) * 0.5
}

/// The Fresnel trait implemented by the various Fresnel term components
pub trait Fresnel {
    /// Compute the fresnel term for light incident to the object at angle `cos_i`
    fn fresnel(&self, cos_i: f32) -> Colorf;
}

/// Computes the Fresnel term for dielectric materials
pub struct Dielectric {
    /// Refractive index of the material the light is coming from
    pub eta_i: f32,
    /// Refractive index of the material the light is hitting/entering
    pub eta_t: f32,
}

impl Dielectric {
    /// Create a new Dielectric Fresnel term for the boundary between two objects.
    /// `eta_i`: refractive index of the material the light is coming from.
    /// `eta_t`: refractive index of the material the light is entering.
    pub fn new(eta_i: f32, eta_t: f32) -> Dielectric { Dielectric { eta_i: eta_i, eta_t: eta_t } }
}

impl Fresnel for Dielectric {
    fn fresnel(&self, cos_i: f32) -> Colorf {
        // We need to find out which side of the material we're incident on so
        // we can pass the correct indices of refraction
        let ci = linalg::clamp(cos_i, -1.0, 1.0);
        let ei = if ci > 0.0 { self.eta_i } else { self.eta_t };
        let et = if ci > 0.0 { self.eta_t } else { self.eta_i };
        let sin_t = ei / et * Float::sqrt(Float::max(0.0, 1.0 - ci * ci));
        //Handle total internal reflection
        if sin_t >= 1.0 {
            Colorf::broadcast(1.0)
        } else {
            let ct = Float::sqrt(Float::max(0.0, 1.0 - sin_t * sin_t));
            dielectric(Float::abs(ci), ct, ei, et)
        }
    }
}

/// Computes the Fresnel term for conductive materials
pub struct Conductor {
    /// Refractive index of the material being hit
    pub eta: Colorf,
    /// Absorption coefficient of the material being hit
    pub k: Colorf,
}

impl Conductor {
    /// Create a new Conductor Fresnel term for the object.
    /// `eta`: refractive index of the material.
    /// `k`: absorption coefficient of the material.
    pub fn new(eta: &Colorf, k: &Colorf) -> Conductor { Conductor { eta: *eta, k: *k } }
}

impl Fresnel for Conductor {
    fn fresnel(&self, cos_i: f32) -> Colorf { conductor(Float::abs(cos_i), &self.eta, &self.k) }
}

