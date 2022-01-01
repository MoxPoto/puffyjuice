//! A Mesh collider
//!
//! # Meshes
//! The user of the Mesh collider should provide a mesh, and the Mesh collider will handle the rest, but do note that
//! it is responsible to reuse mesh data
use crate::{
    util::{flex_buffer, flex_map},
    vec::{Quat, Vec3, Vec4},
};
use flexgen::*;

use super::Collider;

pub struct Mesh {
    pub position: Vec3,
    pub rotation: Quat,
    prev_position: Vec3,
    prev_rotation: Quat,

    pub mesh: NvFlexTriangleMeshId,

    verts: *mut NvFlexBuffer,
    indices: *mut NvFlexBuffer,

    initialized: bool,
    lib: *mut NvFlexLibrary,
}

impl Collider for Mesh {
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

    fn getShapeFlag(&self) -> NvFlexCollisionShapeType {
        NvFlexCollisionShapeType_eNvFlexShapeTriangleMesh
    }

    fn isInitialized(&self) -> bool {
        self.initialized
    }

    fn set_library(&mut self, library: *mut NvFlexLibrary) {
        self.lib = library;
    }

    unsafe fn initializeGeometry(
        &mut self,
        idx: i32,
        geometryBuffer: *mut NvFlexCollisionGeometry,
    ) {
        let geometry = geometryBuffer.offset(idx as isize);
        (*geometry).triMesh.mesh = self.mesh;
        (*geometry).triMesh.scale[0] = 1.0;
        (*geometry).triMesh.scale[1] = 1.0;
        (*geometry).triMesh.scale[2] = 1.0;

        // We've been initialized!!
        self.initialized = true;
    }
}

impl Mesh {
    pub unsafe fn new(
        lib: *mut NvFlexLibrary,
        vertices: Vec<Vec4>,
        lower: Vec3,
        upper: Vec3,
    ) -> Self {
        // BEGIN THE OPERATION OF THE MESH CREATION
        // Create the various buffers

        let verticesBuffer = flex_buffer!(lib, Vec4, vertices.len() as i32);
        let indicesBuffer = flex_buffer!(lib, i32, vertices.len() as i32);

        // MAP THE MOTHAFUCKA

        let verticesPtr: *mut Vec4 = flex_map!(verticesBuffer);
        let indicesPtr: *mut i32 = flex_map!(indicesBuffer);

        // We have no indices so the indices will be very fucking stupid
        // Fill in the vertices

        for (i, v) in vertices.iter().enumerate() {
            *verticesPtr.offset(i as isize) = v.clone();
        }

        // Fill in the indices
        for i in 0..vertices.len() {
            *indicesPtr.offset(i as isize) = i as i32;
        }

        // Unmap the buffers
        NvFlexUnmap(verticesBuffer);
        NvFlexUnmap(indicesBuffer);

        // FINALLY, now we can create a mesh object
        let meshId = NvFlexCreateTriangleMesh(lib);
        // Use all of our data to write to the meshId
        // actually.. hold on
        // a stupid issue occurs here
        // The function wants the lower and upper values as a float array.. so.. we're gonna have to do pretty ugly foolery

        let mut lower_f32: [f32; 3] = [lower.x, lower.y, lower.z];
        let mut upper_f32: [f32; 3] = [upper.x, upper.y, upper.z];

        // Get a pointer to them.. man this is REALLY ugly Rust
        let lower_ptr: *mut f32 = lower_f32.as_mut_ptr();
        let upper_ptr: *mut f32 = upper_f32.as_mut_ptr();

        NvFlexUpdateTriangleMesh(
            lib,
            meshId,
            verticesBuffer,
            indicesBuffer,
            vertices.len().try_into().unwrap(),
            (vertices.len() / 3).try_into().unwrap(),
            lower_ptr,
            upper_ptr,
        );

        // Clear the vertices and indices

        Self {
            position: Vec3::new(),
            rotation: Quat::new(),
            prev_position: Vec3::new(),
            prev_rotation: Quat::new(),

            mesh: meshId,
            initialized: false,
            lib: std::ptr::null_mut(),

            verts: verticesBuffer,
            indices: indicesBuffer,
        }
    }
}

// Drop
impl Drop for Mesh {
    fn drop(&mut self) {
        if self.lib.is_null() {
            println!("MEMORY LEAK! (Mesh): self.lib is a null ptr, meaning this was never properly set..? Cannot free memory");
        } else {
            unsafe {
                NvFlexFreeBuffer(self.verts);
                NvFlexFreeBuffer(self.indices);
                NvFlexDestroyTriangleMesh(self.lib, self.mesh);
            }

            println!("Properly cleaned up (Mesh)");
        }
    }
}
