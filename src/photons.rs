//! A module that generates photons scattered by atoms.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use atomecs::laser_cooling::photons_scattered::ActualPhotonsScatteredVector;
use atomecs::{laser_cooling::photons_scattered::TotalPhotonsScattered, atom::Position};
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