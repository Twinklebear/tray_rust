//! The film module provides color types and a render target that the image
//! is written too. Functions are also provided for saving PPM and BMP images
//! while I wait to migrate over to the Piston image library due to some
//! compile issues TODO: https://github.com/PistonDevelopers/image

pub use self::color::{Colorf, Color24};

pub mod color;

