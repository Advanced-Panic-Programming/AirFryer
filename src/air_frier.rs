use common_game::components::planet;
use common_game::components::planet::{Planet, PlanetState};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceType, Generator,
    GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use std::collections::HashSet;

pub struct PlanetAI {
    has_explorer: bool,
    started: bool,
    pending_warning: bool, // To warn the explorer
}
impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
            has_explorer: false,
            started: false,
            pending_warning: false,
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
        //If the planet is stopped, I check if the message I receive is the start message, else I return None
        match msg {
            OrchestratorToPlanet::StartPlanetAI => {
                self.start(state);
                PlanetToOrchestrator::StartPlanetAIResult {
                    planet_id: state.id(),
                };
            }
            OrchestratorToPlanet::KillPlanet => {
                PlanetToOrchestrator::KillPlanetResult {
                    planet_id: state.id(),
                };
            }
            _ => {}
        }
        if self.started {
            match msg {
                OrchestratorToPlanet::Sunray(sunray) => {
                    // First scenario: empty energy cell -> charge it
                    if !state.cell(0).is_charged() {
                        state.cell_mut(0).charge(sunray);

                        // We don't build the rocket here. We wait for a possible explorer in order to let him use the charge for generating a resource
                    } else {
                        // Second scenario: energy cell already charged -> discharge it by creating a rocket (if possible) and recharge it.
                        if !state.has_rocket() {
                            let _ = state.build_rocket(0);
                            state.cell_mut(0).charge(sunray);
                        }
                    }
                    // Send the SunrayAck
                    Some(PlanetToOrchestrator::SunrayAck {
                        planet_id: state.id(),
                    })
                }
                OrchestratorToPlanet::Asteroid(_) => {
                    // Set asteroid flag and prepare one-cycle warning for explorer
                    self.pending_warning = true;

                    // Try to build the rocket
                    let rocket = self.handle_asteroid(state, generator, combinator);
                    Some(PlanetToOrchestrator::AsteroidAck {
                        planet_id: state.id(),
                        rocket,
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
                    Some(PlanetToOrchestrator::InternalStateResponse {
                        planet_id: state.id(),
                        planet_state: state.to_dummy(),
                    }) //Michele
                }
                OrchestratorToPlanet::IncomingExplorerRequest { .. } => {
                    self.has_explorer = true;
                    Some(PlanetToOrchestrator::IncomingExplorerResponse {
                        planet_id: state.id(),
                        res: Ok(()),
                    }) //Michele
                }
                OrchestratorToPlanet::OutgoingExplorerRequest { .. } => {
                    self.has_explorer = false;
                    Some(PlanetToOrchestrator::OutgoingExplorerResponse {
                        planet_id: state.id(),
                        res: Ok(()),
                    }) //?
                }
                OrchestratorToPlanet::KillPlanet => Some(PlanetToOrchestrator::KillPlanetResult {
                    planet_id: state.id(),
                }),
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
                Some(PlanetToExplorer::SupportedResourceResponse { resource_list: hs })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
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
                    // Reset flag
                    self.pending_warning = false;
                }
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
            ExplorerToPlanet::AvailableEnergyCellRequest {
                explorer_id: _explorer_id,
            } => match state.full_cell() {
                Some(_) => Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: 1u32,
                }),
                None => Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: 0u32,
                }),
            },
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
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
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
    }

    fn stop(&mut self, state: &PlanetState) {
        self.started = false;
        self.has_explorer = false;
    }
}
