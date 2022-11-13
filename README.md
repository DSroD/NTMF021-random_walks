# Random Walks
## Homework for NTMF021 course
### at MFF UK (Charles University)

Parallel computation of 2D random walks on square / triangular / hexagonal grids.

Futhark part parallely computes many random walks of given length and returns mean euclidean distance from initial point (and variance).
Rust part calls Futhark code, plots results to interactive graphs using plotly and computes linear fit of dependence of log (distance) on log (length).

## Requirements
### Build
- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Futhark compiler](https://futhark.readthedocs.io/en/stable/installation.html)
- [GSL](https://www.gnu.org/software/gsl/)
#### ISPC backend
- [ISPC compiler](https://ispc.github.io/downloads.html)
#### CUDA backend
- [nvcc compiler (included in CUDA toolkit)](https://developer.nvidia.com/cuda-downloads)
- GPU with CUDA support

## Building from source
Makefile is provided, build using `make build [backend=(backend)]`.
Available backends are:
- c
- multicore
- cuda
- ispc

When backend parameter is not specified, sequential C backend is used.