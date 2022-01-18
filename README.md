# imaging_diffusion

This repo shows an example of using AtomECS to simulate diffusion in an imaging system.

We use atomecs to simulate the scattering of photons by a cloud of atoms, and integrate the resulting atomic motion.

* The simulation takes an `atoms.h5` file which defines the initial positions and velocities of atoms to simulate.

* The `PhotonOutputter` resource creates an h5 file which stores all of the output information.

* The `RegisterPhotonsSystem` runs each frame, and stores generated photons in the h5 file.

* The `RegisterInitialAtomsSystem` stores the initial positions and velocities of atoms in the h5 file.

## How to run

* The first time you run you will need to generate a suitable input file. You can do this by running the first cell of `analyse.m`.

* To run the program use `cargo run --release`. The `--release` flag indicates the compiler should use optimisations to increase program performance.

* You can plot generated photons using the final cell of `analyse.m`.

![example photon positions](assets/photon_positions.png)
