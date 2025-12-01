use std::collections::HashSet;
use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{BasicResourceType, Combinator, ComplexResourceType, Generator};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};

struct PlanetAI{

}
impl planet::PlanetAI for PlanetAI {
    fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(_) => {
                if state.has_rocket() {
                    todo!()
                }
                else {
                    todo!()
                }
            }
            OrchestratorToPlanet::Asteroid(_) => {
                Some(PlanetToOrchestrator::AsteroidAck {planet_id: state.id(), rocket: self.handle_asteroid(state, generator, combinator)})
            }
            OrchestratorToPlanet::StartPlanetAI(_) => {
                todo!()
            }
            OrchestratorToPlanet::StopPlanetAI(_) => {
                todo!()
            }
            OrchestratorToPlanet::InternalStateRequest(_) => {
                todo!()
            }
        }
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest {..} => {
                let mut hs = HashSet::new();
                hs.insert(BasicResourceType::Carbon);
                Some(PlanetToExplorer::SupportedResourceResponse { resource_list: Some( hs) })
            }
            ExplorerToPlanet::SupportedCombinationRequest { .. } => {
                let mut hs = HashSet::new();
                hs.insert(ComplexResourceType::AIPartner);
                hs.insert(ComplexResourceType::Diamond);
                hs.insert(ComplexResourceType::Dolphin);
                hs.insert(ComplexResourceType::Water);
                hs.insert(ComplexResourceType::Life);
                hs.insert(ComplexResourceType::Robot);
                Some(PlanetToExplorer::SupportedCombinationResponse { combination_list: Some(hs),});
                todo!()
            }
            ExplorerToPlanet::GenerateResourceRequest { .. } => {
                todo!()
            }
            ExplorerToPlanet::CombineResourceRequest { .. } => {
                todo!()
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                todo!()
            }
            ExplorerToPlanet::InternalStateRequest { .. } => {
                todo!()
            }
        }
    }

    fn handle_asteroid(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator) -> Option<Rocket> {
        if state.has_rocket() {
            Some(state.take_rocket()?)
        }
        else{
            None
        }
    }

    fn start(&mut self, state: &PlanetState) {
        todo!()
    }

    fn stop(&mut self, state: &PlanetState) {
        todo!()
    }
}