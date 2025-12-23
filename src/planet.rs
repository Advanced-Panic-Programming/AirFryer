use common_game::components::{
    planet::{self, PlanetState},
    resource::{
        BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
        ComplexResourceType, Generator, GenericResource,
    },
    rocket::Rocket,
};

use common_game::components::planet::DummyPlanetState;
use common_game::components::sunray::Sunray;
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use std::collections::HashSet;

#[allow(dead_code)]
pub struct PlanetAI {
    has_explorer: bool,
    started: bool,
    pending_warning: bool, // To warn the explorer
}

#[allow(dead_code)]
impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
            has_explorer: false,
            started: false,
            pending_warning: false,
        }
    }
}

impl Default for PlanetAI {
    fn default() -> Self {
        Self::new()
    }
}

impl planet::PlanetAI for PlanetAI {
    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: Sunray,
    ) {
        if !state.cell(0).is_charged() {
            state.charge_cell(sunray);
        } else if !state.has_rocket() {
            let _ = state.build_rocket(0);
            state.charge_cell(sunray);
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
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

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> DummyPlanetState {
        state.to_dummy()
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                let mut hs = HashSet::new();
                hs.insert(BasicResourceType::Carbon);
                Some(PlanetToExplorer::SupportedResourceResponse { resource_list: hs })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => {
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
                explorer_id: _,
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
                ComplexResourceRequest::Water(hydrogen, oxygen) => {
                    match combinator.make_water(hydrogen, oxygen, state.cell_mut(0)) {
                        Ok(water) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Water(water)),
                        }),
                        Err((str, hydrogen, oxygen)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::BasicResources(BasicResource::Hydrogen(
                                        hydrogen,
                                    )),
                                    GenericResource::BasicResources(BasicResource::Oxygen(oxygen)),
                                )),
                            })
                        }
                    }
                }
                ComplexResourceRequest::Diamond(carbon, carbon1) => {
                    match combinator.make_diamond(carbon, carbon1, state.cell_mut(0)) {
                        Ok(diamond) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Diamond(diamond)),
                        }),
                        Err((str, carbon, carbon1)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::BasicResources(BasicResource::Carbon(carbon)),
                                    GenericResource::BasicResources(BasicResource::Carbon(carbon1)),
                                )),
                            })
                        }
                    }
                }
                ComplexResourceRequest::Life(water, carbon) => {
                    match combinator.make_life(water, carbon, state.cell_mut(0)) {
                        Ok(life) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Life(life)),
                        }),
                        Err((str, water, carbon)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::ComplexResources(ComplexResource::Water(
                                        water,
                                    )),
                                    GenericResource::BasicResources(BasicResource::Carbon(carbon)),
                                )),
                            })
                        }
                    }
                }
                ComplexResourceRequest::Robot(silicon, life) => {
                    match combinator.make_robot(silicon, life, state.cell_mut(0)) {
                        Ok(robot) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Robot(robot)),
                        }),
                        Err((str, silicon, life)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::BasicResources(BasicResource::Silicon(
                                        silicon,
                                    )),
                                    GenericResource::ComplexResources(ComplexResource::Life(life)),
                                )),
                            })
                        }
                    }
                }
                ComplexResourceRequest::Dolphin(water, life) => {
                    match combinator.make_dolphin(water, life, state.cell_mut(0)) {
                        Ok(dolphin) => Some(PlanetToExplorer::CombineResourceResponse {
                            complex_response: Ok(ComplexResource::Dolphin(dolphin)),
                        }),
                        Err((str, water, life)) => {
                            Some(PlanetToExplorer::CombineResourceResponse {
                                complex_response: Err((
                                    str,
                                    GenericResource::ComplexResources(ComplexResource::Water(
                                        water,
                                    )),
                                    GenericResource::ComplexResources(ComplexResource::Life(life)),
                                )),
                            })
                        }
                    }
                }
                ComplexResourceRequest::AIPartner(robot, diamond) => match combinator
                    .make_aipartner(robot, diamond, state.cell_mut(0))
                {
                    Ok(aipartner) => Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Ok(ComplexResource::AIPartner(aipartner)),
                    }),
                    Err((str, robot, diamond)) => Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Err((
                            str,
                            GenericResource::ComplexResources(ComplexResource::Robot(robot)),
                            GenericResource::ComplexResources(ComplexResource::Diamond(diamond)),
                        )),
                    }),
                },
            },
        }
    }

    fn on_explorer_arrival(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: ID,
    ) {
        self.has_explorer = true;
    }

    fn on_explorer_departure(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: ID,
    ) {
        self.has_explorer = false;
    }

    fn on_start(&mut self, _state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        self.started = true;
        self.has_explorer = false;
    }

    fn on_stop(&mut self, _state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        self.started = false;
        self.has_explorer = false;
    }
}
