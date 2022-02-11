## Building the project

### You'll need:

- Rust (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, you might need to `rustup override set nightly` inside the project folder)

- Python

### Instructions:

Create a Python virtual environment,

```bash
python3 -m venv <my_env_name>
```

activate it

```bash
source <my_env_name>/bin/activate
```

Install Python dependencies

```bash
pip3 install numpy scipy matplotlib scikit-learn scikit-image opencv-python jupyterlab ipykernel maturin
```

Register the virtual environment in Jupyter kernels

```bash
python3 -m ipykernel install --user --name=<my_env_name>
```

Compile the Rust project and install it in the current virtual env

```
maturin develop --release
```

Start the jupyter-lab server, and open the included notebook `example.ipynb`, then you should be good to go.

```bash
jupyter-lab
```

## Implementation details:

Rough responsibilities:

- `lib.rs`: Provides the Python wrapper. The calculation logic is mostly done inside `inner_calculate_v2` function, which
  is commented in a relatively verbose fashion.
- `field.rs`: The `Field` struct that holds the geometric information: currently circles and heatmap size.
- `ray.rs`: The `Ray` struct that represents one ray with origin and direction that can be traced through the field.
- `sensor.rs`: The `Sensor` struct that represents the sensor with rays that it emits.
- `shape.rs`: The `Circle` struct that represents one circle on the field that can be hit by rays. More shapes/obstacles
  can be relatively easily added here with their own implementation of the `Hittable` trait, and utility functions.

- `report.rs`: The struct holds the report of the simulation. This is insanely bad, should be improved later.

- `rangestack.rs`: The `RangeStack` struct that holds the currently covered angle ranges of a circle. It provides methods for
  adding ranges safely wrapping around [0, 2 * PI], merging them, etc..

- `point.rs`, `vector.rs`: math utilities

## Performance

In general, very little real optimization is done. Two main processing steps are parallelized: the outermost iteration which moves the first sensor, and merging the `RangeStack`'s ranges.
The following other optimizations seem realistic to me at the moment:

- Currently if two consecutive rays are hitting the same object, we immediately calculate the angle range that's covered by them. This is unnecessary, because whenever we hit an object, we'll skip while the same object is hit with the upcoming rays, until another object or nothing is hit. The left- and rightmost rays hitting the same object should give us the view angle. This probably can save _a lot_ of time and calculations.

- Having n >= 2 sensors, many position setups are duplicated, because the sensors are identical, thus interchangeable. We could skip the unnecessary parts, but it's not trivial to me atm because of the parallelization.

Given an example heatmap of:

- size (1080, 1080)
- sensor resolution of 2880 (that means the full circle is split into 2880 equal parts, so the angle difference between two consecutive rays is 360° / 2880 = 0.125°)
- pixel step of 10 (meaning we're moving along the heatmap's circumference by 10 pixels at a time)
- 5 circles on the heatmap
- 2 sensors

which is ~186.000 iterations,

runs in **4.11 s**,

measured with an AMD Ryzen 5 4600G (using the Python wrapper, raw Rust code should be slightly faster)

Peak memory usage:

- with Python wrapper: 201.9 MB

- raw Rust code: 6.4 MB
