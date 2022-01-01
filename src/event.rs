//! Events are defined in Rust, and invoked via Lua
//!
//! Events allow for lua to manipulate FleX particles and other buffers

use crate::vec::{Vec3, Vec4};

pub mod setparticle;
pub trait Event {
    /// Allows an event to be invoked with particle data
    /// Will eventually have more avaliable buffers to manipulate
    unsafe fn invoke(&self, particles: *mut Vec4, velocities: *mut Vec3, num_particles: i32);
}

pub struct EventQueue {
    pub events: Vec<Box<dyn Event>>,
}

impl EventQueue {
    /// Instantiates a new event queue
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Adds an event to the queue (consumes the event)
    pub fn add_event(&mut self, event: Box<dyn Event>) {
        self.events.push(event);
    }

    /// Flushes the event queue **(INTERNAL)**
    pub fn flush(&mut self) {
        self.events.clear();
    }
}

unsafe impl Send for EventQueue {}
unsafe impl Sync for EventQueue {}
