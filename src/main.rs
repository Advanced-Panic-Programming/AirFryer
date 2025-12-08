mod air_frier;
mod mock_planet;

use common_game::components::planet::PlanetType;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use crossbeam_channel::unbounded;
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

    let (sdr_expl_to_planet, rcv_expl_to_planet) = unbounded::<ExplorerToPlanet>();
    let (sdr_planet_to_expl, rcv_planet_to_expl) = unbounded::<PlanetToExplorer>();
    let (sdr_planet_to_orc, rcv_planet_to_orc) = unbounded::<PlanetToOrchestrator>();
    let (sdr_orc_to_planet, rcv_orc_to_planet) = unbounded::<OrchestratorToPlanet>();

    // FIXME: `::new` arguments
    // let planet = Planet::new(0, PlanetType::C, Box::new(ia), gene, rcv_expl_to_planet);    if planet.is_ok() {
    //    planet.unwrap().run();
    //}
    //Planet::new(0, PlanetType::C, (), vec![], vec![], ((), ()), ((), ()));
}
#[cfg(test)]
mod tests {
    use super::*;
    use common_game::components::forge::Forge;
    use common_game::components::planet::Planet;
    use common_game::components::resource::{BasicResource, Carbon};
    use common_game::protocols::messages::ExplorerToPlanet::CombineResourceRequest;
    use crossbeam_channel::RecvError;
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use lazy_static::lazy_static;
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;
    // =========================================================================
    // GLOBAL STATIC, STRUCT & FUNCTIONS (to create planets) FOR TEST OPERATIONS
    // =========================================================================

    // Forge enforces a single global instance (see `forge::Forge`), so the tests
    // share one lazily-initialized Forge
    lazy_static! {
        static ref GENERATOR: Forge = Forge::new().expect("Failed to create Forge");
    }

    pub struct TestContext {
        pub snd_orc_to_planet: Sender<OrchestratorToPlanet>,
        pub snd_exp_to_planet: Sender<ExplorerToPlanet>,
        pub snd_planet_to_exp: Sender<PlanetToExplorer>,
        pub rcv_planet_to_exp: Receiver<PlanetToExplorer>,
        pub rcv_planet_to_orc: Receiver<PlanetToOrchestrator>,
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
        let _ = sdr_orc_to_planet.send(OrchestratorToPlanet::StartPlanetAI);
        let _t1 = thread::spawn(move || {
            let _ = planet.unwrap().run();
        });
        sleep(Duration::from_millis(10));

        // StartPlanetAIResponse message consumed from the queue
        let _ = rcv_planet_to_orc.recv();

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

        let (sdr_expl_to_planet, rcv_expl_to_planet) = unbounded::<ExplorerToPlanet>();
        let (sdr_planet_to_expl, rcv_planet_to_expl) = unbounded::<PlanetToExplorer>();
        let (sdr_planet_to_orc, rcv_planet_to_orc) = unbounded::<PlanetToOrchestrator>();
        let (sdr_orc_to_planet, rcv_orc_to_planet) = unbounded::<OrchestratorToPlanet>();

        let new_planet = Planet::new(
            1,
            PlanetType::B,
            Box::new(ia),
            gen_rules,
            comb_rules,
            (rcv_orc_to_planet, sdr_planet_to_orc),
            rcv_expl_to_planet,
        );

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

    // ===========================================
    // HELPER FUNCTIONS FOR COMMON TEST OPERATIONS
    // ===========================================
    //
    // TODO: it would be ideal to use the common test operation inside
    // all the test methods

    /// Registers an explorer with a planet so it can send/receive messages
    fn register_explorer_with_planet(planet: &TestContext, explorer_id: u32) {
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id,
                new_mpsc_sender: planet.snd_planet_to_exp.clone(),
            });

