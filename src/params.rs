//! This is to control the parameters of the solver

use std::mem::MaybeUninit;

use flexgen::*;

// Original C++ code:
/*
    params->gravity[0] = 0.0f;
    params->gravity[1] = 0.0f;
    params->gravity[2] = -10.0f;

    params->wind[0] = 0.0f;
    params->wind[1] = 0.0f;
    params->wind[2] = 0.0f;

    params->radius = 11.15f;
    params->viscosity = 60.f;
    params->dynamicFriction = 0.2f;
    params->staticFriction = 0.0f;
    params->particleFriction = 1.0f; // scale friction between particles by default
    params->freeSurfaceDrag = 0.0f;
    params->drag = 0.0f;
    params->lift = 1.0f;
    params->numIterations = 4;
    params->fluidRestDistance = 6.f;
    params->solidRestDistance = 11.f;

    params->anisotropyScale = 1.0f;
    params->anisotropyMin = 0.1f;
    params->anisotropyMax = 2.0f;
    params->smoothing = 1.0f;

    params->dissipation = 0.0f;
    params->damping = 0.0f;
    params->particleCollisionMargin = 0.2f;
    params->shapeCollisionMargin = 0.0f;
    params->collisionDistance = max(params->solidRestDistance, params->fluidRestDistance) * 1.2f; // Needed for tri-particle intersection
    params->sleepThreshold = 0.0f;
    params->shockPropagation = 0.0f;
    params->restitution = 0.0f;

    params->maxSpeed = FLT_MAX;
    params->maxAcceleration = 100.0f;	// approximately 10x gravity

    params->relaxationMode = eNvFlexRelaxationLocal;
    params->relaxationFactor = 1.0f;
    params->solidPressure = 1.0f;
    params->adhesion = 0.0f;
    params->cohesion = 0.05f;
    params->surfaceTension = 0.0f;
    params->vorticityConfinement = 0.0f;
    params->buoyancy = 1.0f;
    params->diffuseThreshold = 100.0f;
    params->diffuseBuoyancy = 1.0f;
    params->diffuseDrag = 0.8f;
    params->diffuseBallistic = 16;
    params->diffuseLifetime = 2.0f;
*/
pub fn getDefaultParams() -> NvFlexParams {
    NvFlexParams {
        gravity: [0.0, 0.0, -11.0],
        radius: 11.15,
        viscosity: 50.1,
        dynamicFriction: 0.2,
        staticFriction: 0.5,
        particleFriction: 1.0,
        freeSurfaceDrag: 0.0,
        drag: 0.0,
        lift: 1.0,
        numIterations: 3,
        fluidRestDistance: 6.0,
        solidRestDistance: 11.0,
        anisotropyScale: 1.0,
        anisotropyMin: 0.1,
        anisotropyMax: 2.0,
        smoothing: 1.0,
        dissipation: 0.0,
        damping: 0.0,
        particleCollisionMargin: 0.2,
        shapeCollisionMargin: 6.0,
        collisionDistance: 13.15,
        sleepThreshold: 0.0,
        shockPropagation: 0.0,
        restitution: 1.0,
        maxSpeed: std::f32::MAX,
        maxAcceleration: 100.0,
        relaxationMode: NvFlexRelaxationMode_eNvFlexRelaxationLocal,
        relaxationFactor: 1.0,
        solidPressure: 1.0,
        adhesion: 0.0009,
        cohesion: 0.025,
        surfaceTension: 0.0,
        vorticityConfinement: 10.0,
        buoyancy: 1.0,
        diffuseThreshold: 100.0,
        diffuseBuoyancy: 1.0,
        diffuseDrag: 0.8,
        diffuseBallistic: 16,
        diffuseLifetime: 2.0,
        planes: unsafe { MaybeUninit::uninit().assume_init() },
        numPlanes: 0,
        wind: [0.0, 0.0, 0.0],
    }
}
