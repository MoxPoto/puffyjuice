#![allow(non_snake_case)]
#![allow(dead_code)]

//! Contains the main base for Puffyjuice, handling things from ticking the solver to initializing the library
use crate::{
    event::EventQueue,
    params,
    particle::ParticleQueue,
    scene::Scene,
    util::{flex_buffer, flex_map},
    vec::{Vec3, Vec4},
    wrapper::solver::{FlexSolver, Solver},
};
use flexgen::*;
use rand::Rng;
use rglua::rstr;
use std::{
    mem::MaybeUninit,
    os::raw::*,
    sync::Arc,
    sync::{atomic::AtomicPtr, Mutex},
    thread,
    time::Duration,
};

// TODO: Keep this, but don't make the code rely on this as if thats the current particles,
// the user will want to spawn variable amounts of particles
const MAX_PARTICLES: c_int = 13700;
const MAX_COLLIDERS: c_int = 8192;

// hear ye hear ye
// thy code is a travesty
// wonder with caution

pub type FlexLibrary = *mut NvFlexLibrary;

// Random inline API helpers that.. aren't exported in the bindings (understandable)
fn NvFlexMakePhaseWithChannels(group: i32, particleFlags: i32, shapeChannels: i32) -> i32 {
    return (group & NvFlexPhase_eNvFlexPhaseGroupMask)
        | (particleFlags & NvFlexPhase_eNvFlexPhaseFlagsMask)
        | (shapeChannels & NvFlexPhase_eNvFlexPhaseShapeChannelMask);
}

fn NvFlexMakePhase(group: i32, particleFlags: i32) -> i32 {
    return NvFlexMakePhaseWithChannels(
        group,
        particleFlags,
        NvFlexPhase_eNvFlexPhaseShapeChannelMask,
    );
}

fn NvFlexMakeShapeFlagsWithChannels(
    type_: NvFlexCollisionShapeType,
    dynamic: bool,
    shapeChannels: c_int,
) -> i32 {
    return type_
        | (if dynamic {
            NvFlexCollisionShapeFlags_eNvFlexShapeFlagDynamic
        } else {
            0
        })
        | shapeChannels;
}

fn NvFlexMakeShapeFlags(type_: NvFlexCollisionShapeType, dynamic: bool) -> i32 {
    return NvFlexMakeShapeFlagsWithChannels(
        type_,
        dynamic,
        NvFlexPhase_eNvFlexPhaseShapeChannelMask,
    );
}

/// Holds the buffers of the library in a neat named fashion
pub struct JuiceBuffers {
    /// Holds where the particles are, along with ther inverse mass
    particles: *mut NvFlexBuffer,
    /// Holds the velocity of the particles
    velocity: *mut NvFlexBuffer,
    /// Holds the phases of the particles
    phases: *mut NvFlexBuffer,
    /// Holds the currently active particles
    actives: *mut NvFlexBuffer,

    // Geometry
    /// Holds the geometry of the collders
    geometry: *mut NvFlexBuffer,
    /// Holds the colliders positions
    geopositions: *mut NvFlexBuffer,
    /// Holds the colliders quaternions
    georotations: *mut NvFlexBuffer,
    /// Holds the colliders previous positions
    geoprevpos: *mut NvFlexBuffer,
    /// Holds the colliders previous quaternions
    geoprevrot: *mut NvFlexBuffer,
    /// Holds the colliders flags
    geoflags: *mut NvFlexBuffer,
}

unsafe impl Send for JuiceBuffers {}
unsafe impl Sync for JuiceBuffers {}

/// The main base for Puffyjuice, handling things from ticking the solver to initializing the library
pub struct Juice {
    /// A pointer to the flex library instance, lets us call flex functions
    flexlib: FlexLibrary,

    /// Buffers for various FleX related operations
    buffers: Arc<Mutex<JuiceBuffers>>,

    /// The main solver instance
    solver: Solver,

    /// Active or not, used to control the running state of the solver ticking thread
    active: Arc<Mutex<bool>>,

    /// Thread-safe pointer to a `Scene`
    scene: Arc<Mutex<Scene>>,

    /// Thread-safe event queue, these are invoked and destroyed by the solver thread
    events: Arc<Mutex<EventQueue>>,

