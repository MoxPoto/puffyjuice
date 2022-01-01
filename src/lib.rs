#![allow(non_snake_case)]
use std::mem::MaybeUninit;

use event::setparticle::SetParticleEvent;
use flexgen::*;
use rglua::{prelude::*, userdata::Vector};

pub mod util;

pub mod wrapper {
    pub mod solver;
}

pub mod collider;
pub mod event;
pub mod params;
pub mod particle;
pub mod scene;
pub mod vec;

mod juice;

use juice::Juice;

use once_cell::sync::Lazy;
use vec::{Vec3, Vec4};

use crate::{
    collider::{capsule::Capsule, mesh::Mesh},
    particle::Particle,
    vec::Quat,
};

static JUICE_SINGLETON: Lazy<Juice> = Lazy::new(|| {
    let juice = unsafe { Juice::new() };
    juice
});

#[lua_function]
fn getParticlePositions(state: LuaState) -> Result<i32, std::io::Error> {
    // The unsafe keyword is mildly surprising here, but it's necessary to
    // properly understand the pointers that we get when we want the positions
    unsafe {
        let particles = JUICE_SINGLETON.getPositions();
        lua_createtable(state, particles.len() as i32, 0);

        for (i, p) in particles.iter().enumerate() {
            lua_pushinteger(state, i as isize + 1);
            // Due to some Lua C API oddities, a userdata "works," but lua cannot use it.. at all
            // so a unfavorable solution is to create a new table with x, y, z indices

            lua_createtable(state, 0, 3);
            lua_pushnumber(state, p.x.into());
            lua_setfield(state, -2, cstr!("x"));

            lua_pushnumber(state, p.y.into());
            lua_setfield(state, -2, cstr!("y"));

            lua_pushnumber(state, p.z.into());
            lua_setfield(state, -2, cstr!("z"));

            lua_settable(state, -3);
        }
    }

    Ok(1)
}

/// Fetches a number from a field in a table on the top of the stack
/// # Warning
/// This is to be only primarily used within the collider lua interface code!!
macro_rules! getTableNumber {
    ($state:expr, $varName:ident, $key:expr) => {
        lua_getfield($state, -1, cstr!($key));
        let $varName = lua_tonumber($state, -1) as f32;
        lua_pop($state, 1);
    };
}
// Mesh related functions
#[lua_function]
fn createCollider(state: LuaState) -> Result<i32, std::io::Error> {
    // We expect arguments like this: table (mesh vertices), lower bound, upper bound (each tables with x,y,z)
    // Consume from the top of the stack, upper bound first
    getTableNumber!(state, upper_bound_x, "x");
    getTableNumber!(state, upper_bound_y, "y");
    getTableNumber!(state, upper_bound_z, "z");
    // Pop the upper bound off
    lua_pop(state, 1);

    // Now consume the lower bound
    getTableNumber!(state, lower_bound_x, "x");
    getTableNumber!(state, lower_bound_y, "y");
    getTableNumber!(state, lower_bound_z, "z");
    // Pop the lower bound off
    lua_pop(state, 1);

    let upper_bound = Vec3::components(upper_bound_x, upper_bound_y, upper_bound_z);
    let lower_bound = Vec3::components(lower_bound_x, lower_bound_y, lower_bound_z);

    // Now we expect a table of vertices
    let tableLength = lua_objlen(state, -1);
    let mut vertices: Vec<Vec4> = Vec::with_capacity(tableLength as usize);

    for i in 0..tableLength {
        // Lua indices go 1, 2, 3, ...
        // unlike normal indices, which are 0, 1, 2, ...
        let real_index = i + 1;
        // Vertices are also tables with {x, y, z} (to excuse for the lack of Vector userdata support currently in crate rglua)
        lua_pushnumber(state, real_index as f64);
        lua_gettable(state, -2);

        // The vertex is now at the top of the stack, so we can do our normal thing here
        getTableNumber!(state, vert_x, "x");
        getTableNumber!(state, vert_y, "y");
        getTableNumber!(state, vert_z, "z");

        // Pop the vertex off the stack
        lua_pop(state, 1);

        // Push the vertex into the vector
        vertices.push(Vec4::components(vert_x, vert_y, vert_z, 1.0 / 2.0));
        // Rinse and repeat
    }

    // Pop the table off the stack
    lua_pop(state, 1);
    // We have all our data now, it's time to instantiate a Boxed Mesh Collider, insert it into the singleton's scene, and return the index
    printgm!(
        state,
        "Entering unsafe Rust territory, a crash is unlikely but prepare.."
    );

    let collider = unsafe {
        Box::new(Mesh::new(
            JUICE_SINGLETON.get_lib(),
            vertices,
            lower_bound,
            upper_bound,
        ))
    };

    let scenePtr = JUICE_SINGLETON.get_scene();
    // Block while waiting for access to the mutex
    let mut sceneLock = scenePtr.lock().expect("Could not lock scene (wtf?)");
    let sceneObject = &mut *sceneLock;

    let idx = sceneObject.add(collider);

    // Finally, finished!!
    // TODO: Make sure to properly typecheck this function

    // Return the index of the collider
    lua_pushnumber(state, idx as f64);
    Ok(1)
}

