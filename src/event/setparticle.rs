//! An event to set the position of every single particle

use super::Event;
use crate::vec::{Vec3, Vec4};
use rand::Rng;

pub struct SetParticleEvent {
    pub position: Vec4,
}

impl Event for SetParticleEvent {
    unsafe fn invoke(&self, particles: *mut Vec4, velocities: *mut Vec3, num_particles: i32) {
        let mut rng = rand::thread_rng();
        for i in 0..num_particles {
            let mut particle = particles.offset(i as isize);
            let mut velocity = velocities.offset(i as isize);
            let random_velocity: Vec3 = Vec3::components(rng.gen(), rng.gen(), rng.gen());
            *particle = self.position.clone();
            *velocity = random_velocity; // Reset velocity since.. testing proves they get set to the wanted position,
                                         // but that doesn't help when a particle is going fucking 7000 miles per second
        }
    }
}
