use std::collections::HashSet;
use std::time::SystemTime;
use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, ComplexResourceType, Generator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
pub struct PlanetAI{
    has_explorer : bool, // wait for OutgoingExplorerRequest message to be implemented in the common-code trait
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
                    self.start(state);
                    Some(PlanetToOrchestrator::StartPlanetAIResult {planet_id: state.id(),timestamp: SystemTime::now() })
                }
                OrchestratorToPlanet::StopPlanetAI(_) => {
                    self.stop(state);
                    Some(PlanetToOrchestrator::StopPlanetAIResult {planet_id: state.id(), timestamp: SystemTime::now() })
                }
                OrchestratorToPlanet::InternalStateRequest(_) => {
                    todo!()
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
                Some(PlanetToExplorer::SupportedResourceResponse { resource_list: Some( hs) })
            }
            ExplorerToPlanet::SupportedCombinationRequest {explorer_id} => {
                let mut hs = HashSet::new();
                hs.insert(ComplexResourceType::AIPartner);
                hs.insert(ComplexResourceType::Diamond);
                hs.insert(ComplexResourceType::Dolphin);
                hs.insert(ComplexResourceType::Water);
                hs.insert(ComplexResourceType::Life);
                hs.insert(ComplexResourceType::Robot);
                Some(PlanetToExplorer::SupportedCombinationResponse { combination_list: Some(hs)})
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

                /*
                if resource != BasicResourceType::Carbon {
                    return Some(PlanetToExplorer::GenerateResourceResponse(Err("Can't do it")));
                    Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                }
                else{
                    //state.cell_mut(0).charge(Sunray::new()); //DA ELIMINARE
                    let carbon = generator.make_carbon(state.cell_mut(0));
                    match carbon {

                        Ok(res) => {

                            Some(PlanetToExplorer::GenerateResourceResponse {resource: Some(BasicResource::Carbon(res))})
                        }
                        Err(_) => {Some(PlanetToExplorer::GenerateResourceResponse { resource: None })}
                    }

                }
                 */
            }
            ExplorerToPlanet::CombineResourceRequest { .. } => {
                todo!()
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => { // Enum has explorer_id param -> unused
                Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: state.cells_count() as u32 })
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