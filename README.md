# rust-game
Game built for STEM Seminar to demonstrate the power of the Rust programming language using bevy and rapier.

# Running the game from the binary (recommended)
  1. Download the latest release for your platform from the releases page on the right-hand side.
  2. Extract the archive using 7zip or equivalent program.
  3. Double click on the binary (.exe) for your platform.
  
# Controls
```
|==========================================|
| Action | Keyboard/mouse | Xinput gamepad |
|^^^^^^^^|^^^^^^^^^^^^^^^^|^^^^^^^^^^^^^^^^|
| Move   | WASD           | Left stick     |
|------------------------------------------|
| Camera | Mouse movement | Right stick    |
|------------------------------------------|
| Jump   | Space          | Bottom button  |
|------------------------------------------|
| Dash   | Q              | Left button    |
|------------------------------------------|
| Fall   | Shift          | Right trigger  |
|==========================================|
```
# Building from source
  1. Download and install [rust](https://www.rust-lang.org/tools/install)
  2. Clone this repository
  3. Extract the assets.tar.xz file
    ```tar -xf assets.tar.xz```
  4. Add the empty cache file (for convex decompositions)
    ```touch cache.bin```
  5. Run the program once with ```cargo run --release```
  6. Wait for the program to decompose objects. The game will appear frozen, and this process may take several minutes, depending on your computer)
  7. Build the final version using ```cargo build --release```
  8. Your binary will appear in ```target/release/rust-game```
