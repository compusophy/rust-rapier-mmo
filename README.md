# Ant Colony MMO - Phase 1: WASM Foundation

We have successfully pivoted the project to a Bevy + Rapier2D architecture targeting WebAssembly (WASM).

## Completed Features

1.  **Bevy Engine Setup**: Configured `bevy` 0.14 for 2D rendering.
2.  **Rapier Physics**: Integrated `bevy_rapier2d` for physics simulation (used for transit).
3.  **Hexagonal Grid**: Implemented using `hexx` 0.20.
    -   **Visuals**: Thin wireframe lines (Gizmos).
    -   **Logic**: Discrete movement, one unit per cell.
4.  **Web/WASM Support**:
    -   Mobile-friendly full-screen canvas.
    -   Browser compatibility fixes.
5.  **Gameplay Basics**:
    -   **Camera**: Pan (WASD/Arrows) and Zoom (Q/E).
    -   **Controls**:
        -   **Tap Unit**: Select (White Circle).
        -   **Tap Ground**: Move selected units to nearest available hexes.
        -   **Drag**: Box Select.
    -   **Units**:
        -   **Queen**: Gold, centered at (0,0), **Immobile**.
        -   **Workers**: Red, movable, snap to hex centers.

## Project Structure

-   `src/main.rs`: Core game logic (ECS systems).
-   `Cargo.toml`: Dependencies (Bevy, Rapier, Hexx, WASM features).
-   `index.html`: Web entry point.

## How to Run

### Native (Desktop)
```bash
cargo run
```

### Web (Browser)
```bash
trunk serve
# Open http://localhost:8080
```
