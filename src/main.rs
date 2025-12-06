mod air_frier;
mod mock_planet;

use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use std::sync::mpsc;
fn main() {
    //New AI
    let ia = air_frier::PlanetAI::new();

    let mut gene: Vec<BasicResourceType> = Vec::new();
    gene.push(BasicResourceType::Carbon);

    let compl: Vec<ComplexResourceType> = vec![
        ComplexResourceType::Water,
        ComplexResourceType::Life,
        ComplexResourceType::Dolphin,
        ComplexResourceType::Robot,
        ComplexResourceType::AIPartner,
        ComplexResourceType::Diamond,
    ];

    let (sdr_expl_to_planet, rcv_expl_to_planet) = mpsc::channel::<ExplorerToPlanet>();
    let (sdr_planet_to_expl, rcv_planet_to_expl) = mpsc::channel::<PlanetToExplorer>();
    let (sdr_planet_to_orc, rcv_planet_to_orc) = mpsc::channel::<PlanetToOrchestrator>();
    let (sdr_orc_to_planet, rcv_orc_to_planet) = mpsc::channel::<OrchestratorToPlanet>();

    // FIXME: `::new` arguments
    // let planet = Planet::new(0, PlanetType::C, Box::new(ia), gene, rcv_expl_to_planet);    if planet.is_ok() {
    //    planet.unwrap().run();
    //}
    //Planet::new(0, PlanetType::C, (), vec![], vec![], ((), ()), ((), ()));
}
#[cfg(test)]
mod tests {
    use super::*;
    use common_game::components::asteroid::Asteroid;
    use common_game::components::forge::Forge;
    use common_game::components::resource::{BasicResource, Carbon, Generator};
    use common_game::components::sunray::Sunray;
    use common_game::protocols::messages::OrchestratorToPlanet::Asteroid as OtherAsteroid;
    use log::log;
    use std::path::{Component, Components};
    use std::sync::mpsc::RecvError;
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;

    pub struct TestContext {
        pub snd_orc_to_planet: mpsc::Sender<OrchestratorToPlanet>,
        pub snd_exp_to_planet: mpsc::Sender<ExplorerToPlanet>,
        pub snd_planet_to_exp: mpsc::Sender<PlanetToExplorer>,
        pub rcv_planet_to_exp: mpsc::Receiver<PlanetToExplorer>,
        pub rcv_planet_to_orc: mpsc::Receiver<PlanetToOrchestrator>,
    }

    fn spawn_planet() -> TestContext {
        let ia = air_frier::PlanetAI::new();

        let gene: Vec<BasicResourceType> = vec![BasicResourceType::Carbon];

        let compl: Vec<ComplexResourceType> = vec![
            ComplexResourceType::Water,
            ComplexResourceType::Life,
            ComplexResourceType::Dolphin,
            ComplexResourceType::Robot,
            ComplexResourceType::AIPartner,
            ComplexResourceType::Diamond,
        ];

        let (sdr_expl_to_planet, rcv_expl_to_planet) = mpsc::channel::<ExplorerToPlanet>();
        let (sdr_planet_to_expl, rcv_planet_to_expl) = mpsc::channel::<PlanetToExplorer>();
        let (sdr_planet_to_orc, rcv_planet_to_orc) = mpsc::channel::<PlanetToOrchestrator>();
        let (sdr_orc_to_planet, rcv_orc_to_planet) = mpsc::channel::<OrchestratorToPlanet>();

        let planet = Planet::new(
            0,
            PlanetType::C,
            Box::new(ia),
            gene,
            compl,
            (rcv_orc_to_planet, sdr_planet_to_orc),
            rcv_expl_to_planet,
        );
        sdr_orc_to_planet.send(OrchestratorToPlanet::StartPlanetAI);
        let _t1 = thread::spawn(move || {
            planet.unwrap().run();
        });
        sleep(Duration::from_millis(10));
        TestContext {
            snd_orc_to_planet: sdr_orc_to_planet,
            snd_exp_to_planet: sdr_expl_to_planet,
            snd_planet_to_exp: sdr_planet_to_expl,
            rcv_planet_to_orc: rcv_planet_to_orc,
            rcv_planet_to_exp: rcv_planet_to_expl,
        }
    }

