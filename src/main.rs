//! # Use `AtomECS` to simulate diffusion during imaging
//! 
//! This program uses [atomecs](https://github.com/TeamAtomECS/AtomECS) to simulate the diffusion of atoms
//! as they scatter photons during imaging.

extern crate atomecs;
extern crate specs;

use std::time::Instant;

use lib::laser_cooling::force::{EmissionForceOption, EmissionForceConfiguration};
use specs::prelude::*;
use imaging_diffusion::photons::{WritePhotonsSystem, PhotonHistogramSystem, PhotonHistogram};

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, AtomicTransition, Force, Mass, Position, Velocity};
use lib::ecs;
use lib::initiate::NewlyCreated;
use lib::integrator::Timestep;
use lib::laser::gaussian::GaussianBeam;
use lib::laser_cooling::photons_scattered::{ScatteringFluctuationsOption};
use lib::laser_cooling::CoolingLight;
use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;

fn main() {
    
    let now = Instant::now();

    // Create the simulation world
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    
    // Create our dispatcher - which will run the different systems that comprise the simulation.
    let mut builder =
        ecs::create_simulation_dispatcher_builder();


    // Add our extra systems, which do things like generate output.
    //
    // Our photon output system must run after the total scattered each frame has been calculated.
    builder.add(
        file::new::<Position, Text>("pos.txt".to_string(), 10),
        "",
        &[],
    );
    // builder.add(
    //     WritePhotonsSystem::new("photons.csv".to_string()), 
    //     "",
    //     &["calculate_actual_photons"] 
    // );

    // Alternatively, use the histogramming system to count the number of photons at each point in a 3d grid.
    builder.add(
        PhotonHistogramSystem{}, 
        "",
        &["calculate_actual_photons"] 
    );
    world.insert(PhotonHistogram::new(100e-6));

    // Having defined the dispatcher, we now build it and set up required resources in the world.
    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);

    // Create atoms
    for _i in 0..100_000 {
        world
            .create_entity()
            .with(Position {
                pos: Vector3::new(0.0, 0.0, -0.00),
            })
            .with(Atom)
            .with(Force::new())
            .with(Velocity {
                vel: Vector3::new(0.0, 0.0, 0.00),
            })
            .with(NewlyCreated)
            .with(AtomicTransition::rubidium())
            .with(Mass { value: 87.0 })
            .build();
    }

    // Create the imaging laser. We set it aligned to the origin, propagating along +x, and with zero detuning.
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: 0.01,
            power: 0.01,
            direction: Vector3::x(),
            rayleigh_range: f64::INFINITY,
            ellipticity: 0.0,
        })
        .with(CoolingLight::for_species(
            AtomicTransition::rubidium(),
            0.0,
            1,
        ))
        .build();

    // Enable scattering fluctuations and emission forces
    world.insert(EmissionForceOption::On(EmissionForceConfiguration {
        explicit_threshold: 10,
    }));
    world.insert(ScatteringFluctuationsOption::On);

    // Define timestep - we use a small timestep of 0.1 us here to keep it so only ~0-1 photons are emitted each frame.
    let dt = 0.1e-6;
    world.insert(Timestep { delta: dt });


    println!("Initialisation took {} ms.", now.elapsed().as_millis());

    // Run the simulation for a number of steps to generate the output.
    let exposure_us = 15.0;
    let n_steps = (exposure_us * 1.0e-6 / dt).ceil() as u32;
    for _i in 0..n_steps {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
    
    let histogram = world.read_resource::<PhotonHistogram>();
    histogram.write_to_file("photon_histogram.csv".to_string());
    println!("File written.");
}