    /// Thread-safe particle queue, used to spawn particles
    particleQueue: Arc<Mutex<ParticleQueue>>,
}

impl Juice {
    /// Handles errors coming from FleX, not us!!
    pub unsafe extern "C" fn errorHandler(
        err: NvFlexErrorSeverity,
        msg: *const c_char,
        file: *const c_char,
        line: c_int,
    ) {
        let msgRust = rstr!(msg);
        let fileRust = rstr!(file);
        println!(
            "[severity: {}] Encountered FleX error\nmsg: {}\nfile: {}\nline: {}",
            err, msgRust, fileRust, line
        );
    }

    /// Initializes the FleX-related buffers
    unsafe fn initBuffers(lib: &FlexLibrary) -> JuiceBuffers {
        let particles = flex_buffer!(*lib, Vec4, MAX_PARTICLES);
        let velocity = flex_buffer!(*lib, Vec3, MAX_PARTICLES);
        let phases = flex_buffer!(*lib, c_int, MAX_PARTICLES);
        let actives = flex_buffer!(*lib, c_int, MAX_PARTICLES);

        // Geometry
        let geometry = flex_buffer!(*lib, NvFlexCollisionGeometry, MAX_COLLIDERS);

        // Buffers that also have previous variants
        let geopositions = flex_buffer!(*lib, Vec4, MAX_COLLIDERS);
        let georotations = flex_buffer!(*lib, Vec4, MAX_COLLIDERS);
        let geoprevpos = flex_buffer!(*lib, Vec4, MAX_COLLIDERS);
        let geoprevrot = flex_buffer!(*lib, Vec4, MAX_COLLIDERS);

        let geoflags = flex_buffer!(*lib, c_int, MAX_COLLIDERS);

        JuiceBuffers {
            particles,
            velocity,
            phases,
            actives,

            geometry,

            geopositions,
            georotations,
            geoprevpos,
            geoprevrot,

            geoflags,
        }
    }

    /// Instantiates a new Juice
    pub unsafe fn new() -> Self {
        println!("Initializing FleX, this is a potentially unsafe operation, prepare");

        let nullInitPtr: *mut NvFlexInitDesc = std::ptr::null_mut();
        // FleX is fine with having a null pointer for the init struct, as ugly as the code seems
        let flex = NvFlexInit(
            NV_FLEX_VERSION.try_into().unwrap(),
            Some(Juice::errorHandler),
            nullInitPtr,
        );

        println!("Initialized.. creating solver..");

        let mut solverDesc: *mut NvFlexSolverDesc = MaybeUninit::uninit().as_mut_ptr();
        // Uninitialize so we can get the defaults
        NvFlexSetSolverDescDefaults(solverDesc);

        (*solverDesc).maxParticles = MAX_PARTICLES.try_into().unwrap();
        (*solverDesc).maxDiffuseParticles = 0;

        let solverRaw = NvFlexCreateSolver(flex, solverDesc);
        let params = params::getDefaultParams();

        let mut uninitParams: MaybeUninit<NvFlexParams> = MaybeUninit::uninit();
        uninitParams.write(params);

        NvFlexSetParams(solverRaw, uninitParams.as_ptr());

        let wrappedSolver = Solver::new(FlexSolver::new(solverRaw));

        Self {
            flexlib: flex,
            buffers: Arc::new(Mutex::new(Self::initBuffers(&flex))),
            solver: wrappedSolver,
            active: Arc::new(Mutex::new(false)),
            scene: Arc::new(Mutex::new(Scene::new())),
            events: Arc::new(Mutex::new(EventQueue::new())),
            particleQueue: Arc::new(Mutex::new(ParticleQueue::new())),
        }
    }

