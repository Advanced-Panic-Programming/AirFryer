use std::collections::HashSet;
use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, ComplexResourceType, Generator};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};

pub struct PlanetAI{
    has_explorer : bool,
    started: bool,
    pending_warning: bool, // To warn the explorer
    pending_asteroid: bool, // flag for a received asteroid
}
impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
            has_explorer : false,
            started: false,
            pending_warning: false,
            pending_asteroid: false,
        }
    }
}
impl planet::PlanetAI for PlanetAI {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: OrchestratorToPlanet
    ) -> Option<PlanetToOrchestrator> {
        if self.started {

            if self.pending_asteroid {
                let rocket = self.handle_asteroid(state, generator, combinator);
                self.pending_asteroid = false;
                return Some(PlanetToOrchestrator::AsteroidAck {
                    planet_id: state.id(),
                    rocket
                });
            }

            match msg {
                OrchestratorToPlanet::Sunray(sunray) => {
                    if ! state.cell(0).is_charged() {
                        state.cell_mut(0).charge(sunray);

                        if state.can_have_rocket() && !state.has_rocket() {
                            let _ = state.build_rocket(0);
                        }
                    } else {
                        let _ = state.build_rocket(0);
                        state.cell_mut(0).charge(sunray);
                    }
                    Some(PlanetToOrchestrator::SunrayAck { planet_id: 0})
                }
                OrchestratorToPlanet::Asteroid(_) => {
                    // Set asteroid flag and prepare one-cycle warning for explorer
                    self.pending_asteroid = true;
                    self.pending_warning = true;

                    // Try to build the rocket
                    let rocket = self.handle_asteroid(state, generator, combinator);
                    Some(PlanetToOrchestrator::AsteroidAck {
                        planet_id: state.id(),
                        rocket
                    })
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
        } else {
            None
        }
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        _combinator: &Combinator,
        msg: ExplorerToPlanet
    ) -> Option<PlanetToExplorer> {

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
                hs.insert(ComplexResourceType::Life);
                hs.insert(ComplexResourceType::Robot);
                hs.insert(ComplexResourceType::Water);

                // Secret channel:
                // If an asteroid is incoming, remove one element to signal danger.
                // We remove AIPartner to encode bit = 1 ("asteroid arriving").
                if self.pending_warning {
                    hs.remove(&ComplexResourceType::AIPartner);
                }
                Some(PlanetToExplorer::SupportedCombinationResponse { combination_list: hs })
            }
            ExplorerToPlanet::GenerateResourceRequest {explorer_id, resource} => {
                // Only Carbon can be generated on this planet
                if resource != BasicResourceType::Carbon {
                    Some(PlanetToExplorer::GenerateResourceResponse {resource: None})
                } else {
                    let generated = generator.make_carbon(state.cell_mut(0));
                    match generated {
                        Ok(carbon) => {
                            Some(PlanetToExplorer::GenerateResourceResponse {resource: Some(BasicResource::Carbon(carbon))})
                        }
                        Err(string) => {
                            println!("GenerateResourceRequest error: {}", string);
                            Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                        }
                    }
                }

            }
            ExplorerToPlanet::CombineResourceRequest { .. } => {
                todo!()
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                match state.full_cell() {
                    Some(_) => { Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: 1u32 }) }
                    None => { Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: 0u32 }) }
                }
            }
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator
    ) -> Option<Rocket> {

        if state.has_rocket() {
            // reset warning flags after using the rocket
            self.pending_warning = false;
            state.take_rocket()
        } else {
            // Try to build a rocket
            if state.build_rocket(0).is_ok() {
                self.pending_warning = false;
                return state.take_rocket();
            }

            // Couldn't build the rocket -> warn the explorer
            self.pending_warning = true;
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