// Collider position & rotation functions
#[lua_function]
fn setColliderPos(state: LuaState) -> Result<i32, std::io::Error> {
    // We expect arguments like this: collider index, x, y, z
    let collider_idx = lua_tonumber(state, -4) as usize;
    let x = lua_tonumber(state, -3) as f32;
    let y = lua_tonumber(state, -2) as f32;
    let z = lua_tonumber(state, -1) as f32;

    let scenePtr = JUICE_SINGLETON.get_scene();
    // Block while waiting for access to the mutex
    let mut sceneLock = scenePtr.lock().expect("Could not lock scene (wtf?)");
    let sceneObject = &mut *sceneLock;

    let collider = sceneObject.get(collider_idx.try_into().unwrap());
    if let Some(collider) = collider {
        collider.setPosition(Vec3::components(x, y, z));
    } else {
        printgm!(state, "Could not find collider with index {}", collider_idx);
    }

    // Finally, finished!!
    Ok(0)
}

#[lua_function]
fn setColliderRot(state: LuaState) -> Result<i32, std::io::Error> {
    // We expect the arguments like this: collider index, x, y, z, w
    let collider_idx = lua_tonumber(state, -5) as usize;
    let x = lua_tonumber(state, -4) as f32;
    let y = lua_tonumber(state, -3) as f32;
    let z = lua_tonumber(state, -2) as f32;
    let w = lua_tonumber(state, -1) as f32;

    let scenePtr = JUICE_SINGLETON.get_scene();
    // Block while waiting for access to the mutex
    let mut sceneLock = scenePtr.lock().expect("Could not lock scene (wtf?)");
    let sceneObject = &mut *sceneLock;

    let collider = sceneObject.get(collider_idx.try_into().unwrap());
    if let Some(collider) = collider {
        collider.setRotation(Quat::components(x, y, z, w));
    } else {
        printgm!(state, "Could not find collider with index {}", collider_idx);
    }

    Ok(0)
}

#[lua_function]
fn removeCollider(state: LuaState) -> Result<i32, std::io::Error> {
    let collider_idx = lua_tonumber(state, -1) as usize;

    let scenePtr = JUICE_SINGLETON.get_scene();
    // Block while waiting for access to the mutex
    let mut sceneLock = scenePtr.lock().expect("Could not lock scene (wtf?)");
    let sceneObject = &mut *sceneLock;

    if sceneObject.isValid(collider_idx.try_into().unwrap()) {
        sceneObject.remove(collider_idx.try_into().unwrap());
    } else {
        printgm!(
            state,
            "Could not find collider with index {}, len: {}",
            collider_idx,
            sceneObject.len()
        );
    }

    Ok(0)
}

// Collider-specific related down here
#[lua_function]
fn spawnPlayerCollider(state: LuaState) -> Result<i32, std::io::Error> {
    let collider = unsafe { Box::new(Capsule::new(12.0, 10.0)) };

    let scenePtr = JUICE_SINGLETON.get_scene();
    // Block while waiting for access to the mutex
    let mut sceneLock = scenePtr.lock().expect("Could not lock scene (wtf?)");
    let sceneObject = &mut *sceneLock;

    let idx = sceneObject.add(collider);

    // Finally, finished!!
    // Return the index of the collider
    lua_pushnumber(state, idx as f64);
    Ok(1)
}

