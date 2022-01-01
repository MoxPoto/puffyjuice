/// # WARNING
/// This is primarily to be used with particles, not anything requiring 4 components
///
/// The `w` property is the inverse mass, and is used for FleX related operations (1 / mass)

#[derive(Clone, Debug)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Clone, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec4 {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0 / 2.0,
        }
    }

    pub fn components(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from(other: &Vec3) -> Self {
        Self {
            x: other.x,
            y: other.y,
            z: other.z,
            w: 1.0 / 2.0,
        }
    }

    pub fn add(left: &Vec4, other: &Vec4) -> Self {
        Self {
            x: left.x + other.x,
            y: left.y + other.y,
            z: left.z + other.z,
            w: 1.0 / 2.0,
        }
    }
}

impl Vec3 {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn components(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

pub type Quat = Vec4;
