# air_fryer.rs

Planet implementation of the [common_crate](https://github.com/unitn-ap-2025/common), providing the core logic for managing a planet’s resources, energy, and interactions with explorers.

## Overview

<img align="right" width="230" src="https://github.com/user-attachments/assets/e7af14f8-9400-4f85-b55c-e7af98d949a9">

**Planet** is an autonomous agent that runs on a separate thread and manages:
- **Resource Generation**: Creates basic resources (Carbon)
- **Resource Combination**: Combines basic resources into complex resources
- **Energy Management**: Handles energy cells and rocket construction
- **Explorer Interactions**: Responds to explorer requests for resources
- **Asteroid Defense**: Detects and defends against incoming asteroids

## Architecture

### Message-Driven Design

The system uses **crossbeam_channel** for inter-thread communication between:
- **Orchestrator**: Central game controller
- **Explorers**: Players requesting resources
- **Planet**: AI handler managing planet state

Messages flow bidirectionally:
```
  Orchestrator  ⟷         Planet        ⟷       Explorers
   (commands)        (queries & updates)     (requests & responses)
```

## Core Features

### 1. **Energy Management**

The planet manages energy cells to power all operations:

- **Sunray Charging**: Receives sunrays from the orchestrator and charges energy cells
  - First sunray: Charges cell if empty
  - Subsequent sunrays: Build rocket (if needed) or charge new cells
  
- **Cell-Based Operations**: Each resource generation/combination consumes one charged energy cell

### 2. **Resource Generation**

Basic resource generation (currently implements **Carbon**):

```rust
ExplorerToPlanet::GenerateResourceRequest { resource: Carbon }
  ↓
PlanetAI checks:
  - Is it Carbon? Ok
  - Is cell charged? Ok
  ↓
Generator creates Carbon
  ↓
PlanetToExplorer::GenerateResourceResponse { resource: Some(Carbon) }
```

### 3. **Resource Combination**

Combines two resources into one complex resource using the **Combinator**:

**Supported Combinations:**
| Combination | Input 1 | Input 2 | Output |
|---|---|---|---|
| Water | Hydrogen | Oxygen | Water |
| Diamond | Carbon | Carbon | Diamond |
| Life | Water | Carbon | Life |
| Robot | Silicon | Life | Robot |
| Dolphin | Water | Life | Dolphin |
| AIPartner | Robot | Diamond | AIPartner |

Each combination:
1. Checks if both resources are available
2. Consumes one charged energy cell
3. Returns the combined resource or an error (with both input resources)

### 4. **Asteroid Defense System**

Early warning system to alert explorers of incoming asteroids:

- **Asteroid Handling**: 
  - If rocket exists &rArr; Send rocket and reset warning
  - If no rocket &rArr; Build rocket from energy cell (if possible)
  - If can't build &rArr; Flag pending warning

- **Explorer Notification**: Uses a "secret channel" in the `SupportedCombinationResponse`
  - Normal state: All 6 complex resources supported
  - Asteroid incoming: Removes `AIPartner` from list (encoded as bit = 1)
  - Explorer detects missing resource and knows asteroid is coming

### 5. **Explorer Management**

Tracks explorer presence and responds to queries:

- **Resource Queries**: `SupportedResourceRequest` &rArr; Returns `{Carbon}`
- **Combination Queries**: `SupportedCombinationRequest` &rArr; Returns supported combinations
- **Energy Status**: `AvailableEnergyCellRequest` &rArr; Returns charged cell count
- **Explorer Tracking**: Manages incoming/outgoing explorer requests

## Implementation Details

### PlanetAI State Machine

```
┌─────────────────┐
│   Stopped       │
│  (started=false)│
└────────┬────────┘
         │ StartPlanetAI
         ↓
┌─────────────────┐
│   Running       │
│  (started=true) │ ← Processes all messages
└────────┬────────┘
         │ StopPlanetAI
         ↓
┌─────────────────┐
│   Stopped       │
└─────────────────┘
```

### Message Handlers

**`handle_orchestrator_msg`**: 
- `StartPlanetAI` / `StopPlanetAI`: Control planet lifecycle
- `Sunray`: Charge energy cells or build rocket
- `Asteroid`: Trigger defense mechanism
- `IncomingExplorerRequest` / `OutgoingExplorerRequest`: Track explorers
- `InternalStateRequest`: Return planet snapshot
- `KillPlanet`: Terminate gracefully

**`handle_explorer_msg`**:
- `GenerateResourceRequest`: Create basic resources
- `CombineResourceRequest`: Combine two resources
- `SupportedResourceRequest`: Report capabilities
- `SupportedCombinationRequest`: List possible combinations (with asteroid warning)
- `AvailableEnergyCellRequest`: Report energy status

## Testing

The project includes comprehensive test suites:

- **Unit Tests**: Individual feature tests
- **Integration Tests**: Multi-planet scenarios
- **Combination Tests**: All 6 resource combinations in a single test

## Dependencies

- **common_game**: Core game framework with components and protocols
- **crossbeam_channel**: Efficient multi-producer, multi-consumer channels
- **log**: Logging framework

## Future Enhancements

Planned improvements for upcoming releases:

-  Add logging for all state transitions

For a complete list of proposed features and known issues, see our [GitHub Issues](https://github.com/Advanced-Panic-Programming/Main/issues).
