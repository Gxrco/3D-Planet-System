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
![Solar System Render](https://github.com/user-attachments/assets/0779f04b-0e3f-4141-ba2e-e1853694bd67)

## Technical Requirements
- Rust
- minifb window system
- nalgebra-glm for mathematics
- Custom shader implementation
- No external texture dependencies