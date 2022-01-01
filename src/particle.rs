//! Has things relating to particles, most notably the `Particle` struct and `ParticleQueue`

use crate::vec::Vec3;

/// A particle is a blueprint for a FleX particle, usually in a `ParticleQueue`
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
}

/// A queue of particles, used to create FleX particles
/// This should usually only be used with the `JUICE_SINGLETON`
pub struct ParticleQueue {
    /// Used for keeping track of the active particles in the solver
    pub particleCount: i32,
    /// Used for queuing up a `Particle` to be added to the solvers
    pub particles: Vec<Particle>,
}

impl ParticleQueue {
    /// Instantiates a new particle queue
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            particleCount: 0,
        }
    }

    /// Adds a particle to the queue
    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    /// Flushes the particle queue
    pub fn flush(&mut self) {
        self.particles.clear();
    }
}
