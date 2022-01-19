# rusty-vegetation

Graphical Rust program that uses a fractal algorithm to draw a "tree" of sorts.

## To Build and Run

### On Linux:

1. Install `build-essentials` or the equivalent.
2. Set up Rust using the following:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

3. `cd` to the rusty-vegetation root directory, then build and install it by running:

```bash
cargo build
cargo install --path .
```

4. Run the program:

```bash
rusty-vegetation 1 75 9 0.000651041666667 0.0009765625
```

### On Windows:

1. Install the Visual C++ 2019 Build Tools (command-line), or else full Visual Studio 2019 including C++ workflow.
2. Install and run `rustup`, such as using Chocolatey.
3. `cd` to the rusty-vegetation root directory, then build and install it by running:

```PowerShell
cargo build
cargo install --path .
```
 
4. Run the program:

```PowerShell
rusty-vegetation 1 75 9 0.000651041666667 0.0009765625
```

## Command-Line Arguments

The command-line arguments to the program are, in order:

- The number of trees to generate (effectively limited to 1 at this point)
- The percentage fill factor to use (0-100)
- The fractal level to use (7-9 is a good range)
- The horizontal scaling factor to use (try 1 / 1536, or 0.000651041666667)
- The vertical scaling factor (try 1 / 1024, or 0.0009765625)
