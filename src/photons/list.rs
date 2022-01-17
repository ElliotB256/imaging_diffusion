//! Yet another implementation of a way to gather photons.
//! This one builds a vec of photon records in parallel each timestep, and stores the result in memory.

use hdf5::{File, H5Type, SimpleExtents, SliceOrIndex};
use atomecs::{atom::{Position, Velocity, Atom}, laser_cooling::photons_scattered::ActualPhotonsScatteredVector, initiate::NewlyCreated};
use nalgebra::Vector3;
use rand_distr::{UnitSphere, Distribution};
use specs::prelude::*;
use ndarray::arr1;

/// Represents emission of a photon
#[derive(Clone, Copy)]
pub struct PhotonEmission {
    pub position: Vector3<f64>,
    pub direction: Vector3<f64>
}

#[derive(H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(C)]
struct PhotonRecord(f64, f64, f64, f64, f64, f64);
impl PhotonRecord {
    fn new(p: &PhotonEmission) -> Self {
        PhotonRecord {
            0: p.position[0], 
            1: p.position[1],
            2: p.position[2],
            3: p.direction[0],
            4: p.direction[1],
            5: p.direction[2],
        }
    }
}

#[derive(H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(C)]
struct InitialAtomPositionRecord(f64, f64, f64, f64, f64, f64);
impl InitialAtomPositionRecord {
    fn new(p: &Position, v: &Velocity) -> Self {
        InitialAtomPositionRecord { 0: p.pos[0], 1: p.pos[1], 2: p.pos[2], 3: v.vel[0], 4: v.vel[1], 5: v.vel[2] }
    }
}

/// Provides methods for writing photon and atom data to an h5 file.
pub struct PhotonOutputter {
    pub file: File
}
impl PhotonOutputter {
    pub fn new(filename: String) -> Self {
        let file = File::create(filename).expect("Could not create file.");
        let builder = file.new_dataset_builder();
        let se = SimpleExtents::new(&[(1,None)]);
        builder.chunk_cache(10_000, 10_000*48, 1.0).empty::<PhotonRecord>().shape(se).create("photons").expect("Could not create dataset");
        //file.new_attr_builder().with_data(VarLenAscii::from_ascii("atomecs")).create("origin").expect("Unable to create attribute");
        PhotonOutputter { file }
    }

    pub fn append_photons(&self, photons: Vec<PhotonEmission>) {
        // create records and append them to the dataset.
        let n = photons.len();
        let records: Vec<PhotonRecord> = photons.into_iter().map(|p| PhotonRecord::new(&p)).collect();
        let dataset = self.file.dataset("photons").expect("Could not open dataset.");
        let old_length = dataset.size();
        let new_length = old_length + n;
        // resize to new length
        dataset.resize(new_length).expect("Unable to resize dataset.");
        // select a new slice at the end
        dataset.write_slice(&arr1(records.as_slice()), SliceOrIndex::Unlimited{ start: old_length, step: 1, block: 1}).expect("Unable to write photons to file.");
    }

    fn write_initial_atom_positions(&self, records: Vec<InitialAtomPositionRecord>) {
        let n = records.len();
        println!("Writing {:?} initial atom positions and velocities to h5 file.", n);
        let builder = self.file.new_dataset_builder();
        builder.with_data(&arr1(records.as_slice())).create("atoms").expect("Could not create dataset");
    }
}

pub struct RegisterPhotonsSystem;
impl<'a> System<'a> for RegisterPhotonsSystem {
    type SystemData = (
        ReadExpect<'a, PhotonOutputter>,
        ReadStorage<'a, ActualPhotonsScatteredVector>,
        ReadStorage<'a, Position>,
    );
    fn run(&mut self, (output, totals, positions): Self::SystemData) {
        use rayon::prelude::*;

        // Generate photons scattered by each atom in the system.
        let photons: Vec<PhotonEmission> = (&totals, &positions).par_join().map(
            |(total, position)| {
            let mut rng = rand::thread_rng();
            let number = total.contents.iter().map(|a| a.scattered.round() as u32).sum();
            let mut list = Vec::<PhotonEmission>::new();
            for _i in 0..number {
                let v: [f64; 3] = UnitSphere.sample(&mut rng);
                list.push(PhotonEmission {
                    position: position.pos,
                    direction: Vector3::new(v[0], v[1], v[2])
                });
            };
            list
        }).flatten().collect();
        output.append_photons(photons);
    }
}

/// This system gets the initial positions and velocities of atoms immediately after creation, and stores them in the h5 output file.
pub struct RegisterInitialAtomsSystem;
impl<'a> System<'a> for RegisterInitialAtomsSystem {
    type SystemData = (
        ReadExpect<'a, PhotonOutputter>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, NewlyCreated>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
    );
    fn run(&mut self, (output, atoms, new, positions, velocities): Self::SystemData) {
        use rayon::prelude::*;

        // Get initial atom positions
        let atoms: Vec<InitialAtomPositionRecord> = (&atoms, &new, &positions, &velocities).par_join().map(
            |(_atom, _new, pos, vel)| {
                InitialAtomPositionRecord::new(&pos, &vel)
        }).collect();
        if atoms.len() > 0 {
            output.write_initial_atom_positions(atoms);
        }
    }
}