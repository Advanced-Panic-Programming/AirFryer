mod air_frier;

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

    let mut compl: Vec<ComplexResourceType> = Vec::new();
    compl.push(ComplexResourceType::Water);
    compl.push(ComplexResourceType::Life);
    compl.push(ComplexResourceType::Dolphin);
    compl.push(ComplexResourceType::Robot);
    compl.push(ComplexResourceType::AIPartner);

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
    if planet.is_ok() {
        planet.unwrap().run();
    }
    //Planet::new(0, PlanetType::C, (), vec![], vec![], ((), ()), ((), ()));
}
#[cfg(test)]
mod tests {
    use super::*;
    use common_game::components::asteroid::Asteroid;
    use common_game::components::energy_cell::{self, EnergyCell};
    use common_game::components::generator::Generator;
    use common_game::components::resource::{BasicResource, Combinator, Hydrogen, Oxygen, Water};
    use common_game::components::sunray::Sunray;
    use common_game::protocols::messages::OrchestratorToPlanet::Asteroid as OtherAsteroid;
    use log::log;
    use std::path::{Component, Components};
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

        let mut gene: Vec<BasicResourceType> = Vec::new();
        gene.push(BasicResourceType::Carbon);

        let mut compl: Vec<ComplexResourceType> = Vec::new();
        compl.push(ComplexResourceType::Water);
        compl.push(ComplexResourceType::Life);
        compl.push(ComplexResourceType::Dolphin);
        compl.push(ComplexResourceType::Robot);
        compl.push(ComplexResourceType::AIPartner);

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
        let generator = common_game::components::generator::Generator::new();
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
                    destroyed,
                } => {
                    // assert!(r.is_none());
                    todo!()
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
        let generator = common_game::components::generator::Generator::new();
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
                    destroyed,
                } => {
                    // assert!(r.is_some());
                    todo!()
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
        let generator = common_game::components::generator::Generator::new();
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

    #[test]
    fn ask_for_water_from_explorer_success() {
        let planet = spawn_planet();

        let generator_ast_sry = common_game::components::generator::Generator::new();

        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: planet.snd_planet_to_exp,
            });

        let _ = planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(
            generator_ast_sry.as_ref().unwrap().generate_sunray(),
        ));

        // NOTE: our planet can only produce carbon, so we're
        // simulating the fact that the explorer has collected
        // the required resources from other planets
        // FIX: how can I generate basic resources without having the 'add' (pub for the
        // "common_crate") for the 'generator_resources'?
        let generator_resources = common_game::components::resource::Generator::new();
        let mut energy_cell = EnergyCell::new();

        let sunray = generator_ast_sry.as_ref().unwrap().generate_sunray();
        energy_cell.charge(sunray);
        let hydrogen = generator_resources.make_hydrogen(&mut energy_cell).unwrap();

        let sunray = generator_ast_sry.as_ref().unwrap().generate_sunray();
        energy_cell.charge(sunray);
        let oxygen = generator_resources.make_oxygen(&mut energy_cell).unwrap();

        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::CombineResourceRequest {
                explorer_id: 0,
                msg: common_game::components::resource::ComplexResourceRequest::Water(hydrogen, oxygen) ,
            });

        sleep(Duration::from_millis(100));

        let response = planet.rcv_planet_to_exp.recv();
        match response {
            Ok(msg) => match msg {
                PlanetToExplorer::CombineResourceResponse { complex_response } => {
                    match complex_response {
                        Ok(_water) => (), // TODO: some kind of test?
                        Err(_) => panic!("Test failed, we haven't received the water"),
                    }
                }
                _ => {
                    panic!("The 'msg' have an error")
                }
            },
            Err(_) => {
                panic!("The 'response' have an error")
            }
        }
    }

    #[test]
    fn ask_for_planet_available_energy_cell() {
        let planet = spawn_planet();
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        let res = planet.rcv_planet_to_exp.recv();
        match res {
            Ok(msg) => match msg {
                PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                    println!("Available energy cells: {:?}", available_cells);
                    assert_eq!(1, available_cells);
                }
                _ => {
                    println!("Wrong response");
                    assert!(false);
                }
            },
            Err(_) => {
                println!("Result error");
            }
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
