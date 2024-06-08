use nalgebra::{SVector, Unit};
use reflect::*;
use std::error::Error;

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};
use core::ops::Deref;

pub use serde_json;

/// This is essentially `try_into` then `try_map` but the latter is nightly-only
pub fn json_array_to_float_array<const D: usize>(
    json_array: &[serde_json::Value],
) -> Option<[Float; D]> {
    let array: &[serde_json::Value; D] = json_array.try_into().ok()?;

    let mut center_coords_array = [0.; D];
    for (coord, value) in center_coords_array.iter_mut().zip(array) {
        *coord = value.as_f64()? as Float;
    }
    Some(center_coords_array)
}

pub fn json_array_to_vector<const D: usize>(
    json_array: &[serde_json::Value],
) -> Option<SVector<Float, D>> {
    json_array_to_float_array(json_array).map(SVector::from)
}

pub fn map_json_array<C: FromIterator<T>, T>(
    json: &serde_json::Value,
    map: impl FnMut(&serde_json::Value) -> Result<T, Box<dyn Error>>,
) -> Result<C, Box<dyn Error>> {
    json.as_array()
        .ok_or("json value must be an array")?
        .iter()
        .map(map)
        .collect()
}

pub trait JsonType {
    /// Returns a string, unique to the type, found in the "type" field of the json
    /// representation of a "dynamic" mirror containing a mirror of this type
    fn json_type() -> String;
}

impl<T: JsonType> JsonType for [T] {
    fn json_type() -> String {
        format!("[]{}", T::json_type())
    }
}

pub trait JsonSer {
    /// Serialize `self` into a JSON object.
    fn to_json(&self) -> serde_json::Value;
}

impl<T: JsonSer> JsonSer for [T] {
    fn to_json(&self) -> serde_json::Value {
        serde_json::Value::Array(Vec::from_iter(self.iter().map(T::to_json)))
    }
}

impl<const N: usize, T: JsonSer> JsonSer for [T; N] {
    fn to_json(&self) -> serde_json::Value {
        self.as_slice().to_json()
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all
// types implementing `Deref` makes the trait unusable downstream

impl<T: JsonSer + ?Sized> JsonSer for Box<T> {
    fn to_json(&self) -> serde_json::Value {
        self.deref().to_json()
    }
}

impl<T: JsonSer + ?Sized> JsonSer for Arc<T> {
    fn to_json(&self) -> serde_json::Value {
        self.deref().to_json()
    }
}

impl<T: JsonSer + ?Sized> JsonSer for Rc<T> {
    fn to_json(&self) -> serde_json::Value {
        self.deref().to_json()
    }
}

impl<T: JsonSer> JsonSer for Vec<T> {
    fn to_json(&self) -> serde_json::Value {
        self.deref().to_json()
    }
}

impl<'a, T: JsonSer + ?Sized> JsonSer for &'a T {
    fn to_json(&self) -> serde_json::Value {
        (*self).to_json()
    }
}

impl<'a, T: JsonSer + ?Sized> JsonSer for &'a mut T {
    fn to_json(&self) -> serde_json::Value {
        self.deref().to_json()
    }
}

impl<const D: usize> JsonSer for Ray<D> {
    /// Serialize a ray into a JSON object.
    ///
    /// The format of the returned object is explained in [`Self::from_json`]
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "origin": self.origin.as_slice(),
            "direction": self.dir.as_ref().as_slice(),
        })
    }
}

pub trait JsonDes {
    /// Deserialize from a JSON object.
    ///
    /// Returns an error if `json`'s format or values are invalid.
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

impl<const D: usize> JsonDes for Ray<D> {
    /// Deserialize a new ray from a JSON object.
    ///
    /// The JSON object must follow the following format:
    ///
    /// ```json
    /// {
    ///     "origin": [9., 8., 7., ...], // (an array of D floats)
    ///     "direction": [9., 8., 7., ...], // (an array of D floats, must have at least one non-zero value)
    /// }
    /// ```
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        let origin = json
            .get("origin")
            .and_then(serde_json::Value::as_array)
            .ok_or("Missing ray origin")?;

        let direction = json
            .get("direction")
            .and_then(serde_json::Value::as_array)
            .ok_or("Missing ray direction")?;

        let origin = json_array_to_vector(origin).ok_or("Invalid ray origin")?;

        let direction = json_array_to_vector(direction).ok_or("Invalid ray direction")?;

        let direction =
            Unit::try_new(direction, Float::EPSILON).ok_or("Unable to normalize ray direction")?;

        Ok(Self {
            origin,
            dir: direction,
        })
    }
}

impl<T: JsonDes> JsonDes for Vec<T> {
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        map_json_array(json, T::from_json)
    }
}

pub fn serialize_simulation<const D: usize>(
    mirror: &(impl JsonSer + ?Sized),
    rays: impl IntoIterator<Item = Ray<D>>,
) -> serde_json::Value {
    serde_json::json!({
        "dim": D,
        "mirror": mirror.to_json(),
        "rays": Vec::from_iter(rays.into_iter().map(|ray| ray.to_json())),
    })
}

pub fn deserialize_simulation<const D: usize, M: JsonDes>(
    json: &serde_json::Value,
) -> Result<(M, Vec<Ray<D>>), Box<dyn Error>> {
    let dim = json
        .get("dim")
        .ok_or("dim field expected")?
        .as_u64()
        .ok_or("dim field must be a positive integer")? as usize;
    if dim != D {
        return Err(format!("dimension must be {D}").into());
    }
    Ok((
        M::from_json(json.get("mirror").ok_or("mirror field expected")?)?,
        map_json_array(
            json.get("rays").ok_or("ray field expected")?,
            Ray::from_json,
        )?,
    ))
}
