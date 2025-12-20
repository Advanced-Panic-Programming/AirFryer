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
