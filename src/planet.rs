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
use log::{info, warn};
use std::collections::HashSet;

pub struct PlanetAI {
    pending_warning: bool, // To warn the explorer
}

impl PlanetAI {
    pub fn new() -> PlanetAI {
        PlanetAI {
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
            info!(target: "planet", "[{}] Sunray charged the energy cell", state.id());
        } else if !state.has_rocket() {
            let _ = state.build_rocket(0);
            state.charge_cell(sunray);
            info!(target: "planet", "[{}] Sunray used to build a rocket", state.id());
        } else {
            info!(target: "planet", "[{}] Sunray received but cell already charged and rocket already built", state.id());
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
            info!(target: "planet", "[{}] Asteroid destroyed by rocket", state.id());
            state.take_rocket()
        } else {
            // Try to build a rocket
            if state.build_rocket(0).is_ok() {
                self.pending_warning = false;
                info!(target: "planet", "[{}] Rocket built just in time, asteroid destroyed", state.id());
                return state.take_rocket();
            }

            // Couldn't build the rocket -> warn the explorer
            self.pending_warning = true;
            warn!(target: "planet", "[{}] No rocket available, planet will be destroyed by the asteroid", state.id());
            None
        }
    }

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> DummyPlanetState {
        info!(target: "planet", "[{}] Internal state requested", state.id());
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
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        explorer_id: ID,
    ) {
        info!(target: "planet", "[{}] Explorer [{}] arrived", state.id(), explorer_id);
    }

    fn on_explorer_departure(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        explorer_id: ID,
    ) {
        info!(target: "planet", "[{}] Explorer [{}] departed", state.id(), explorer_id);
    }

    fn on_start(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        info!(target: "planet", "[{}] Planet AI started", state.id());
    }

    fn on_stop(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        info!(target: "planet", "[{}] Planet AI stopped", state.id());
    }
}