        // IncomingExplorerResponse message consumed from the queue
        let _ = planet.rcv_planet_to_orc.recv();
    }

    /// Charges a planet with N sunrays
    /// NOTE: the [air_frier::PlanetAI] of [air_frier] waits to build the rocket until an asteroid is coming
    fn charge_planet_with_sunrays(planet: &TestContext, count: usize) {
        for _ in 0..count {
            let _ = planet
                .snd_orc_to_planet
                .send(OrchestratorToPlanet::Sunray(GENERATOR.generate_sunray()));
        }
        sleep(Duration::from_millis(50));
    }

    /// Requests a basic resource from a planet and returns the response
    fn get_basic_resource(
        planet: &TestContext,
        explorer_id: u32,
        resource_type: BasicResourceType,
    ) -> Option<BasicResource> {
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource: resource_type,
            });

        let response = planet.rcv_planet_to_exp.recv().unwrap();
        match response {
            PlanetToExplorer::GenerateResourceResponse { resource } => resource,
            _ => None,
        }
    }

    /// Requests a complex resource from a planet and returns the response
    /// The error type is a tuple: (error_message, left_resource, right_resource)
    fn combine_resources(
        planet: &TestContext,
        explorer_id: u32,
        request: common_game::components::resource::ComplexResourceRequest,
    ) -> Result<
        common_game::components::resource::ComplexResource,
        (
            String,
            common_game::components::resource::GenericResource,
            common_game::components::resource::GenericResource,
        ),
    > {
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::CombineResourceRequest {
                explorer_id,
                msg: request,
            });

        let response = planet.rcv_planet_to_exp.recv().unwrap();
        match response {
            PlanetToExplorer::CombineResourceResponse { complex_response } => complex_response,
            _ => panic!("Unexpected response type for CombineResourceRequest"),
        }
    }

    /// Helper to extract Carbon from BasicResource enum
    #[allow(dead_code)]
    fn extract_carbon(resource: Option<BasicResource>) -> Option<Carbon> {
        match resource {
            Some(BasicResource::Carbon(c)) => Some(c),
            _ => None,
        }
    }

    /// Helper to extract Oxygen from BasicResource enum
    #[allow(dead_code)]
    fn extract_oxygen(
        resource: Option<BasicResource>,
    ) -> Option<common_game::components::resource::Oxygen> {
        match resource {
            Some(BasicResource::Oxygen(o)) => Some(o),
            _ => None,
        }
    }

    /// Helper to extract Hydrogen from BasicResource enum
    #[allow(dead_code)]
    fn extract_hydrogen(
        resource: Option<BasicResource>,
    ) -> Option<common_game::components::resource::Hydrogen> {
        match resource {
            Some(BasicResource::Hydrogen(h)) => Some(h),
            _ => None,
        }
    }

    /// Helper to extract Silicon from BasicResource enum
    #[allow(dead_code)]
    fn extract_silicon(
        resource: Option<BasicResource>,
    ) -> Option<common_game::components::resource::Silicon> {
        match resource {
            Some(BasicResource::Silicon(s)) => Some(s),
            _ => None,
        }
    }

    /// Helper to extract Water from ComplexResource enum
    #[allow(dead_code)]
    fn extract_water(
        resource: common_game::components::resource::ComplexResource,
    ) -> Option<common_game::components::resource::Water> {
        match resource {
            common_game::components::resource::ComplexResource::Water(w) => Some(w),
            _ => None,
        }
    }

    /// Helper to extract Life from ComplexResource enum
    #[allow(dead_code)]
    fn extract_life(
        resource: common_game::components::resource::ComplexResource,
    ) -> Option<common_game::components::resource::Life> {
        match resource {
            common_game::components::resource::ComplexResource::Life(l) => Some(l),
            _ => None,
        }
    }

    /// Helper to extract Dolphin from ComplexResource enum
    #[allow(dead_code)]
    fn extract_dolphin(
        resource: common_game::components::resource::ComplexResource,
    ) -> Option<common_game::components::resource::Dolphin> {
        match resource {
            common_game::components::resource::ComplexResource::Dolphin(d) => Some(d),
            _ => None,
        }
    }

    /// Helper to extract Robot from ComplexResource enum
    #[allow(dead_code)]
    fn extract_robot(
        resource: common_game::components::resource::ComplexResource,
    ) -> Option<common_game::components::resource::Robot> {
        match resource {
            common_game::components::resource::ComplexResource::Robot(r) => Some(r),
            _ => None,
        }
    }

    /// Helper to extract Diamond from ComplexResource enum
    #[allow(dead_code)]
    fn extract_diamond(
        resource: common_game::components::resource::ComplexResource,
    ) -> Option<common_game::components::resource::Diamond> {
        match resource {
            common_game::components::resource::ComplexResource::Diamond(d) => Some(d),
            _ => None,
        }
    }

    /// Helper to extract AIPartner from ComplexResource enum
    #[allow(dead_code)]
    fn extract_aipartner(
        resource: common_game::components::resource::ComplexResource,
    ) -> Option<common_game::components::resource::AIPartner> {
        match resource {
            common_game::components::resource::ComplexResource::AIPartner(a) => Some(a),
            _ => None,
        }
    }

    // ===========================================
    // START OF TESTING
    // ===========================================

    // TODO: structure the code:
    // ========
    // CATEGORY
    // ========
    // for each "test category"
    // EG:
    // - asteroid
    // - rocket
    // - resource request
    //  - basic resource
    //  - complex resource
    //  ...
    ///Sends an asteroid to the planet and checks that the planet responds with a none
    #[test]
    fn test_asteroid_with_no_rocket() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Asteroid(
                GENERATOR.generate_asteroid(),
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
                _ => {
                    panic!("Other message received!")
                }
            },
            Err(_) => {}
        }
    }
    ///Send a sunray to the planet, later send an asteroid and check if che planet responds with a rocket
    #[test]
    fn test_asteroid_with_rocket() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(GENERATOR.generate_sunray()));
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Asteroid(
                GENERATOR.generate_asteroid(),
            ));
        let res_sunray = planet.rcv_planet_to_orc.recv(); //Reading the response to the sunray
        match res_sunray {
            Ok(PlanetToOrchestrator::SunrayAck { .. }) => {
                assert!(true);
            }
            _ => {
                panic!("Sunray Ack not received");
            }
        }
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(msg) => match msg {
                PlanetToOrchestrator::AsteroidAck {
                    planet_id: _,
                    rocket: r,
                } => {
                    assert!(r.is_some());
                }
                _ => {
                    panic!("Other message received!")
                }
            },
            Err(_) => {
                panic!("Error!");
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
                        panic!("Resource generated successfully!");
                    } else {
                        println!("Resource not generated!");
                        assert!(true);
                    }
                }
                _ => {
                    panic!("Other message received!")
                }
            },
            Err(_) => {
                panic!("Result error");
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
                        panic!("Resource generated successfully!");
                    } else {
                        println!("Resource not generated!");
                        assert!(true);
                    }
                }
                _ => {
                    panic!("Other message received!");
                }
            },
            Err(_) => {
                panic!("Result error");
            }
        }
    }
    #[test]
    fn ask_for_carbon_with_energy() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: planet.snd_planet_to_exp,
            });
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(GENERATOR.generate_sunray()));
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Sunray(GENERATOR.generate_sunray()));
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

    // ========================================
    // PLANET SUNRAYS MANAGEMENT
    // ========================================

    // Helper function
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

    /// Tests how the planet manages sunrays
    /// 0 Sunray / 1 Sunray / 2 Sunray / 3+ Sunray
    #[test]
    fn ask_for_planet_available_energy_cell() {
        let planet = spawn_planet();

        register_explorer_with_planet(&planet, 0);

        // Test with no sunray received
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        let res = planet.rcv_planet_to_exp.recv();
        assert_eq!(match_available_energy_cell_response(res), 0);

        // Test with 1 sunray received -> rocket was NOT build -> expected 1
        charge_planet_with_sunrays(&planet, 1); // Energy cell charged
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        let res = planet.rcv_planet_to_exp.recv();
        assert_eq!(match_available_energy_cell_response(res), 1);

        // Test with 2 sunray received -> rocket + 1 charge -> expected 1
        charge_planet_with_sunrays(&planet, 1); // Energy cell charged
        charge_planet_with_sunrays(&planet, 1); // Rocket built + Energy Cell Recharged
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        let res = planet.rcv_planet_to_exp.recv();
        assert_eq!(match_available_energy_cell_response(res), 1);

        // Test with 3+ sunray received -> rocket + 1 charge -> expected 1
        // Note: Our Planet has only 1 energy cell
        charge_planet_with_sunrays(&planet, 3);
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 });
        let res = planet.rcv_planet_to_exp.recv();
        assert_eq!(match_available_energy_cell_response(res), 1);
    }

    // ========================================
    // PLANET SECRET WARNING
    // ========================================

    // Helper function
    // Returns the length of the SupportedCombinations Hashset
    fn match_supported_combination_request_response(
        msg_res: Result<PlanetToExplorer, RecvError>,
    ) -> i32 {
        match msg_res {
            Ok(msg) => match msg {
                PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                    combination_list.len() as i32
                }
                _ => panic!("Wrong message type"),
            },
            Err(err) => {
                println!("Planet response error: {}", err);
                panic!("No Response");
            }
        }
    }

    #[test]
    fn explorer_detects_no_asteroid_from_supported_combinations() {
        let planet = spawn_planet();

        register_explorer_with_planet(&planet, 0);

        // Explorer asks normally
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: 0 });

        let msg_res = planet.rcv_planet_to_exp.recv();

        let res = match_supported_combination_request_response(msg_res);
        // Full set size must be exactly 6
        assert_eq!(res, 6);

        // Explorer-side "decoder"
        let asteroid_detected = res != 6;
        assert!(!asteroid_detected, "Explorer was warned uselessly");
    }

    // debug function
    fn match_planet_to_orc_message(msg: PlanetToOrchestrator) -> String {
        match msg {
            PlanetToOrchestrator::AsteroidAck { .. } => String::from("AsteroidAck"),
            PlanetToOrchestrator::SunrayAck { .. } => String::from("SunrayAck"),
            PlanetToOrchestrator::StartPlanetAIResult { .. } => String::from("StartPlanetAIResult"),
            PlanetToOrchestrator::StopPlanetAIResult { .. } => String::from("StopPlanetAIResult"),
            PlanetToOrchestrator::KillPlanetResult { .. } => String::from("KillPlanetResult"),
            PlanetToOrchestrator::InternalStateResponse { .. } => {
                String::from("InternalStateResponse")
            }
            PlanetToOrchestrator::IncomingExplorerResponse { .. } => {
                String::from("IncomingExplorerResponse")
            }
            PlanetToOrchestrator::OutgoingExplorerResponse { .. } => {
                String::from("OutgoingExplorerResponse")
            }
            PlanetToOrchestrator::Stopped { .. } => String::from("Stopped"),
        }
    }

    #[test]
    fn explorer_detects_asteroid_from_supported_combinations() {
        let planet = spawn_planet();

        register_explorer_with_planet(&planet, 0);

        // Send Asteroid
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::Asteroid(
                GENERATOR.generate_asteroid(),
            ));
        sleep(Duration::from_secs(1));

        // Explorer requests CombinationRules normally
        let _ = planet
            .snd_exp_to_planet
            .send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: 0 });

        // Orchestrator receives AsteroidAck
        let ack_res = planet.rcv_planet_to_orc.recv();
        match ack_res {
            Ok(ack) => match ack {
                PlanetToOrchestrator::AsteroidAck { rocket, .. } => {
                    if rocket.is_some() {
                        println!("Received asteroid ACK, with a rocket");
                    } else {
                        println!("Received asteroid ACK, without a rocket");
                    }
                }
                _ => {
                    println!("Received: {}", match_planet_to_orc_message(ack));
                    panic!("Expected AsteroidAck");
                }
            },
            Err(err) => {
                println!("Planet response error: {}", err);
                panic!("No response");
            }
        }

        // Before the KillPlanetResult the planet should have sent the warning response to the explorer SupportedCombinationRequest request
        let msg_res = planet.rcv_planet_to_exp.recv();
        let res = match_supported_combination_request_response(msg_res);
        // Explorer should receive only 5 combination rules
        assert_eq!(res, 5);

        // Explorer-side "decoder"
        let asteroid_detected = res != 6; // Should evaluate to true
        assert!(asteroid_detected, "Explorer was NOT warned");
        println!("EXPLORER SUCCESSFULLY WARNED!!!");

        // Orchestrator sends KillPlanet
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::KillPlanet);

        //TODO: Explorer asks to change planet
        // ...

        //TODO: Orchestrator manages the explorer request before receiving the KillPlanetResult
        // ...
        println!("Explorer escaped in time!");

        // Orchestrator receives the KillPlanetResult
        let kill_res = planet.rcv_planet_to_orc.recv();
        match kill_res {
            Ok(ack) => match ack {
                PlanetToOrchestrator::KillPlanetResult { planet_id } => {
                    println!("Received kill planet by {}", planet_id);
                }
                _ => panic!("Expected KillPlanetResult"),
            },
            Err(err) => {
                println!("Planet response error: {}", err);
                panic!("No response");
            }
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

    // ========================================
    // COMPLEX RESOURCE COMBINATION TESTS
    // ========================================

    /// Test Water combination: Hydrogen + Oxygen => Water
    #[test]
    fn combine_resource_water_success() {
        let (main_planet, resource_planet) = spawn_dual_planets();
        let explorer_id = 0;

        // Setup
        register_explorer_with_planet(&main_planet, explorer_id);
        register_explorer_with_planet(&resource_planet, explorer_id);
        charge_planet_with_sunrays(&main_planet, 1);
        charge_planet_with_sunrays(&resource_planet, 1);

        // Get Hydrogen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let hydrogen =
            get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Hydrogen);

        // Get Oxygen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let oxygen = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Oxygen);

        // Extract and combine
        if let (Some(h), Some(o)) = (extract_hydrogen(hydrogen), extract_oxygen(oxygen)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Water(h, o),
            );

            match result {
                Ok(_water) => println!("Water created successfully!"),
                Err(e) => panic!("Water creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Hydrogen and Oxygen resources");
        }
    }

    /// Diamond = Carbon + Carbon
    #[test]
    fn combine_resource_diamond_success() {
        let (main_planet, _resource_planet) = spawn_dual_planets();
        let explorer_id = 0;

        // Setup
        register_explorer_with_planet(&main_planet, explorer_id);
        charge_planet_with_sunrays(&main_planet, 1); // Rocket

        // Get first Carbon
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon_1 = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Get second Carbon
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon_2 = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Extract Carbon values and combine
        if let (Some(c1), Some(c2)) = (extract_carbon(carbon_1), extract_carbon(carbon_2)) {
            charge_planet_with_sunrays(&main_planet, 1); // Energy for combination
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Diamond(c1, c2),
            );

            match result {
                Ok(_diamond) => println!("Diamond created successfully!"),
                Err(e) => panic!("Diamond creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Carbon resources");
        }
    }

    /// Diamond = Carbon + Carbon
    /// Fail: The only possible error we can get when creating a combined resource
    /// is the planet not having the required energy cell (we can't have the
    /// "recipe error", because our planet implements all the combination requests).
    /// NOTE: we only implemented this failure test because `make_complex_resource`
    /// is macro-generated, so the behavior is identical for all resource
    /// combination functions
    #[test]
    fn combine_resource_diamond_fail() {
        let (main_planet, _resource_planet) = spawn_dual_planets();
        let explorer_id = 0;

        // Setup
        register_explorer_with_planet(&main_planet, explorer_id);
        charge_planet_with_sunrays(&main_planet, 1); // Rocket

        // Get first Carbon
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon_1 = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Get second Carbon
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon_2 = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Extract Carbon values and combine
        if let (Some(c1), Some(c2)) = (extract_carbon(carbon_1), extract_carbon(carbon_2)) {
            // charge_planet_with_sunrays(&main_planet, &generator, 1); // <--- no energy cell to
            // use to combine the elements
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Diamond(c1, c2),
            );

            match result {
                Ok(_diamond) => panic!("Diamond created successfully... impossible!"),
                Err((str, _, _)) => assert_eq!("EnergyCell not charged!", str),
            }
        } else {
            panic!("Failed to extract Carbon resources");
        }
    }

    /// Test Life combination: Water + Carbon => Life
    #[test]
    fn combine_resource_life() {
        let (main_planet, resource_planet) = spawn_dual_planets();
        let explorer_id = 0;

        // Setup
        register_explorer_with_planet(&main_planet, explorer_id);
        register_explorer_with_planet(&resource_planet, explorer_id);
        charge_planet_with_sunrays(&main_planet, 1);
        charge_planet_with_sunrays(&resource_planet, 1);

        // Get Water from main planet
        // Get Hydrogen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let hydrogen =
            get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Hydrogen);

        // Get Oxygen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let oxygen = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Oxygen);

        // Extract and combine
        let water = if let (Some(h), Some(o)) = (extract_hydrogen(hydrogen), extract_oxygen(oxygen))
        {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Water(h, o),
            );

            match result {
                Ok(water) => water,
                Err(e) => panic!("Water creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Hydrogen and Oxygen resources");
        };

        // Get Carbon from main planet
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Combine Water + Carbon => Life
        if let (Some(c), Some(w)) = (extract_carbon(carbon), extract_water(water)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Life(w, c),
            );

            match result {
                Ok(_life) => println!("Life created successfully!"),
                Err(e) => panic!("Life creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Water or Carbon resources");
        }
    }

    /// Test Dolphin combination: Water + Life => Dolphin
    #[test]
    fn combine_resource_dolphin() {
        let (main_planet, resource_planet) = spawn_dual_planets();
        let explorer_id = 0;

        // Setup
        register_explorer_with_planet(&main_planet, explorer_id);
        register_explorer_with_planet(&resource_planet, explorer_id);
        charge_planet_with_sunrays(&main_planet, 1);
        charge_planet_with_sunrays(&resource_planet, 1);

        // Get Water from main planet
        // Get Hydrogen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let hydrogen =
            get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Hydrogen);

        // Get Oxygen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let oxygen = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Oxygen);

        // Create Water
        let water = if let (Some(h), Some(o)) = (extract_hydrogen(hydrogen), extract_oxygen(oxygen))
        {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Water(h, o),
            );

            match result {
                Ok(water) => water,
                Err(e) => panic!("Water creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Hydrogen and Oxygen resources");
        };

        // Get Life from main planet
        // Get Hydrogen and Oxygen again from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let hydrogen_2 =
            get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Hydrogen);

        charge_planet_with_sunrays(&resource_planet, 1);
        let oxygen_2 = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Oxygen);

        // Create Water for Life combination
        let water_for_life =
            if let (Some(h), Some(o)) = (extract_hydrogen(hydrogen_2), extract_oxygen(oxygen_2)) {
                charge_planet_with_sunrays(&main_planet, 1);
                let result = combine_resources(
                    &main_planet,
                    explorer_id,
                    common_game::components::resource::ComplexResourceRequest::Water(h, o),
                );

                match result {
                    Ok(water) => water,
                    Err(e) => panic!("Water creation failed: {:?}", e),
                }
            } else {
                panic!("Failed to extract Hydrogen and Oxygen resources");
            };

        // Get Carbon from main planet
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Create Life
        let life =
            if let (Some(c), Some(w)) = (extract_carbon(carbon), extract_water(water_for_life)) {
                charge_planet_with_sunrays(&main_planet, 1);
                let result = combine_resources(
                    &main_planet,
                    explorer_id,
                    common_game::components::resource::ComplexResourceRequest::Life(w, c),
                );

                match result {
                    Ok(life) => life,
                    Err(e) => panic!("Life creation failed: {:?}", e),
                }
            } else {
                panic!("Failed to extract Water or Carbon resources");
            };

        // Combine Water + Life => Dolphin
        if let (Some(w), Some(l)) = (extract_water(water), extract_life(life)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Dolphin(w, l),
            );

            match result {
                Ok(_dolphin) => println!("Dolphin created successfully!"),
                Err(e) => panic!("Dolphin creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Water or Life resources");
        }
    }

    /// Test Robot combination: Silicon + Life => Robot
    #[test]
    fn combine_resource_robot() {
        let (main_planet, resource_planet) = spawn_dual_planets();
        let explorer_id = 0;

        // Setup
        register_explorer_with_planet(&main_planet, explorer_id);
        register_explorer_with_planet(&resource_planet, explorer_id);
        charge_planet_with_sunrays(&main_planet, 1);
        charge_planet_with_sunrays(&resource_planet, 1);

        // Get Silicon from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let silicon = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Silicon);

        // Get Life from main planet
        // Get Hydrogen and Oxygen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let hydrogen =
            get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Hydrogen);

        charge_planet_with_sunrays(&resource_planet, 1);
        let oxygen = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Oxygen);

        // Create Water
        let water = if let (Some(h), Some(o)) = (extract_hydrogen(hydrogen), extract_oxygen(oxygen))
        {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Water(h, o),
            );

            match result {
                Ok(water) => water,
                Err(e) => panic!("Water creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Hydrogen and Oxygen resources");
        };

        // Get Carbon from main planet
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Create Life
        let life = if let (Some(c), Some(w)) = (extract_carbon(carbon), extract_water(water)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Life(w, c),
            );

            match result {
                Ok(life) => life,
                Err(e) => panic!("Life creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Water or Carbon resources");
        };

        // Combine Silicon + Life => Robot
        if let (Some(s), Some(l)) = (extract_silicon(silicon), extract_life(life)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Robot(s, l),
            );

            match result {
                Ok(_robot) => println!("Robot created successfully!"),
                Err(e) => panic!("Robot creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Silicon or Life resources");
        }
    }

    /// Test AIPartner combination: Robot + Diamond => AIPartner
    #[test]
    fn combine_resource_aipartner() {
        let (main_planet, resource_planet) = spawn_dual_planets();
        let explorer_id = 0;

        // Setup
        register_explorer_with_planet(&main_planet, explorer_id);
        register_explorer_with_planet(&resource_planet, explorer_id);
        charge_planet_with_sunrays(&main_planet, 1);
        charge_planet_with_sunrays(&resource_planet, 1);

        // Get Robot from main planet
        // Get Silicon from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let silicon = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Silicon);

        // Get Hydrogen and Oxygen from resource planet
        charge_planet_with_sunrays(&resource_planet, 1);
        let hydrogen =
            get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Hydrogen);

        charge_planet_with_sunrays(&resource_planet, 1);
        let oxygen = get_basic_resource(&resource_planet, explorer_id, BasicResourceType::Oxygen);

        // Create Water
        let water = if let (Some(h), Some(o)) = (extract_hydrogen(hydrogen), extract_oxygen(oxygen))
        {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Water(h, o),
            );

            match result {
                Ok(water) => water,
                Err(e) => panic!("Water creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Hydrogen and Oxygen resources");
        };

        // Get Carbon from main planet
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Create Life
        let life = if let (Some(c), Some(w)) = (extract_carbon(carbon), extract_water(water)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Life(w, c),
            );

            match result {
                Ok(life) => life,
                Err(e) => panic!("Life creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Water or Carbon resources");
        };

        // Create Robot
        let robot = if let (Some(s), Some(l)) = (extract_silicon(silicon), extract_life(life)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::Robot(s, l),
            );

            match result {
                Ok(robot) => robot,
                Err(e) => panic!("Robot creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Silicon or Life resources");
        };

        // Get Diamond from main planet
        // Get first Carbon
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon_1 = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Get second Carbon
        charge_planet_with_sunrays(&main_planet, 1);
        let carbon_2 = get_basic_resource(&main_planet, explorer_id, BasicResourceType::Carbon);

        // Create Diamond
        let diamond =
            if let (Some(c1), Some(c2)) = (extract_carbon(carbon_1), extract_carbon(carbon_2)) {
                charge_planet_with_sunrays(&main_planet, 1);
                let result = combine_resources(
                    &main_planet,
                    explorer_id,
                    common_game::components::resource::ComplexResourceRequest::Diamond(c1, c2),
                );

                match result {
                    Ok(diamond) => diamond,
                    Err(e) => panic!("Diamond creation failed: {:?}", e),
                }
            } else {
                panic!("Failed to extract Carbon resources");
            };

        // Combine Robot + Diamond => AIPartner
        if let (Some(r), Some(d)) = (extract_robot(robot), extract_diamond(diamond)) {
            charge_planet_with_sunrays(&main_planet, 1);
            let result = combine_resources(
                &main_planet,
                explorer_id,
                common_game::components::resource::ComplexResourceRequest::AIPartner(r, d),
            );

            match result {
                Ok(_aipartner) => println!("AIPartner created successfully!"),
                Err(e) => panic!("AIPartner creation failed: {:?}", e),
            }
        } else {
            panic!("Failed to extract Robot or Diamond resources");
        }
    }

    #[test]
    fn multiple_start_ai_messages_are_ignored() {
        let planet = spawn_planet();
        let snd = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::StartPlanetAI);
        let res = planet.rcv_planet_to_orc.try_recv();
        match res {
            Ok(_) => {
                assert!(false);
            }
            Err(_) => {
                print!("Ignored message");
                assert!(true);
            }
        }
    }

    #[test]
    fn multiple_stop_ai_messages_are_ignored() {
        let planet = spawn_planet();
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::StopPlanetAI);
        let _ = planet
            .snd_orc_to_planet
            .send(OrchestratorToPlanet::StopPlanetAI);
        let res_stop_ack = planet.rcv_planet_to_orc.recv();
        match res_stop_ack {
            Ok(PlanetToOrchestrator::StopPlanetAIResult { .. }) => {
                assert!(true);
            }
            _ => {
                panic!("Other message than the expected");
            }
        }
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(PlanetToOrchestrator::Stopped { .. }) => {
                assert!(true);
            }
            Err(_) => {
                panic!("Failed to receive Planet");
            }
            _ => {
                panic!("Failed to receive Planet, other message");
            }
        }
    }
}
