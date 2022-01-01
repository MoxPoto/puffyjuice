//! A capsule collider, this is PERFECT and intended to be used with players

use crate::{
    collider::Collider,
    vec::{Quat, Vec3},
};

use flexgen::*;

pub struct Capsule {
    pub radius: f32,
    pub halfheight: f32,

    position: Vec3,
    rotation: Quat,

    prev_position: Vec3,
    prev_rotation: Quat,

    initialized: bool,
}

impl Collider for Capsule {
    fn position(&self) -> Vec3 {
        self.position.clone()
    }

    fn rotation(&self) -> Quat {
        self.rotation.clone()
    }

    fn prev_position(&self) -> Vec3 {
        self.prev_position.clone()
    }

    fn prev_rotation(&self) -> Quat {
        self.prev_rotation.clone()
    }

    fn setPosition(&mut self, pos: Vec3) {
        self.prev_position = self.position.clone();
        self.position = pos;
    }

    fn setRotation(&mut self, rot: Quat) {
        self.prev_rotation = self.rotation.clone();
        self.rotation = rot;
    }

    fn set_library(&mut self, _library: *mut flexgen::NvFlexLibrary) {
        // Not required for this capsule collider, no independent data is allocated
    }

    fn getShapeFlag(&self) -> NvFlexCollisionShapeType {
        NvFlexCollisionShapeType_eNvFlexShapeCapsule
    }

    unsafe fn initializeGeometry(
        &mut self,
        idx: i32,
        geometryBuffer: *mut NvFlexCollisionGeometry,
    ) {
        let geometry = geometryBuffer.offset(idx as isize);

        (*geometry).capsule.radius = self.radius;
        (*geometry).capsule.halfHeight = self.halfheight;

        self.initialized = true;
    }

    fn isInitialized(&self) -> bool {
        self.initialized
    }
}

impl Capsule {
    pub fn new(radius: f32, halfheight: f32) -> Self {
        // Nothing special here
        Self {
            radius,
            halfheight,
            initialized: false,
            position: Vec3::new(),
            rotation: Quat::new(),
            prev_position: Vec3::new(),
            prev_rotation: Quat::new(),
        }
    }
}