    /// This method spawns the MockPlanet which provides all the possible
    /// kind of basic resources. This is required because [air_frier] planet
    /// can only generate 'Carbon' and in order to test the `CombineResourceRequest`
    /// we need to have also the others `BasicResouce`s
    fn spawn_resource_planet() -> TestContext {
        let ia = mock_planet::MockAI::new();

        // This planet generates everything except Carbon
        let gen_rules: Vec<BasicResourceType> = vec![
            BasicResourceType::Oxygen,
            BasicResourceType::Hydrogen,
            BasicResourceType::Silicon,
            BasicResourceType::Carbon,
        ];

        let comb_rules: Vec<ComplexResourceType> = vec![];

        let (sdr_expl_to_planet, rcv_expl_to_planet) = mpsc::channel::<ExplorerToPlanet>();
        let (sdr_planet_to_expl, rcv_planet_to_expl) = mpsc::channel::<PlanetToExplorer>();
        let (sdr_planet_to_orc, rcv_planet_to_orc) = mpsc::channel::<PlanetToOrchestrator>();
        let (sdr_orc_to_planet, rcv_orc_to_planet) = mpsc::channel::<OrchestratorToPlanet>();

        let new_planet = Planet::new(
            1,
            PlanetType::B,
            Box::new(ia),
            gen_rules,
            comb_rules,
            (rcv_orc_to_planet, sdr_planet_to_orc),
            rcv_expl_to_planet,
        );

        // FIXME: possible to comment this part because MockAI doesn't need
        // to run `start` (it doesn't have any field to set when starting)
        let _ = sdr_orc_to_planet.send(OrchestratorToPlanet::StartPlanetAI);

        match new_planet {
            Ok(mut planet) => {
                let _t1 = thread::spawn(move || {
                    let _ = planet.run();
                });
            }
            Err(err) => panic!("Error while creating the planet: \n {}", err),
        }

        sleep(Duration::from_millis(10));

        TestContext {
            snd_orc_to_planet: sdr_orc_to_planet,
            snd_exp_to_planet: sdr_expl_to_planet,
            snd_planet_to_exp: sdr_planet_to_expl,
            rcv_planet_to_orc: rcv_planet_to_orc,
            rcv_planet_to_exp: rcv_planet_to_expl,
        }
    }

    /// Used to have 2 planets:
    /// 1. [air_frier] => our planet
    /// 2. [mock_planet] => mock planet used to generate all the other
    /// basics resources not implemented in our planet
    fn spawn_dual_planets() -> (TestContext, TestContext) {
        let main_planet = spawn_planet();
        let resource_planet = spawn_resource_planet();
        (main_planet, resource_planet)
    }

