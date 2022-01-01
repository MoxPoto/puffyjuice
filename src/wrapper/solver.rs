//! A safe wrapper around the FleX solver, mainly for multi-threading purposes

use flexgen::*;
use std::sync::Arc;
use std::sync::Mutex;

/// A safe wrapper around a `*mut NvFlexSolver`
pub struct FlexSolver {
    solver: *mut NvFlexSolver,
}

unsafe impl Send for FlexSolver {}
unsafe impl Sync for FlexSolver {}

impl FlexSolver {
    pub fn new(solver: *mut NvFlexSolver) -> Self {
        Self { solver }
    }

    pub fn get(&self) -> *mut NvFlexSolver {
		assert!(!self.solver.is_null(), "FlexSolver is null");
        self.solver
    }
}

pub struct Solver {
    solver: Arc<Mutex<FlexSolver>>,
}

impl Solver {
    /// Instantiates a `Solver` from a `*mut NvFlexSolver`
    pub unsafe fn new(solver: FlexSolver) -> Self {
        // Crucial to consume the solver

        Self {
            solver: Arc::new(Mutex::new(solver)),
        }
    }

    /// Clones the pointer to the mutex-wrapped solver, used for a thread
    pub fn getForThread(&self) -> Arc<Mutex<FlexSolver>> {
        Arc::clone(&self.solver)
    }
}
