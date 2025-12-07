mod air_frier;

use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::messages;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use crossbeam_channel::{Receiver, Sender, unbounded};
fn main() {
    //New AI
    let ia = air_frier::PlanetAI::new();

    let mut gene: Vec<BasicResourceType> = Vec::new();
    gene.push(BasicResourceType::Carbon);

    let mut compl: Vec<ComplexResourceType> = Vec::new();
    compl.push(ComplexResourceType::Water);
    compl.push(ComplexResourceType::Life);
    compl.push(ComplexResourceType::Dolphin);
    compl.push(ComplexResourceType::Robot);
    compl.push(ComplexResourceType::AIPartner);

    let (sdr_expl_to_planet, rcv_expl_to_planet) = unbounded::<ExplorerToPlanet>();
    let (sdr_planet_to_expl, rcv_planet_to_expl) = unbounded::<PlanetToExplorer>();
    let (sdr_planet_to_orc, rcv_planet_to_orc) = unbounded::<PlanetToOrchestrator>();
    let (sdr_orc_to_planet, rcv_orc_to_planet) = unbounded::<OrchestratorToPlanet>();

    let planet = Planet::new(
        0,
        PlanetType::C,
        Box::new(ia),
        gene,
        compl,
        (rcv_orc_to_planet, sdr_planet_to_orc),
        rcv_expl_to_planet,
    );
    if planet.is_ok() {
        planet.unwrap().run();
    }
    //Planet::new(0, PlanetType::C, (), vec![], vec![], ((), ()), ((), ()));
}
#[cfg(test)]
mod tests {
    use super::*;
    use common_game::components::forge::Forge;
    use crossbeam_channel::RecvError;
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;

    pub struct TestContext {
        pub snd_orc_to_planet: Sender<OrchestratorToPlanet>,
        pub snd_exp_to_planet: Sender<ExplorerToPlanet>,
        pub snd_planet_to_exp: Sender<PlanetToExplorer>,
        pub rcv_planet_to_exp: Receiver<PlanetToExplorer>,
        pub rcv_planet_to_orc: Receiver<PlanetToOrchestrator>,
    }
    fn spawn_planet() -> TestContext {
        let ia = air_frier::PlanetAI::new();

        let mut gene: Vec<BasicResourceType> = Vec::new();
        gene.push(BasicResourceType::Carbon);

        let mut compl: Vec<ComplexResourceType> = Vec::new();
        compl.push(ComplexResourceType::Water);
        compl.push(ComplexResourceType::Life);
        compl.push(ComplexResourceType::Dolphin);
        compl.push(ComplexResourceType::Robot);
        compl.push(ComplexResourceType::AIPartner);

        let (sdr_expl_to_planet, rcv_expl_to_planet) = unbounded::<ExplorerToPlanet>();
        let (sdr_planet_to_expl, rcv_planet_to_expl) = unbounded::<PlanetToExplorer>();
        let (sdr_planet_to_orc, rcv_planet_to_orc) = unbounded::<PlanetToOrchestrator>();
        let (sdr_orc_to_planet, rcv_orc_to_planet) = unbounded::<OrchestratorToPlanet>();

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
        let t1 = thread::spawn(move || {
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
                    rocket: r,
                } => {
                    assert!(r.is_none());
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
                    rocket: r,
                } => {
                    assert!(r.is_some());
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
        sleep(Duration::from_secs(1));
        // Receive ACK
        let ack = planet.rcv_planet_to_orc.recv().unwrap();
        match ack {
            PlanetToOrchestrator::AsteroidAck { rocket, .. } => {
                if rocket.is_some() {
                    println!("Received asteroid ACK, with a rocket");
                } else {
                    println!("Received asteroid ACK, without a rocket");
                }
            }
            _ => panic!("Expected AsteroidAck"),
        }
    }
    #[test]
    fn incoming_explorer() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: planet.snd_planet_to_exp,
            });
        let _ = planet.rcv_planet_to_orc.recv();
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(PlanetToOrchestrator::IncomingExplorerResponse { planet_id, res }) => {
                assert_eq!(planet_id, 0); // Verifica ID
                assert!(res.is_ok(), "The result should be Ok");
                println!("The explorer has been accepted!");
            }
            Ok(_) => panic!("Wrong message,"),
            Err(e) => panic!("The planet didn't respond: {:?}", e),
        }
    }

    #[test]
    fn outgoing_explorer() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id: 0 });
        let _ = planet.rcv_planet_to_orc.recv();
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(PlanetToOrchestrator::OutgoingExplorerResponse { planet_id, res }) => {
                assert_eq!(planet_id, 0); // Verifica ID
                assert!(res.is_ok(), "The result should be Ok");
                println!("The explorer has been ejected!");
            }
            Ok(_) => panic!("Wrong message"),
            Err(e) => panic!("The planet didn't respond: {:?}", e),
        }
    }

    #[test]
    fn planet_internal_state_request() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::InternalStateRequest);
        let _ = planet.rcv_planet_to_orc.recv();
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(PlanetToOrchestrator::InternalStateResponse {
                planet_id,
                planet_state,
            }) => {
                assert_eq!(planet_id, 0);
                assert_eq!(
                    planet_state.has_rocket, false,
                    "the planet doesn't have a rocket"
                );
                //assert_eq!(planet_state.energy_cells.iter().map(|cell| cell.is_charged()).collect(), 1, "Correct!");
                //assert_eq!(planet_state.energy_cells.iter().filter(|cell| cell.is_cherged()).collect(), 0, "The planet has no energy cell charged");
            }
            Ok(_) => panic!("Wrong message"),
            Err(e) => panic!("The planet didn't respond: {:?}", e),
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
