//! A `Scene` describes every object that the particle system will be able to interact with

use crate::collider::Collider;
use rand::Rng;

pub struct SceneRecord {
    pub id: i32,
    pub collider: Box<dyn Collider>,
}
pub struct Scene {
    pub objects: Vec<SceneRecord>,
}

impl Scene {
    /// Creates a new scene
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Checks if an ID is valid
    pub fn isValid(&self, index: i32) -> bool {
        for collider in &self.objects {
            if collider.id == index {
                return true;
            }
        }

        false
    }

    /// Adds a new collider to the scene, returns a index
    pub fn add(&mut self, collider: Box<dyn Collider>) -> i32 {
        let mut rng = rand::thread_rng();
        let generatedID = rng.gen_range(1..std::i32::MAX);

        let record = SceneRecord {
            id: generatedID.clone(),
            collider: collider,
        };

        self.objects.push(record);
        generatedID
    }

    /// Gets a collider from the scene
    pub fn get(&mut self, colliderIndex: i32) -> Option<&mut Box<dyn Collider>> {
        if self.isValid(colliderIndex) {
            for record in self.objects.iter_mut() {
                if record.id == colliderIndex {
                    return Some(&mut record.collider);
                }
            }

            None
        } else {
            None
        }
    }

    /// Removes a collider from the scene
    pub fn remove(&mut self, collider: i32) {
        for (idx, record) in self.objects.iter().enumerate() {
            if record.id == collider {
                self.objects.swap_remove(idx);
                return;
            }
        }
    }

    /// Returns the number of colliders in the scene
    pub fn len(&self) -> usize {
        self.objects.len()
    }
}

unsafe impl Send for Scene {}
unsafe impl Sync for Scene {}
