# AirFryer.rs

A Rust implementation of a planet agent for the [common_game](https://github.com/unitn-ap-2025/common) framework. This crate provides the core logic for managing a planet's resources, energy systems, and interactions with explorers in a distributed game environment.

Documentation available: [docs](https://advanced-panic-programming.github.io/AirFryer/)

## Overview

<img align="right" width="230" src="https://github.com/user-attachments/assets/e7af14f8-9400-4f85-b55c-e7af98d949a9">

**AirFryer** is a Type-C planet that operates as an autonomous agent running in its own thread. It manages resource generation, energy management, and explorer communications through a message-driven architecture.

### Key Capabilities

- **Resource Generation**: Creates Carbon as the primary basic resource
- **Complex Resource Synthesis**: Combines basic resources into 6 different complex resources
- **Energy Management**: Handles sunray charging and energy cell operations
- **Explorer Protocol**: Responds to resource requests and combination queries
- **Asteroid Defense**: Early warning system and rocket-based defense mechanism

## Features

### Energy Management System

The planet manages energy through a cell-based system:

1. **Sunray Reception**: 
   - First sunray charges an empty energy cell
   - Subsequent sunrays build rockets (if needed) or charge additional cells

2. **Energy Consumption**:
   - Each resource generation consumes one charged energy cell
   - Each resource combination consumes one charged energy cell
   - Rocket construction requires energy from cells

### Resource Operations

#### Basic Resource Generation

The planet generates **Carbon** as its basic resource:

```ignore
// Explorer requests Carbon
ExplorerToPlanet::GenerateResourceRequest { resource: BasicResourceType::Carbon }
  ↓
// Planet validates request and energy availability
PlanetAI.handle_explorer_msg()
  ↓
// Generator creates Carbon using energy cell
Generator.make_carbon(energy_cell)
  ↓
// Response sent back to explorer
PlanetToExplorer::GenerateResourceResponse { resource: Some(Carbon) }
```

#### Complex Resource Combination

The planet supports 6 complex resource combinations:

| Complex Resource | Required Inputs | Energy Cost |
|------------------|-----------------|-------------|
| **Water** | Hydrogen + Oxygen | 1 cell |
| **Diamond** | Carbon + Carbon | 1 cell |
| **Life** | Water + Carbon | 1 cell |
| **Robot** | Silicon + Life | 1 cell |
| **Dolphin** | Water + Life | 1 cell |
| **AIPartner** | Robot + Diamond | 1 cell |

### Asteroid Defense System

<!-- TODO: finish this section -->

## API Reference

For APIs, see: [docs](https://advanced-panic-programming.github.io/AirFryer/)

## Testing

The project includes comprehensive test coverage:

<!-- TODO: finish this section -->

### Test Categories

- **Unit Tests**: Individual component testing

<!-- TODO: finish this section -->

## Future Enhancements

For a complete list of proposed features and known issues, see our [GitHub Issues](https://github.com/Advanced-Panic-Programming/AirFryer/issues).
