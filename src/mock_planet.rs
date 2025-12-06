use common_game::{
    components::{self, planet, resource},
    protocols::messages::{self, OrchestratorToPlanet, PlanetToExplorer},
};

pub struct MockAI {}

#[allow(dead_code)]
impl MockAI {
    pub fn new() -> Self {
        Self {}
    }
}

impl planet::PlanetAI for MockAI {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut planet::PlanetState,
        _generator: &resource::Generator,
        _combinator: &resource::Combinator,
        msg: messages::OrchestratorToPlanet,
    ) -> Option<messages::PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                let charge = state.charge_cell(sunray);
                match charge {
                    Some(_) => (), // cell already charged
                    None => (),    // cell charged
                }
                Some(messages::PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                })
            }
            _ => None,
        }
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut planet::PlanetState,
        generator: &resource::Generator,
        _combinator: &resource::Combinator,
        msg: messages::ExplorerToPlanet,
    ) -> Option<messages::PlanetToExplorer> {
        match msg {
            messages::ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => todo!(),
            messages::ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => todo!(),
            messages::ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: _,
                resource,
            } => match resource {
                resource::BasicResourceType::Oxygen => match state.full_cell() {
                    Some((energy_cell, _)) => {
                        let oxygen = generator.make_oxygen(energy_cell);
                        match oxygen {
                            Ok(oxygen) => Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(resource::BasicResource::Oxygen(oxygen)),
                            }),
                            Err(_) => {
                                Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                            }
                        }
                    }
                    None => Some(PlanetToExplorer::GenerateResourceResponse { resource: None }),
                },
                resource::BasicResourceType::Hydrogen => match state.full_cell() {
                    Some((energy_cell, _)) => {
                        let hydrogen = generator.make_hydrogen(energy_cell);
                        match hydrogen {
                            Ok(hydrogen) => Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(resource::BasicResource::Hydrogen(hydrogen)),
                            }),
                            Err(_) => {
                                Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                            }
                        }
                    }
                    None => Some(PlanetToExplorer::GenerateResourceResponse { resource: None }),
                },
                resource::BasicResourceType::Carbon => match state.full_cell() {
                    Some((energy_cell, _)) => {
                        let carbon = generator.make_carbon(energy_cell);
                        match carbon {
                            Ok(carbon) => Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(resource::BasicResource::Carbon(carbon)),
                            }),
                            Err(_) => {
                                Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                            }
                        }
                    }
                    None => Some(PlanetToExplorer::GenerateResourceResponse { resource: None }),
                },
                resource::BasicResourceType::Silicon => match state.full_cell() {
                    Some((energy_cell, _)) => {
                        let silicon = generator.make_silicon(energy_cell);
                        match silicon {
                            Ok(silicon) => Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(resource::BasicResource::Silicon(silicon)),
                            }),
                            Err(_) => {
                                Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                            }
                        }
                    }
                    None => Some(PlanetToExplorer::GenerateResourceResponse { resource: None }),
                },
            },
            messages::ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _,
                msg: _,
            } => todo!(),
            messages::ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _ } => todo!(),
        }
    }

    fn handle_asteroid(
        &mut self,
        _state: &mut planet::PlanetState,
        _generator: &resource::Generator,
        _combinator: &resource::Combinator,
    ) -> Option<components::rocket::Rocket> {
        todo!()
    }

    fn start(&mut self, _state: &planet::PlanetState) {
        ()
    }

    fn stop(&mut self, _state: &planet::PlanetState) {
        ()
    }
}
