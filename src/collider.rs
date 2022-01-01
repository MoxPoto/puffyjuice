//! This handles collisions, and also supplies a stock of collision shapes that are programmed to be used with the solver
use crate::vec::{Quat, Vec3};
use flexgen::*;

pub mod capsule;
pub mod mesh;

pub trait Collider {
    /// The position of the collider
    fn position(&self) -> Vec3;
    /// The rotation of the collider
    fn rotation(&self) -> Quat;

    /// The previous position of the collider
    fn prev_position(&self) -> Vec3;
    /// The previous rotation of the collider
    fn prev_rotation(&self) -> Quat;

    /// Set the position of the collider
    fn setPosition(&mut self, pos: Vec3);
    /// Set the rotation of the collider
    fn setRotation(&mut self, rot: Quat);

    /// Returns a boolean indicating if the collider has been initialized
    fn isInitialized(&self) -> bool;

    /// Get the shape flag for the collider
    fn getShapeFlag(&self) -> NvFlexCollisionShapeType;
    /// This function initializes the geometry buffer for the specific `Collider`
    /// This is required because some colliders have special properties, such as a mesh collider
    unsafe fn initializeGeometry(&mut self, idx: i32, geometryBuffer: *mut NvFlexCollisionGeometry);

    /// Lets the collider have access to FleX functions
    fn set_library(&mut self, library: *mut NvFlexLibrary);
}
