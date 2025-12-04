use std::collections::HashSet;
use std::os::unix::raw::time_t;
use std::time::{Duration, SystemTime};
use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, ComplexResourceType, Generator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};

pub struct PlanetAI{
    has_explorer : bool,
    started: bool,
    pending_warning: bool, // To warn the explorer
    delay_asteroid_ack: u32, // Delay planet destruction
    pending_rocket_for_ack: Option<Option<Rocket>>, // Rocket/None to send against asteroid
}
impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
            has_explorer : false,
            started: false,
            pending_warning: false,
            delay_asteroid_ack: 0,
            pending_rocket_for_ack: None,
        }
    }
}
impl planet::PlanetAI for PlanetAI {
    fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
        if self.started {
            // Handle asteroid response delay
            if self.delay_asteroid_ack > 0 {
                self.delay_asteroid_ack -= 1;

                if self.delay_asteroid_ack > 0 {
                    return None;
                }

                // Delay ended
                let rocket = self.pending_rocket_for_ack.take().unwrap();

                return Some(PlanetToOrchestrator::AsteroidAck {
                    planet_id: state.id(),
                    rocket,
                });
            }

            match msg {
                OrchestratorToPlanet::Sunray(sunray) => {
                    if ! state.cell(0).is_charged() {

                        state.cell_mut(0).charge(sunray);

                        if state.can_have_rocket(){

                            if ! state.has_rocket(){
                                let _ = state.build_rocket(0);
                            }
                        }
                    }
                    else{
                        let _ = state.build_rocket(0);
                        state.cell_mut(0).charge(sunray);
                    }
                    Some(PlanetToOrchestrator::SunrayAck { planet_id: 0})
                }
                OrchestratorToPlanet::Asteroid(_) => {
                    let rocket = self.handle_asteroid(state, generator, combinator);

                    // Set AsteroidAck delay
                    self.pending_rocket_for_ack = Some(rocket);
                    self.delay_asteroid_ack = 10;

                    // Delayed response
                    None

                    // Some(PlanetToOrchestrator::AsteroidAck {planet_id: state.id(), rocket: self.handle_asteroid(state, generator, combinator)})
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
                    todo!()
                }
                OrchestratorToPlanet::OutgoingExplorerRequest { .. } => {
                    todo!()
                }
            }
        }  else {
            None
        }
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {

        // Warn the explorer
        if self.pending_warning {
            self.pending_warning = false;
            return Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: 0u32 }) // 0 = incoming asteroid
        }

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
            self.pending_warning = false;
            return state.take_rocket();
        }

        // Try to build a rocket
        if state.build_rocket(0).is_ok() {
        if state.build_rocket(0).is_ok() {
            self.pending_warning = false;
            return state.take_rocket();
        }

        // Couldn't build the rocket -> warn the explorer
        self.pending_warning = true;

        // Exploit handle_explorer_msg to send the secret message
        let _ = self.handle_explorer_msg(
            state,
            generator,
            combinator,
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 }
        );
        // Return that we don't have a rocket
        None
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