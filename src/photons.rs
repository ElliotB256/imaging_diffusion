//! A module that generates photons scattered by atoms.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::atomic::{Ordering, AtomicU32};

use atomecs::laser_cooling::photons_scattered::ActualPhotonsScatteredVector;
use atomecs::{atom::Position};
use nalgebra::Vector3;
use specs::prelude::*;
use rand_distr;
use rand_distr::{Distribution, UnitSphere};
use std::io::Write;

/// This system writes to an output file when an atom scatters a photon.
///
/// The emission is assumed to be isotropic.
pub struct WritePhotonsSystem {
    stream: BufWriter<File>,
}
impl WritePhotonsSystem {
    /// Create a new [WritePhotonSystem] with given output filename.
    pub fn new(
        file_name: String,
    ) -> Self
    {
        let path = Path::new(&file_name);
        let display = path.display();
        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };
        let writer = BufWriter::new(file);
        WritePhotonsSystem {
            stream: writer
        }
    }
}
impl<'a> System<'a> for WritePhotonsSystem {
    type SystemData = (
        ReadStorage<'a, ActualPhotonsScatteredVector>,
        ReadStorage<'a, Position>,
    );
    fn run(&mut self, (totals, positions): Self::SystemData) {

        let mut rng = rand::thread_rng();

        // Generate photons scattered by each atom in the system.
        for (total, position) in (&totals, &positions).join() {
            let number = total.contents.iter().map(|a| a.scattered.round() as u32).sum();
            for _ in 0..number {
                // Pick a random direction
                let v: [f64; 3] = UnitSphere.sample(&mut rng);

                // Write a line for this photon in the output file
                writeln!(self.stream, "{:?},{:?},{:?},{:?},{:?},{:?}", position.pos[0], position.pos[1], position.pos[2], v[0], v[1], v[2]).expect("Could not write output.");
            }
        }
    }
}


const CELL_DIM: usize = 512;
const CELL_NUM: usize = CELL_DIM*CELL_DIM*CELL_DIM;
const ELEMENT: AtomicU32 = AtomicU32::new(0);

/// This system constructs a spatial histogram of where photons are produced
pub struct PhotonHistogram {
    cell_size: f64,
    cells: Vec<AtomicU32>
}
impl PhotonHistogram {
    pub fn new(
        domain_size: f64
    ) -> Self
    {
        let mut cells = Vec::new();
        for _ in 0..CELL_NUM {
            cells.push(ELEMENT);
        }
        PhotonHistogram {
            cell_size: domain_size / CELL_DIM as f64,
            cells
        }
    }

    /// Counts a given position into the histogram.
    pub fn count(&self, position: Vector3<f64>) {
        if let Some(index) = self.get_index(position) {
            self.cells[index].fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Get the cell index for a given position.
    fn get_index(&self, position: Vector3<f64>) -> Option<usize> {
        let x = (position[0] / self.cell_size) as i32 + (CELL_DIM as i32) / 2;
        let y = (position[1] / self.cell_size) as i32 + (CELL_DIM as i32) / 2;
        let z = (position[2] / self.cell_size) as i32 + (CELL_DIM as i32) / 2;

        if (x < 0 || x >= CELL_DIM as i32) || (y < 0 || y >= CELL_DIM as i32) || (z < 0 || z >= CELL_DIM as i32) {
            return None;
        } else {
            return Some(
                (z as usize) * CELL_DIM * CELL_DIM
                + (y as usize) * CELL_DIM
                + x as usize
            );
        }
    }

    pub fn write_to_file(&self, file_name: String) {
        let path = Path::new(&file_name);
        let display = path.display();
        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };
        let mut writer = BufWriter::new(file);
        for v in self.cells.iter() {
            write!(writer, "{:?},", v.load(Ordering::SeqCst)).expect("Could not write output.");
        }
    }
}
pub struct PhotonHistogramSystem;
impl<'a> System<'a> for PhotonHistogramSystem {
    type SystemData = (
        ReadExpect<'a, PhotonHistogram>,
        ReadStorage<'a, ActualPhotonsScatteredVector>,
        ReadStorage<'a, Position>,
    );
    fn run(&mut self, (histogram, totals, positions): Self::SystemData) {

        use rayon::prelude::*;

        // Generate photons scattered by each atom in the system.
        (&totals, &positions).par_join().for_each(|(total, position)| {
            let number = total.contents.iter().map(|a| a.scattered.round() as u32).sum();
            for _ in 0..number {
                // sow positions into the histogram
                histogram.count(position.pos);
            }
        });
    }
}