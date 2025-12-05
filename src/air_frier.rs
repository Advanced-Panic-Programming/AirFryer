use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    ComplexResourceType, Generator,
};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use std::collections::HashSet;
pub struct PlanetAI {
    has_explorer: bool,
    started: bool,
    pending_warning: bool, // To warn the explorer
    pending_asteroid: bool, // flag for a received asteroid
}
impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
            has_explorer: false,
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
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        //If the planet is stopped, I check if the message i receive is the start message, else I return None
        match msg {
            OrchestratorToPlanet::StartPlanetAI => {
                self.start(state);
                PlanetToOrchestrator::StartPlanetAIResult {
                    planet_id: state.id(),
                };
            }
            _ => {}
        }
        if self.started {
            match msg {
                OrchestratorToPlanet::Sunray(sunray) => {
                    //Cambiare lo scenario in cui sia presente l'espolatore in modo da tenere solo la cella carica indipendentemente dal razzo
                    if !state.cell(0).is_charged() {
                        state.cell_mut(0).charge(sunray);

                        if state.can_have_rocket() {
                            if !state.has_rocket() {
                                let _ = state.build_rocket(0);
                            }
                        }
                    } else {
                        //Definire meglio e gestire error in caso ci sia già un rocket
                        if !state.has_rocket() {
                            let _ = state.build_rocket(state.cells_count());
                            state.cell_mut(0).charge(sunray);
                        }
                    }
                    Some(PlanetToOrchestrator::SunrayAck { planet_id: 0 })
                }
                OrchestratorToPlanet::Asteroid(asteroid) => {
                    Some(PlanetToOrchestrator::AsteroidAck {
                        planet_id: state.id(),
                        rocket: self.handle_asteroid(state, generator, combinator),
                    })
                }
                OrchestratorToPlanet::StartPlanetAI => {
                    self.start(state);
                    Some(PlanetToOrchestrator::StartPlanetAIResult {
                        planet_id: state.id(),
                    })
                }
                OrchestratorToPlanet::StopPlanetAI => {
                    self.stop(state);
                    Some(PlanetToOrchestrator::StopPlanetAIResult {
                        planet_id: state.id(),
                    })
                }
                OrchestratorToPlanet::InternalStateRequest => {
                    todo!() //Michele
                }
                OrchestratorToPlanet::IncomingExplorerRequest { .. } => {
                    todo!() //Michele
                }
                OrchestratorToPlanet::OutgoingExplorerRequest { .. } => {
                    todo!() //?
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
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        //Questo OK, l'altro no, pianeta non starta se
        if self.started {
            if !self.has_explorer {
                self.has_explorer = true;
            }
            match msg {
                ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                    let mut hs = HashSet::new();
                    hs.insert(BasicResourceType::Carbon);
                    Some(PlanetToExplorer::SupportedResourceResponse { resource_list: hs })
                }
                ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                    let mut hs = HashSet::new();
                    hs.insert(ComplexResourceType::AIPartner);
                    hs.insert(ComplexResourceType::Diamond);
                    hs.insert(ComplexResourceType::Dolphin);
                    hs.insert(ComplexResourceType::Water);
                    hs.insert(ComplexResourceType::Life);
                    hs.insert(ComplexResourceType::Robot);
                    Some(PlanetToExplorer::SupportedCombinationResponse {
                        combination_list: hs,
                    })
                }
                ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id,
                    resource,
                } => {
                    if resource != BasicResourceType::Carbon {
                        Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                    } else {
                        let generated = generator.make_carbon(state.cell_mut(0));
                        match generated {
                            Ok(carbon) => Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(BasicResource::Carbon(carbon)),
                            }),
                            Err(_) => {
                                Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                            }
                        }
                    }
                }
                ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                    todo!()
                }
                ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                    Some(PlanetToExplorer::AvailableEnergyCellResponse {
                        available_cells: state.cells_count() as u32,
                    })
                }
            }
        } else {
            None
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
        self.started = true;
        self.has_explorer = false;
        //to do
    }

    fn stop(&mut self, state: &PlanetState) {
        self.started = false;
        self.has_explorer = false;
        //to do
    }
}
