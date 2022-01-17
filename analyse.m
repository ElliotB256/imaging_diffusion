%% Position atoms
% Position some atoms - lets make a csv.

pos = zeros(10, 3);
% string of atoms separated by 1um along x.
pos(:,1) = (-4.5:1:4.5) * 1e-6;
csvwrite('input.csv', pos);

%% Analyse
% Analyse the simulated photon distribution and see how it looks.

data = csvread('photons.csv');
pos = data(:, 1:3);

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

%% Histogram
% Analyse the histogram (if you are using the PhotonHistogramSystem).

data = csvread('photon_histogram.csv');
data = reshape(data, 256, 256, 256);

% Sum along one dimension to make a 2d image
image = squeeze(sum(data, 3));
imagesc(log10(image));

% % Zoom in on the interesting bit
% xlim(255 + [ -30 30 ]);
% ylim(255 + [ -30 30 ]);