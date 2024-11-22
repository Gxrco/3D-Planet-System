# Rust Solar System Renderer

A sophisticated 3D solar system renderer implemented in pure Rust, featuring advanced shader-based planet rendering without relying on textures or external materials.

## Key Features

### Dynamic Planet Rendering
- Advanced multi-layered shader system for realistic planet visualization
- Real-time procedural generation of planetary features
- Complex atmospheric and surface effects

### Celestial Bodies
- Realistic star (Sun) with dynamic plasma-like surface and pulsing effects
- Detailed rocky planets with unique characteristics:
  - Mercury: Crystalline surface with mineral variations
  - Earth: Multi-layered terrain with oceans, continents, and dynamic cloud systems
  - Mars: Detailed surface with canyons and dynamic dust storms
- Gas giants with distinctive features:
  - Jupiter: Dynamic band patterns and storm systems
  - Saturn: Complete ring system with crystalline effects
- Moon system with realistic crater generation and orbital mechanics

### Technical Highlights
- Pure shader-based implementation without textures
- Multi-layered rendering pipeline
- Real-time atmospheric effects and cloud movements
- Dynamic lighting system with ambient, diffuse, and specular components
- Procedural noise-based terrain generation
- Interactive camera system with orbital controls

### Controls
- Arrow keys: Orbit camera
- WASD: Move camera focus
- QE: Camera up/down
- 1-8 keys: Toggle planet visibility
- ESC: Exit application

## Implementation Details
The project demonstrates advanced graphics programming concepts including:
- Custom shader pipeline implementation
- Procedural texture generation
- Dynamic lighting calculations
- Atmospheric scattering simulation
- Orbital mechanics
- Real-time noise-based effects

## Screenshots
- Star (Breathing effect)
<img src="https://github.com/user-attachments/assets/e5801329-ab52-4afa-9248-150d87a0e84a" width="280" height="300">

- Planet Purple (Waving surface)
<img src="https://github.com/user-attachments/assets/1f987745-8e50-4c4e-86bb-500f3c440d32" width="280" height="300">

- Planet Earth? (Own a satellite)
<img src="https://github.com/user-attachments/assets/f826d67a-e136-4806-89b5-89f590525040" width="280" height="300">

- Planet Large striping (Implement bands shade)
<img src="https://github.com/user-attachments/assets/22576db3-2226-4a6e-81e1-812b6a87ffe9" width="280" height="300">

- New planet (it has a ring)
<img src="https://github.com/user-attachments/assets/d190aa23-8684-45b0-9694-eb5dec78a97d" width="280" height="300">

- Mars brother (Aspect red)
<img src="https://github.com/user-attachments/assets/52f31570-0266-4bbe-9955-2fed5ead19bb" width="280" height="300">

- Planet toxic (Double storm waving surface)
<img src="https://github.com/user-attachments/assets/d9bceb38-bfad-472c-968e-86dcab60ef5f" width="280" height="300">



## Technical Requirements
- Rust
- minifb window system
- nalgebra-glm for mathematics
- Custom shader implementation
- No external texture dependencies