    pub fn startSolver(&self) {
        let solverCopy = self.solver.getForThread();
        let bufferCopy = self.buffers.clone();
        let activeCopy = self.active.clone();
        let activeCopyForThread = self.active.clone();
        let sceneCopy = self.scene.clone();
        let eventsCopy = self.events.clone();
        let particleQueueCopy = self.particleQueue.clone();
        let mut flexLibraryCopy = AtomicPtr::new(self.flexlib.clone());

        // While we're here, let's also set the solver to active
        let mut activePtr = activeCopy
            .lock()
            .expect("Couldn't unlock activeCopy (wtf?)");

        *activePtr = true;

        thread::spawn(move || {
            // Used for spawning particles randomly so they dont look like they're all in one place
            let mut rng = rand::thread_rng();
            let mut initialized = false;

            loop {
                unsafe {
                    let active = activeCopyForThread
                        .lock()
                        .expect("Couldn't unlock activeCopy (wtf?)");

                    // We dont use a while loop here because during the tick, our thread yields, allowing for the rest of the program to
                    // actually be able to take control of the mutex and also manipulate the active state

                    let solverMutex = solverCopy.lock().expect("Couldn't lock solverCopy (wtf?)");

                    if *active {
                        // We also.. you know.. need the buffers, so let's obtain a lock to them
                        let bufferMutex =
                            bufferCopy.lock().expect("Couldn't lock bufferCopy (wtf?)");
                        let mut sceneMutex =
                            sceneCopy.lock().expect("Couldn't lock sceneCopy (wtf?)");
                        let mut eventsMutex =
                            eventsCopy.lock().expect("Couldn't lock eventsCopy (wtf?)");

                        // We also need to get the particle queue
                        let mut particleQueueMutex = particleQueueCopy
                            .lock()
                            .expect("Couldn't lock particleQueueCopy (wtf?)");

                        let buffers = &*bufferMutex;
                        let solver = &*solverMutex;
                        let scene = &mut *sceneMutex;
                        let events = &mut *eventsMutex;
                        let particleQueue = &mut *particleQueueMutex;

                        let particles: *mut Vec4 = flex_map!(buffers.particles);
                        let velocity: *mut Vec3 = flex_map!(buffers.velocity);
                        let phases: *mut c_int = flex_map!(buffers.phases);
                        let actives: *mut c_int = flex_map!(buffers.actives);

                        // TODO: ... d.. do something with the buffers we map? it's expensive to do so..

                        /*
                        if !initialized {
                            for i in 0..MAX_PARTICLES.try_into().unwrap() {
                                let mut particle = particles.offset(i as isize);
                                let mut phase = phases.offset(i as isize);
                                let mut active = actives.offset(i as isize);

                                (*particle) = Vec4::components(
                                    rng.gen::<f32>() * 0.01,
                                    rng.gen::<f32>() * 0.01,
                                    rng.gen::<f32>() * 0.01,
                                    1.0 / 2.0,
                                );

                                *phase = NvFlexMakePhase(
                                    0,
                                    NvFlexPhase_eNvFlexPhaseSelfCollide
                                        | NvFlexPhase_eNvFlexPhaseFluid,
                                );

                                *active = i;
                            }

                            initialized = true;
                        }
                        */

                        if particleQueue.particleCount < MAX_PARTICLES.try_into().unwrap() {
                            // Create particles from the queue
                            for particle in &particleQueue.particles {
                                let index = particleQueue.particleCount;

                                let mut particlePtr = particles.offset(index as isize);
                                let mut phase = phases.offset(index as isize);
                                let mut active = actives.offset(index as isize);
                                let mut velocity = velocity.offset(index as isize);

                                (*particlePtr) = Vec4::components(
                                    particle.pos.x,
                                    particle.pos.y,
                                    particle.pos.z,
                                    1.0 / 2.0,
                                );

                                *phase = NvFlexMakePhase(
                                    0,
                                    NvFlexPhase_eNvFlexPhaseSelfCollide
                                        | NvFlexPhase_eNvFlexPhaseFluid,
                                );

                                *active = particleQueue.particleCount;
                                *velocity = particle.vel.clone();

                                particleQueue.particleCount += 1;
                            }

                            // Clear the particle queue
                            particleQueue.flush();
                        }

                        // Before unmapping, flush the queue
                        for event in &events.events {
                            event.invoke(
                                particles.clone(),
                                velocity.clone(),
                                particleQueue.particleCount.try_into().unwrap(),
                            );
                        }

                        events.flush();

                        NvFlexUnmap(buffers.particles);
                        NvFlexUnmap(buffers.velocity);
                        NvFlexUnmap(buffers.phases);
                        NvFlexUnmap(buffers.actives);

                        // Work on geometries next
                        // TODO: do this
                        // Map some geometric buffers we need
                        let geometry: *mut NvFlexCollisionGeometry = flex_map!(buffers.geometry);
                        let geopositions: *mut Vec4 = flex_map!(buffers.geopositions);
                        let georotations: *mut Vec4 = flex_map!(buffers.georotations);
                        let geoflags: *mut c_int = flex_map!(buffers.geoflags);

                        let geoprevpos: *mut Vec4 = flex_map!(buffers.geoprevpos);
                        let geoprevrot: *mut Vec4 = flex_map!(buffers.geoprevrot);

                        for (index, record) in (scene.objects.iter_mut().enumerate()) {
                            let mut collider = &mut record.collider;

                            if !collider.isInitialized() {
                                collider.set_library(*flexLibraryCopy.get_mut());
                                collider.initializeGeometry(index.try_into().unwrap(), geometry);
                            }

                            // Update positions.. rotations.. flags.. everything!!
                            let geoPos = geopositions.offset(index as isize);
                            let geoRot = georotations.offset(index as isize);
                            let geoFlags = geoflags.offset(index as isize);
                            let geoPrevPos = geoprevpos.offset(index as isize);
                            let geoPrevRot = geoprevrot.offset(index as isize);

                            *geoPos = Vec4::from(&collider.position());
                            *geoRot = collider.rotation();
                            *geoFlags = NvFlexMakeShapeFlags(collider.getShapeFlag(), false);
                            *geoPrevPos = Vec4::from(&collider.prev_position());
                            *geoPrevRot = collider.prev_rotation();
                        }

                        // Unmap the geometry buffers
                        NvFlexUnmap(buffers.geometry);
                        NvFlexUnmap(buffers.geopositions);
                        NvFlexUnmap(buffers.georotations);
                        NvFlexUnmap(buffers.geoflags);
                        NvFlexUnmap(buffers.geoprevpos);
                        NvFlexUnmap(buffers.geoprevrot);

                        // Now we can tick the solver.. but first let's write all our data to the gpu
                        NvFlexSetActiveCount(
                            solver.get(),
                            particleQueue.particleCount.try_into().unwrap(),
                        );
                        NvFlexSetActive(solver.get(), buffers.actives, std::ptr::null_mut());
                        NvFlexSetParticles(solver.get(), buffers.particles, std::ptr::null_mut());
                        NvFlexSetVelocities(solver.get(), buffers.velocity, std::ptr::null_mut());
                        NvFlexSetPhases(solver.get(), buffers.phases, std::ptr::null_mut());
                        NvFlexSetShapes(
                            solver.get(),
                            buffers.geometry,
                            buffers.geopositions,
                            buffers.georotations,
                            buffers.geoprevpos,
                            buffers.geoprevrot,
                            buffers.geoflags,
                            scene.len().try_into().unwrap(),
                        );

                        NvFlexUpdateSolver(solver.get(), 0.01 * 8.0, 3, false);

                        NvFlexGetParticles(solver.get(), buffers.particles, std::ptr::null_mut());
                        NvFlexGetVelocities(solver.get(), buffers.velocity, std::ptr::null_mut());
                        NvFlexGetPhases(solver.get(), buffers.phases, std::ptr::null_mut());
                    } else {
                        break;
                    }

                    // Then wait to be truthful to the `dt` argument
                    // a little tale to tell the tale
                    // once upon a time
                    // a long time ago
                    // the dt argument is the time between frames
                    // the time between frames is the time between frames
                    // but thus the dt argument is the time between frames
                    // and thus the dt argument is the time between frames
                    // so the dt was 0.03 * 8.0
                    // and thus the colliders were so unreliable
                    // that the dt was 0.03 * 8.0
                    // and thus the colliders were so unreliable
                    // that the dt was 0.03 * 8.0
                    // a simple change to 0.01 * 8.0
                    // and thus the colliders were so reliable
                    // that the dt was 0.01 * 8.0
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });

        println!("Launched solver!");
    }

    pub unsafe fn cleanup(&self) {
        println!("Destroying FleX...");
        // We need to obtain many various locks, so prepare for that
        // If you constantly see this paradigm, this is simply a way to get a pointer to our class variables
        // without affecting the rest of the program

        let solverCopy = self.solver.getForThread();
        let bufferCopy = self.buffers.clone();
        let activeCopy = self.active.clone();
        let sceneCopy = self.scene.clone();

        let mut activeMutex = activeCopy.lock().expect("Couldn't lock activeCopy (wtf?)");
        // Shut it down firstly
        *activeMutex = false;

        // Now, the program flow is programmed in a way where this is a safe spot to completely shut down the solver
        let solverMutex = solverCopy.lock().expect("Couldn't lock solverCopy (wtf?)");
        let bufferMutex = bufferCopy.lock().expect("Couldn't lock bufferCopy (wtf?)");
        let mut sceneMutex = sceneCopy.lock().expect("Couldn't lock sceneCopy (wtf?)");

        let solver = &*solverMutex;
        let bufferMutex = &*bufferMutex;
        let scene = &mut *sceneMutex;

        // Drop all the objects, they'll handle it themselves
        scene.objects.clear();

        // Remove buffers, then destroy solver
        NvFlexFreeBuffer(bufferMutex.particles);
        NvFlexFreeBuffer(bufferMutex.velocity);
        NvFlexFreeBuffer(bufferMutex.phases);
        NvFlexFreeBuffer(bufferMutex.actives);

        // Geometry
        NvFlexFreeBuffer(bufferMutex.geometry);
        NvFlexFreeBuffer(bufferMutex.geopositions);
        NvFlexFreeBuffer(bufferMutex.georotations);
        NvFlexFreeBuffer(bufferMutex.geoflags);
        NvFlexFreeBuffer(bufferMutex.geoprevpos);
        NvFlexFreeBuffer(bufferMutex.geoprevrot);

        NvFlexDestroySolver(solver.get());

        NvFlexShutdown(self.flexlib);
    }

    /// A massively abstracted utility function to get the current state of the particles
    /// this does perform mutex magic, so expect for it to block
    ///
    /// # Safety
    /// The function does perform a bit of pointer magic, so it's not safe to call this function
    /// **HOWEVER**, the chance that this is a unsafe operation is very slim, and if it were to cause a crash, then.. there
    /// is worse problems
    pub unsafe fn getPositions(&self) -> Vec<Vec3> {
        let bufferCopy = self.buffers.clone();
        let bufferMutex = bufferCopy.lock().expect("Couldn't lock bufferCopy (wtf?)");
        let buffers = &*bufferMutex;

        let particleQueueCopy = self.get_particle_queue();
        let particleQueueMutex = particleQueueCopy
            .lock()
            .expect("Couldn't lock particleQueueCopy (wtf?)");
        let particleQueue = &*particleQueueMutex;

        // Now, we have access to the buffers, so the only thing left is to
        // map the buffers meanwhile we have a critical section of processing time to map and unmap without
        // this affecting the rest of the program

        // This code is reasonable because the Juice class is primarily operations around the FleX library,
        // and puffyjuice itself is made out of the Lua interfacing

        let particles: *mut Vec4 = flex_map!(buffers.particles);

        // TODO: particles should be handled by a variable amount, but this is a constant for now
        // TODO: Fix this on Day 3 of the work plan

        let mut positions: Vec<Vec3> =
            Vec::with_capacity(particleQueue.particleCount.try_into().unwrap());

        for i in 0..particleQueue.particleCount.try_into().unwrap() {
            let particle = &*particles.offset(i as isize);
            positions.push(Vec3::components(particle.x, particle.y, particle.z));
        }

        NvFlexUnmap(buffers.particles);

        positions
    }

    /// Returns the FlexLibrary
    pub fn get_lib(&self) -> FlexLibrary {
        self.flexlib.clone()
    }

    /// Returns a `Arc<Mutex<Scene>>` to the caller, allowing for proper multithreaded access
    pub fn get_scene(&self) -> Arc<Mutex<Scene>> {
        self.scene.clone()
    }

    /// Returns a `Arc<Mutex<EventQueue>>` to the caller, allowing for proper multithreaded access
    pub fn get_event_queue(&self) -> Arc<Mutex<EventQueue>> {
        self.events.clone()
    }

    /// Returns a `Arc<Mutex<ParticleQueue>` to the caller, allowing for proper multithreaded access
    pub fn get_particle_queue(&self) -> Arc<Mutex<ParticleQueue>> {
        self.particleQueue.clone()
    }
}

unsafe impl Send for Juice {}
unsafe impl Sync for Juice {}

// The above is needed for.. you know.. Lazy static initialization
