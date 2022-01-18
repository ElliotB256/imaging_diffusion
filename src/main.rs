//! # Use `AtomECS` to simulate diffusion during imaging
//! 
//! This program uses [atomecs](https://github.com/TeamAtomECS/AtomECS) to simulate the diffusion of atoms
//! as they scatter photons during imaging.

extern crate atomecs;
extern crate specs;

use std::time::Instant;

use hdf5::{File, SliceOrIndex, Error, H5Type};
use imaging_diffusion::photons::list::{RegisterPhotonsSystem, PhotonOutputter, RegisterInitialAtomsSystem};
use lib::laser_cooling::force::{EmissionForceOption, EmissionForceConfiguration};
use serde::Deserialize;
use specs::prelude::*;

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

#[derive(Debug, Deserialize)]
pub struct AtomRecord {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

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

    // Output atoms to an h5 file
    builder.add(RegisterPhotonsSystem, "", &[]);
    builder.add(RegisterInitialAtomsSystem, "", &[]);
    world.insert(PhotonOutputter::new("output.h5".to_string()));

    // // Having defined the dispatcher, we now build it and set up required resources in the world.
    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world);

    // Create atoms from an input h5 file.
    // The input file format has a dataset called 'atoms' which has (x,y,z,vx,vy,vz) in SI units
    load_atoms_from_h5(&mut world).expect("Unable to load initial atom position and velocity from 'atoms.h5' input file.");

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
    let exposure_us = 100.0;
    let n_steps = (exposure_us * 1.0e-6 / dt).ceil() as u32;
    for _i in 0..n_steps {
        dispatcher.dispatch(&mut world);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}

const READ_BATCH_SIZE: usize = 1000;

#[derive(H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(C)]
pub struct InputAtomPositionRecord {
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64
}

fn load_atoms_from_h5(world: &mut World) -> Result<(), Error> {
    let file = File::open("atoms.h5")?;
    let ds = file.dataset("atoms")?;
    let mut n_created = 0;
    
    for i in (0..ds.size()).step_by(READ_BATCH_SIZE) {
        let n_to_read = READ_BATCH_SIZE.min(ds.size()-i);
        let atoms = ds.read_slice_1d::<InputAtomPositionRecord, SliceOrIndex>(SliceOrIndex::SliceCount{ start: i, step: 1, block: 1, count: n_to_read })?;
        n_created += atoms.len();

        for atom in atoms {
            world
                .create_entity()
                .with(Position {
                    pos: Vector3::new(atom.x, atom.y, atom.z),
                })
                .with(Atom)
                .with(Force::new())
                .with(Velocity {
                    vel: Vector3::new(atom.vx, atom.vy, atom.vz),
                })
                .with(NewlyCreated)
                .with(AtomicTransition::rubidium())
                .with(Mass { value: 87.0 })
                .build();
        }        
    }

    println!("Loaded {:?} atoms from the input h5 file.", n_created);
    Ok(())
}