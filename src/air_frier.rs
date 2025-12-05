use common_game::components::planet;
use common_game::components::planet::{PlanetState, PlanetType};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    ComplexResourceType, Generator, GenericResource,
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
}
impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
            has_explorer: false,
            started: false,
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
                        destroyed: todo!(),
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
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                let mut hs = HashSet::new();
                hs.insert(BasicResourceType::Carbon);
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: hs,
                })
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
                        Err(string) => {
                            Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                        }
                    }
                }
            }
            ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _,
                msg,
            } => match msg {
                common_game::components::resource::ComplexResourceRequest::Water(
                    hydrogen,
                    oxygen,
                ) => match combinator.make_water(hydrogen, oxygen, state.cell_mut(0)) {
                    Ok(water) => Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Ok(ComplexResource::Water(water)),
                    }),
                    Err((str, hydrogen, oxygen)) => {
                        Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Err((
                                str,
                                GenericResource::BasicResources(
                                    common_game::components::resource::BasicResource::Hydrogen(
                                        hydrogen,
                                    ),
                                ),
                                GenericResource::BasicResources(
                                    common_game::components::resource::BasicResource::Oxygen(
                                        oxygen,
                                    ),
                                ),
                            )),
                        })
                    }
                },
                common_game::components::resource::ComplexResourceRequest::Diamond(
                    carbon,
                    carbon1,
                ) => match combinator.make_diamond(carbon, carbon1, state.cell_mut(0)) {
                    Ok(diamond) => Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Ok(ComplexResource::Diamond(diamond)),
                    }),
                    Err((str, carbon, carbon1)) => {
                        Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Err((
                                str,
                                GenericResource::BasicResources(
                                    common_game::components::resource::BasicResource::Carbon(
                                        carbon,
                                    ),
                                ),
                                GenericResource::BasicResources(
                                    common_game::components::resource::BasicResource::Carbon(
                                        carbon1,
                                    ),
                                ),
                            )),
                        })
                    }
                },
                common_game::components::resource::ComplexResourceRequest::Life(water, carbon) => {
                    match combinator.make_life(water, carbon, state.cell_mut(0)) {
                        Ok(life) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Life(life)),
                        }),
                        Err((str, water, carbon)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::ComplexResources(
                                        common_game::components::resource::ComplexResource::Water(
                                            water,
                                        ),
                                    ),
                                    GenericResource::BasicResources(
                                        common_game::components::resource::BasicResource::Carbon(
                                            carbon,
                                        ),
                                    ),
                                )),
                            })
                        }
                    }
                }
                common_game::components::resource::ComplexResourceRequest::Robot(silicon, life) => {
                    match combinator.make_robot(silicon, life, state.cell_mut(0)) {
                        Ok(robot) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Robot(robot)),
                        }),
                        Err((str, silicon, life)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::BasicResources(
                                        common_game::components::resource::BasicResource::Silicon(
                                            silicon,
                                        ),
                                    ),
                                    GenericResource::ComplexResources(
                                        common_game::components::resource::ComplexResource::Life(
                                            life,
                                        ),
                                    ),
                                )),
                            })
                        }
                    }
                }
                common_game::components::resource::ComplexResourceRequest::Dolphin(water, life) => {
                    match combinator.make_dolphin(water, life, state.cell_mut(0)) {
                        Ok(dolphin) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Dolphin(dolphin)),
                        }),
                        Err((str, water, life)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::ComplexResources(
                                        common_game::components::resource::ComplexResource::Water(
                                            water,
                                        ),
                                    ),
                                    GenericResource::ComplexResources(
                                        common_game::components::resource::ComplexResource::Life(
                                            life,
                                        ),
                                    ),
                                )),
                            })
                        }
                    }
                }
                common_game::components::resource::ComplexResourceRequest::AIPartner(
                    robot,
                    diamond,
                ) => match combinator.make_aipartner(robot, diamond, state.cell_mut(0)) {
                    Ok(aipartner) => Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Ok(ComplexResource::AIPartner(aipartner)),
                    }),
                    Err((str, robot, diamond)) => Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Err((
                            str,
                            GenericResource::ComplexResources(
                                common_game::components::resource::ComplexResource::Robot(robot),
                            ),
                            GenericResource::ComplexResources(
                                common_game::components::resource::ComplexResource::Diamond(
                                    diamond,
                                ),
                            ),
                        )),
                    }),
                },
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_count() as u32,
                })
            }
            // ExplorerToPlanet::InternalStateRequest { .. } => {
            //     //Verrà tolto,l'esploratore non deve poter accedere allo stato interno del pianeta
            //     todo!()
            // }
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket> {
        if state.has_rocket() {
            Some(state.take_rocket()?)
        } else {
            // Error handling is done in the common-code trait implementation
            // i.e. planet can have rocket, planet energy cell is charged, ...

            // Try to build the rocket
            let _ = state.build_rocket(state.cells_count());
            state.take_rocket()
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
