#![doc = include_str!("../README.md")]

use common_game::{
    components::{
        planet as common_planet,
        resource::{BasicResourceType, ComplexResourceType},
    },
    protocols::{
        orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator},
        planet_explorer::ExplorerToPlanet,
    },
    utils::ID,
};

use crossbeam_channel::{Receiver, Sender};

pub(crate) mod planet;
pub use crate::planet::PlanetAI;

#[cfg(test)]
mod tests;

/// Creates a new planet instance with predefined resource capabilities.
///
/// This function creates a Type-C planet that can generate Carbon as a basic resource
/// and combine various complex resources including Water, Life, Dolphin, Robot,
/// Diamond, and AIPartner.
///
/// # Arguments
///
/// * `id` - Unique identifier for the planet.
/// * `planet_ai` - Implementation of the Planet AI logic.
/// * `orchestrator_channels` - A tuple of (Receiver for Orchestrator, Sender to Orchestrator).
/// * `explorers_receiver` - Channel to receive incoming requests from explorers.
///
/// # Returns
///
/// * `Ok(Planet)` - The initialized planet object.
/// * `Err(String)` - An error if the configuration is invalid.
///
/// # Example
///
/// ```rust
/// use air_fryer::{create_planet, PlanetAI};
/// use crossbeam_channel::unbounded;
///
/// // Orchestrator <=> Planet channels
/// let (tx_to_planet, rx_from_orc) = unbounded();
/// let (tx_to_orc, rx_from_planet) = unbounded();
///
/// // Setup Explorer => Planet channel
/// let (tx_from_explorer, rx_at_planet) = unbounded();
///
/// // Instantiate and handle the Result
/// let planet_id = 42;
/// let ai = PlanetAI::new();
///
/// match create_planet(
///     planet_id,
///     ai,
///     (rx_from_orc, tx_to_orc),
///     rx_at_planet,
/// ) {
///     Ok(_planet) => println!("Planet {} is initialized and ready!", planet_id),
///     Err(e) => panic!("Initialization failed: {}", e),
/// }
/// ```
pub fn create_planet(
    id: ID,
    planet_ai: planet::PlanetAI,
    orchestrator_channels: (Receiver<OrchestratorToPlanet>, Sender<PlanetToOrchestrator>),
    explorers_receiver: Receiver<ExplorerToPlanet>,
) -> Result<common_planet::Planet, String> {
    common_planet::Planet::new(
        id,
        common_planet::PlanetType::C,
        Box::new(planet_ai),
        vec![BasicResourceType::Carbon],
        vec![
            ComplexResourceType::Water,
            ComplexResourceType::Life,
            ComplexResourceType::Dolphin,
            ComplexResourceType::Robot,
            ComplexResourceType::Diamond,
            ComplexResourceType::AIPartner,
        ],
        orchestrator_channels,
        explorers_receiver,
    )
}
