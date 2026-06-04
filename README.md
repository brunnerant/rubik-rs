# Rust Rubik's cube libraries
Rust data structures and algorithms to solve a Rubik's cube.

For now, there is only one crate - `rubik-lib` that contains the logic to represent Rubik's cube states and solve them. In the future, I plan to add other crates to display cubes in 3D, and to scan cubes from images using computer vision.

The library `rubik-lib` is organized into different modules:
- `core`: the core data structures used to represent moves and state, and how to compose them.
- `algebra`: using the core data structures, this builds algebraic abstractions over the Rubik's cube group, like symmetries, coordinates, and move tables.
- `solve`: this contains the different solvers that I implemented for the Rubik's cube.

# Solvers
For now, there are two solvers:
- `solve/four_list`: using the four list algorithm, uses an optimized brute-force search on the length 20 moves to find a solution. This is guaranteed to find a solution in 20 moves or less, thanks to God's number, but it is rather slow because of the sheer number of different states that it has to search. It usually takes a few minutes to find a solution.
- `solve/kociemba`: work in progress.