#[lua_function]
fn setParticles(state: LuaState) -> Result<i32, std::io::Error> {
    getTableNumber!(state, pos_x, "x");
    getTableNumber!(state, pos_y, "y");
    getTableNumber!(state, pos_z, "z");

    // Pop off the position
    lua_pop(state, 1);

    let particlePos = Vec4::components(pos_x, pos_y, pos_z, 1.0 / 2.0);
    let particleEvent = Box::new(SetParticleEvent {
        position: particlePos,
    });

    let eventPtr = JUICE_SINGLETON.get_event_queue();
    // Block while waiting for access to the mutex
    let mut eventLock = eventPtr.lock().expect("Could not lock event queue (wtf?)");
    let eventObject = &mut *eventLock;
    eventObject.add_event(particleEvent);
    Ok(0)
}

#[lua_function]
fn addParticles(state: LuaState) -> Result<i32, std::io::Error> {
    // We expect a table, that.. contains tables
    // and the table inside of the table is a lua particle struct, simply having 2 members:
    // pos and vel, both vectors

    // We want a table because if we were to perhaps, make a particle cube or sphere,
    // we'd have to invoke this function THOUSANDS of times, and we'd have to do it
    // in a loop, so.. we'll just do it in a table

    // Get the particle queue pointer
    let particlePtr = JUICE_SINGLETON.get_particle_queue();
    // Block while waiting for access to the mutex
    let mut particleLock = particlePtr
        .lock()
        .expect("Could not lock particle queue (wtf?)");
    let particleObject = &mut *particleLock;

    let tableLength = lua_objlen(state, -1);
    let mut particles: Vec<Particle> = Vec::with_capacity(tableLength as usize);

    for i in 0..tableLength {
        // Lua indices go 1, 2, 3, ...
        // unlike normal indices, which are 0, 1, 2, ...
        let real_index = i + 1;
        lua_pushnumber(state, real_index as f64);
        lua_gettable(state, -2);

        // The particle structure is at the top of the stack, lets push it and then pop it, for pos and vel
        lua_getfield(state, -1, cstr!("pos"));
        getTableNumber!(state, pos_x, "x");
        getTableNumber!(state, pos_y, "y");
        getTableNumber!(state, pos_z, "z");
        // Pop off the pos
        lua_pop(state, 1);
        // Push the vel
        lua_getfield(state, -1, cstr!("vel"));
        getTableNumber!(state, vel_x, "x");
        getTableNumber!(state, vel_y, "y");
        getTableNumber!(state, vel_z, "z");
        // Pop off the vel, and while we're at it, additionally pop the particle struct
        lua_pop(state, 2);

        // Construct our Particle
        let particle = Particle {
            pos: Vec3::components(pos_x, pos_y, pos_z),
            vel: Vec3::components(vel_x, vel_y, vel_z),
        };

        particles.push(particle);
    }

    for particle in particles {
        particleObject.add_particle(particle);
    }

    Ok(0)
}

#[lua_function]
fn clearParticles(_state: LuaState) -> Result<i32, std::io::Error> {
    let particlePtr = JUICE_SINGLETON.get_particle_queue();
    // Block while waiting for access to the mutex
    let mut particleLock = particlePtr
        .lock()
        .expect("Could not lock particle queue (wtf?)");
    let particleObject = &mut *particleLock;
    particleObject.particleCount = 0;

    Ok(0)
}
#[gmod_open]
fn entry(state: LuaState) -> Result<i32, std::io::Error> {
    // We don't push objects to lua, so return 0 (# of returns)
    unsafe {
        winapi::um::consoleapi::AllocConsole();
    }

    println!("Starting solver...");
    JUICE_SINGLETON.startSolver();

    // Register the functions
    let juiceLib = reg! [
        "GetParticlePos" => getParticlePositions,
        "CreateCollider" => createCollider,
        "CreatePlayerCollider" => spawnPlayerCollider,
        "SetColliderPos" => setColliderPos,
        "SetColliderRot" => setColliderRot,
        "RemoveCollider" => removeCollider,
        "SetParticles" => setParticles,
        "AddParticles" => addParticles,
        "ClearParticles" => clearParticles
    ];

    // Register the library
    // TODO: Figure out if there is a regression with luaL_register not running twice on a modded lua state (unloaded manually)
    // or if there is misusage with my code
    luaL_register(state, cstr!("Juice"), juiceLib.as_ptr());

    Ok(0)
}

#[gmod_close]
fn exit(state: LuaState) -> Result<i32, std::io::Error> {
    lua_pushnil(state); // Set nil to our lua library
    lua_setglobal(state, cstr!("Juice"));

    unsafe {
        JUICE_SINGLETON.cleanup();
        winapi::um::wincon::FreeConsole();
    }

    Ok(0)
}
