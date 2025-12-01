use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{Combinator, Generator};
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
                if state.has_rocket() {
                    Some(PlanetToOrchestrator::AsteroidAck { planet_id: state.id(), rocket: state.take_rocket() })
                }
                else {
                    Some(PlanetToOrchestrator::AsteroidAck { planet_id: state.id(), rocket: None })
                }
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
        todo!()
    }

    fn handle_asteroid(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator) -> Option<Rocket> {
        todo!()
    }

    fn start(&mut self, state: &PlanetState) {
        todo!()
    }

    fn stop(&mut self, state: &PlanetState) {
        todo!()
    }
}