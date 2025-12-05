use std::collections::HashSet;
use std::time::SystemTime;
use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, ComplexResourceType, Generator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
pub struct PlanetAI{
    has_explorer : bool,
    started: bool,
}
impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
            has_explorer : false,
            started: false,
        }
    }
}
impl planet::PlanetAI for PlanetAI {
    fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
        if self.started {
            match msg {
                OrchestratorToPlanet::Sunray(Sunray) => {
                    if ! state.cell(0).is_charged() {

                        state.cell_mut(0).charge(Sunray);

                        if state.can_have_rocket(){

                            if ! state.has_rocket(){
                                state.build_rocket(0);
                            }
                        }
                    }
                    else{
                        state.build_rocket(0);
                        state.cell_mut(0).charge(Sunray);
                    }
                    Some(PlanetToOrchestrator::SunrayAck { planet_id: 0})
                }
                OrchestratorToPlanet::Asteroid(Asteroid) => {
                    Some(PlanetToOrchestrator::AsteroidAck {planet_id: state.id(), rocket: self.handle_asteroid(state, generator, combinator)})
                }
                OrchestratorToPlanet::StartPlanetAI => {
                    self.start(state);
                    Some(PlanetToOrchestrator::StartPlanetAIResult {planet_id: state.id()})
                }
                OrchestratorToPlanet::StopPlanetAI => {
                    self.stop(state);
                    Some(PlanetToOrchestrator::StopPlanetAIResult {planet_id: state.id()})
                }
                OrchestratorToPlanet::InternalStateRequest => {
                    todo!()
                },
                OrchestratorToPlanet::IncomingExplorerRequest { .. } =>{
                    self.has_explorer = true;
                    Some(PlanetToOrchestrator::IncomingExplorerResponse {
                        planet_id: state.id(),
                        res: Ok(()),
                    })
                }
                OrchestratorToPlanet::OutgoingExplorerRequest { .. } => {
                    self.has_explorer = false;
                    Some(PlanetToOrchestrator::OutgoingExplorerResponse {
                        planet_id: state.id(),
                        res: Ok(()),
                    })
                }
            }
        }
        else{
            None
        }
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest {explorer_id } => {
                let mut hs = HashSet::new();
                hs.insert(BasicResourceType::Carbon);
                Some(PlanetToExplorer::SupportedResourceResponse { resource_list: hs })
            }
            ExplorerToPlanet::SupportedCombinationRequest {explorer_id} => {
                let mut hs = HashSet::new();
                hs.insert(ComplexResourceType::AIPartner);
                hs.insert(ComplexResourceType::Diamond);
                hs.insert(ComplexResourceType::Dolphin);
                hs.insert(ComplexResourceType::Water);
                hs.insert(ComplexResourceType::Life);
                hs.insert(ComplexResourceType::Robot);
                Some(PlanetToExplorer::SupportedCombinationResponse { combination_list: hs })
            }
            ExplorerToPlanet::GenerateResourceRequest {explorer_id, resource} => {
                if resource != BasicResourceType::Carbon {
                    Some(PlanetToExplorer::GenerateResourceResponse {resource: None})
                }
                else{
                    let generated = generator.make_carbon(state.cell_mut(0));
                    match generated {
                        Ok(carbon) => {
                            Some(PlanetToExplorer::GenerateResourceResponse {resource: Some(BasicResource::Carbon(carbon))})
                        }
                        Err(string) => {
                            Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                        }
                    }
                }

            }
            ExplorerToPlanet::CombineResourceRequest { .. } => {
                todo!()
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: state.cells_count() as u32 })
            }
        }
    }

    fn handle_asteroid(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator) -> Option<Rocket> {
        if state.has_rocket() {
            Some(state.take_rocket()?)
        }  else {
            // Error handling is done in the common-code trait implementation
            // i.e. planet can have rocket, planet energy cell is charged, ...

            // Try to build the rocket
            let _ = state.build_rocket(state.cells_count());
            state.take_rocket()

        }
    }

    fn start(&mut self, state: &PlanetState) {
        //Manda messaggio per vedere se ha l'esploratore
        //if -> true o false
        self.started = true;
        //to do
    }

    fn stop(&mut self, state: &PlanetState) {
        self.started = false;

        //to do
    }
}