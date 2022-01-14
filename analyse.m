%% Analyse
% Analyse the simulated photon distribution and see how it looks.

data = csvread('photons.csv');
pos = data(:, 1:3);

% Convert to units of um
pos = pos * 1e6;

% Produce a plot showing where photons are emitted.
plot3(pos(:,1), pos(:,2), pos(:,3), '.')

% Make the plot pretty.
axis equal;
view(-45, 45);
set(gcf, 'Color', 'w');
xlabel('x ($\mu$m)', 'Interpreter', 'Latex');
xlabel('y ($\mu$m)', 'Interpreter', 'Latex');
xlabel('z ($\mu$m)', 'Interpreter', 'Latex');