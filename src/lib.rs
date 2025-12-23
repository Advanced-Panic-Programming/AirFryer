use crate::air_fryer::PlanetAI;
use common_game::components::planet;
use common_game::components::planet::PlanetType;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};

mod air_fryer;
mod mock_planet;

/// Creates a new planet instance with predefined resource capabilities.
/// 
/// This function creates a Type-C planet that can generate Carbon as a basic resource
/// and combine various complex resources including Water, Life, Dolphin, Robot, 
/// Diamond, and AIPartner.
/// 
/// # Arguments
/// 
/// * `id` - Unique identifier for the planet
/// * `planet_ai` - AI implementation that controls the planet's behavior
/// * `orchestrator_channels` - Communication channels with the orchestrator:
///   - Receiver for orchestrator messages (OrchestratorToPlanet)
///   - Sender for planet responses (PlanetToOrchestrator)
/// * `explorers_receiver` - Receiver for messages from explorers (ExplorerToPlanet)
/// 
/// # Returns
/// 
/// * `Ok(Planet)` - Successfully created planet instance
/// * `Err(String)` - Error message if planet creation fails
/// 
/// # Example
/// 
/// ```ignore
/// let planet = create_planet(
///     42,
///     PlanetAI::new(),
///     (orc_receiver, planet_sender),
///     explorer_receiver
/// )?;
/// ```
pub fn create_planet(
    id: ID,
    planet_ai: PlanetAI,
    orchestrator_channels: (Receiver<OrchestratorToPlanet>, Sender<PlanetToOrchestrator>),
    explorers_receiver: Receiver<ExplorerToPlanet>,
) -> Result<planet::Planet, String> {
    planet::Planet::new(
        id,
        PlanetType::C,
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