    #[test]
    ///Sends an asteroid to the planet and checks that the planet responde with a none
    fn test_asteroid_with_no_rocket() {
        let mut planet = spawn_planet();
        let generator = common_game::components::forge::Forge::new();
        planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Asteroid(
                generator.unwrap().generate_asteroid(),
            ));
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(msg) => match msg {
                PlanetToOrchestrator::AsteroidAck {
                    planet_id: _,
                    destroyed: dest,
                } => {
                    assert_eq!(dest, true);
                }
                _ => {}
            },
            Err(_) => {}
        }
    }
    #[test]
    ///Sends a sunray to the planet, that makes a rocket with it, later it sends an asteroid and we check if che planet respond with a rocket
    fn test_asteroid_with_rocket() {
        let planet = spawn_planet();
        let generator = common_game::components::forge::Forge::new();
        let _ = planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(
            generator.as_ref().unwrap().generate_sunray(),
        ));
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Asteroid(
                generator.unwrap().generate_asteroid(),
            ));
        let _ = planet.rcv_planet_to_orc.recv();
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(msg) => match msg {
                PlanetToOrchestrator::AsteroidAck {
                    planet_id: _,
                    destroyed: destr,
                } => {
                    assert_eq!(destr, false);
                }
                _ => {}
            },
            Err(_) => {
                assert!(false);
            }
        }
    }
    #[test]
    fn ask_for_carbon_from_explorer() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: planet.snd_planet_to_exp,
            });
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Carbon,
            });
        let res = planet.rcv_planet_to_exp.recv();
        match res {
            Ok(msg) => match msg {
                PlanetToExplorer::GenerateResourceResponse { resource } => {
                    if resource.is_some() {
                        println!("Resource generated successfully!");
                        assert!(false);
                    } else {
                        println!("Resource not generated!");
                    }
                }
                _ => {
                    assert!(false);
                }
            },
            Err(_) => {
                println!("Result error");
            }
        }
    }
    #[test]
    fn ask_for_hydrogen_from_explorer() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: planet.snd_planet_to_exp,
            });
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Hydrogen,
            });
        let res = planet.rcv_planet_to_exp.recv();
        match res {
            Ok(msg) => match msg {
                PlanetToExplorer::GenerateResourceResponse { resource } => {
                    if resource.is_some() {
                        println!("Resource generated successfully!");
                        assert!(false);
                    } else {
                        println!("Resource not generated!");
                    }
                }
                _ => {
                    assert!(false);
                }
            },
            Err(_) => {
                println!("Result error");
            }
        }
    }

    /// Example test showing how to use dual planets:
    /// 1. Main planet generates Carbon
    /// 2. Resource planet generates Oxygen, Hydrogen, Silicon
    /// 3. Explorer can fetch resources from both and test combinations
    #[test]
    fn example_dual_planet_test_for_combine_resource() {
        let (main_planet, resource_planet) = spawn_dual_planets();
        let generator = Forge::new();

        // TODO to improve: create methods for common part (charging sunray, asking
        // for basic / complex resources)

        // Register explorer with both planets
        let _ = main_planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: main_planet.snd_planet_to_exp.clone(),
            });

        let _ =
            resource_planet
                .snd_orc_to_planet
                .send(OrchestratorToPlanet::IncomingExplorerRequest {
                    explorer_id: 0,
                    new_mpsc_sender: resource_planet.snd_planet_to_exp.clone(),
                });

        // Charge both planets with sunrays
        // NOTE: [air_frier] requires two energy cells:
        // - First sunray builds the rocket
        // - Second sunray charges the cell
        let _ = main_planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(
                generator.as_ref().unwrap().generate_sunray(),
            ));
        let _ = resource_planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(
                generator.as_ref().unwrap().generate_sunray(),
            ));

        // Second sunray charges the energy cell
        let _ = main_planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(
                generator.as_ref().unwrap().generate_sunray(),
            ));
        let _ = resource_planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(
                generator.as_ref().unwrap().generate_sunray(),
            ));

        sleep(Duration::from_millis(50));

        // Get Carbon from main planet
        let _ = main_planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Carbon,
            });
        let carbon_response = main_planet.rcv_planet_to_exp.recv().unwrap();
        println!("Carbon response received");
        let carbon_1 = match carbon_response {
            PlanetToExplorer::GenerateResourceResponse { resource } => match resource {
                Some(carbon) => Some(carbon),
                None => None,
            },
            _ => None,
        };

        // Getting the second Carbon to combine with the first and get
        // Diamond
        let _ = main_planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(
                generator.as_ref().unwrap().generate_sunray(),
            ));

        let _ = main_planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Carbon,
            });
        let carbon_response = main_planet.rcv_planet_to_exp.recv().unwrap();

        println!("Carbon response received");

        let carbon_2 = match carbon_response {
            PlanetToExplorer::GenerateResourceResponse { resource } => match resource {
                Some(carbon) => Some(carbon),
                None => None,
            },
            _ => None,
        };

        // Generating another sunray that will be used to generate 
        // the complex resource
        let _ = main_planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(
                generator.as_ref().unwrap().generate_sunray(),
            ));

        // TODO: delete this part for THIS test, because:
        // Diamond = Carbon + Carbon
        // Get Oxygen from resource planet
        let _ = resource_planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Oxygen,
            });
        let oxygen_response = resource_planet.rcv_planet_to_exp.recv().unwrap();
        let oxygen = match oxygen_response {
            PlanetToExplorer::GenerateResourceResponse { resource } => match resource {
                Some(oxygen) => Some(oxygen),
                None => None,
            },
            _ => None,
        };

        // Get Water (ComplexResourceRequest) from main planet
        // Extract Carbon from the BasicResource enum
        if let (
            Some(common_game::components::resource::BasicResource::Carbon(c1)),
            Some(common_game::components::resource::BasicResource::Carbon(c2)),
        ) = (carbon_1, carbon_2)
        {
            let _ = main_planet
                .snd_exp_to_planet
                .send(ExplorerToPlanet::CombineResourceRequest {
                    explorer_id: 0,
                    msg: common_game::components::resource::ComplexResourceRequest::Diamond(c1, c2),
                });
            let diamond_response = main_planet.rcv_planet_to_exp.recv().unwrap();
            println!("Diamond combination response received");
            match diamond_response {
                PlanetToExplorer::CombineResourceResponse { complex_response } => {
                    match complex_response {
                        Ok(_diamond) => {
                            println!("Diamond created successfully!");
                            assert!(true, "Diamond should have been created");
                        }
                        Err(e) => {
                            println!("Failed to create Diamond: {:?}", e);
                            assert!(false, "Diamond creation should not have failed");
                        }
                    }
                }
                _ => println!("Unexpected response type for CombineResourceRequest"),
            }
        } else {
            panic!("Carbon resources were not the expected type");
        }
    }

    #[test]
    fn ask_for_carbon_with_energy() {
        let planet = spawn_planet();
        let generator = common_game::components::forge::Forge::new();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: planet.snd_planet_to_exp,
            });
        let _ = planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(
            generator.as_ref().unwrap().generate_sunray(),
        ));
        let _ = planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(
            generator.as_ref().unwrap().generate_sunray(),
        ));
        sleep(Duration::from_millis(100));
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Carbon,
            });
        let res = planet.rcv_planet_to_exp.recv();
        match res {
            Ok(msg) => match msg {
                PlanetToExplorer::GenerateResourceResponse { resource } => {
                    if resource.is_some() {
                        println!("Resource generated successfully!");
                        assert!(true);
                    } else {
                        println!("Resource not generated!");
                        assert!(false);
                    }
                }
                _ => {
                    assert!(false);
                }
            },
            Err(_) => {
                println!("Result error");
            }
        }
    }

    fn match_available_energy_cell_response(res: Result<PlanetToExplorer, RecvError>) -> i32 {
        match res {
            Ok(msg) => match msg {
                PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                    available_cells as i32
                }
                _ => -1,
            },
            Err(err) => {
                println!("Result error: {}", err);
                -1
            }
        }
    }

    #[test]
    fn ask_for_planet_available_energy_cell() {
        let planet = spawn_planet();
        let generator = Forge::new();

        // Test with no sunray received
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        let mut res = planet.rcv_planet_to_exp.recv();
        assert_eq!(match_available_energy_cell_response(res), 0);

        // Test with 1 sunray received -> rocket was build -> expected 0
        let _ = planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(
            generator.as_ref().unwrap().generate_sunray(),
        ));
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        res = planet.rcv_planet_to_exp.recv();
        assert_eq!(match_available_energy_cell_response(res), 0);

        // Test with 2 sunray received -> rocket + 1 charge -> expected 1
        let _ = planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(
            generator.as_ref().unwrap().generate_sunray(),
        )); // Rocket built
        let _ = planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(
            generator.as_ref().unwrap().generate_sunray(),
        )); // EnergyCell built
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        res = planet.rcv_planet_to_exp.recv();
        assert_eq!(match_available_energy_cell_response(res), 1);
    }

    #[test]
    fn explorer_detects_no_asteroid_from_supported_combinations() {
        let planet = spawn_planet();

        // Explorer asks normally
        planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: 0 })
            .unwrap();

        let msg = planet.rcv_planet_to_exp.recv().unwrap();

        match msg {
            PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                println!("Combination list: {:?}", combination_list);

                // Full set size must be exactly 6
                assert_eq!(combination_list.len(), 6);

                // Explorer-side "decoder"
                let asteroid_detected = combination_list.len() != 6;

                assert!(!asteroid_detected, "Explorer incorrectly detected asteroid");
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn explorer_detects_asteroid_from_supported_combinations() {
        let planet = spawn_planet();
        let generator = Forge::new();

        // Send Asteroid
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Asteroid(
                generator.unwrap().generate_asteroid(),
            ));

        // Receive ACK
        let ack = planet.rcv_planet_to_orc.recv().unwrap();
        match ack {
            PlanetToOrchestrator::AsteroidAck { destroyed, .. } => {
                if !destroyed {
                    println!("Received asteroid ACK, with destroyed false");
                } else {
                    println!("Received asteroid ACK, with destroyed true");
                }
            }
            _ => panic!("Expected AsteroidAck"),
        }

        // 2. Now the explorer sends the SupportedCombinationRequest
        planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: 0 })
            .unwrap();

        let msg = planet.rcv_planet_to_exp.recv().unwrap();

        match msg {
            PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                println!("Combination list after asteroid: {:?}", combination_list);

                // When asteroid is pending, planet should REMOVE one item → len = 5
                assert_eq!(combination_list.len(), 5);

                // Explorer-side decoding:
                let asteroid_detected = combination_list.len() != 6;

                assert!(asteroid_detected, "Explorer failed to detect asteroid");
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn multiple_start_ai_messages_are_ignored() {}

    #[test]
    fn multiple_stop_ai_messages_are_ignored() {}
    #[test]
    fn arrival_of_exploer() {}
    #[test]
    fn departure_of_explorer() {}
}
