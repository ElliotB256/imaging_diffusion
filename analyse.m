%% Position atoms
% Let's create an initial input file of atom positions and velocities.

% normal distribution for position, 100 um.
pos_dist = makedist('Normal', 'sigma', 1e-4); % m
vel_dist = makedist('Normal', 'sigma', 1e-3); % m/s

% create structures for output file.
n = int32(1e4);
atoms = arrayfun(...
    @(x,y,z,vx,vy,vz) struct('x', x, 'y', y, 'z', z, 'vx', vx, 'vy', vy, 'vz', vz), ...
    random(pos_dist,n,1), random(pos_dist,n,1), random(pos_dist,n,1), ...
    random(vel_dist,n,1), random(vel_dist,n,1), random(vel_dist,n,1) ...
        );

hdf5write('atoms.h5', '/atoms', atoms);

%% Run simulation
% Run the simulation in command line:
%   cargo run --release

%% Analyse
% Analyse the simulated photon distribution and see how it looks.
% 
% You can read matlab output using hdf5read but its waaaay too slow:
% photons = hdf5read('output.h5', '/photons');
% 
% Instead, use h5read:
p = h5read('output.h5', '/photons');
pos = [p.x0, p.x1, p.x2];

% Convert to units of um
pos = pos * 1e6;

% Produce a plot showing where photons are emitted.
plot3(pos(:,1), pos(:,2), pos(:,3), '.')

% Make the plot pretty.
% axis equal;
view(-45, 45);
set(gcf, 'Color', 'w');
xlabel('x ($\mu$m)', 'Interpreter', 'Latex');
xlabel('y ($\mu$m)', 'Interpreter', 'Latex');
xlabel('z ($\mu$m)', 'Interpreter', 'Latex');
