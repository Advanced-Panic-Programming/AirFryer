mod air_frier;

use std::sync::mpsc;
use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
fn main() {
    //New AI
    let ia = air_frier::PlanetAI::new();

    let mut gene:Vec<BasicResourceType> = Vec::new();
    gene.push(BasicResourceType::Carbon);

    let mut compl:Vec<ComplexResourceType> = Vec::new();
    compl.push(ComplexResourceType::Water);
    compl.push(ComplexResourceType::Life);
    compl.push(ComplexResourceType::Dolphin);
    compl.push(ComplexResourceType::Robot);
    compl.push(ComplexResourceType::AIPartner);

    let (sdr_expl_to_planet, rcv_expl_to_planet) = mpsc::channel::<ExplorerToPlanet>();
    let (sdr_planet_to_expl, rcv_planet_to_expl) = mpsc::channel::<PlanetToExplorer>();
    let (sdr_planet_to_orc, rcv_planet_to_orc) = mpsc::channel::<PlanetToOrchestrator>();
    let (sdr_orc_to_planet, rcv_orc_to_planet) = mpsc::channel::<OrchestratorToPlanet>();

    let planet = Planet::new(0, PlanetType::C, ia, gene, compl, (rcv_orc_to_planet, sdr_planet_to_orc), (rcv_expl_to_planet, sdr_planet_to_expl));
    if planet.is_ok(){
        planet.unwrap().run();
    }

}
#[cfg(test)]
mod tests {
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;
    use common_game::components::asteroid::Asteroid;
    use common_game::components::sunray::Sunray;
    use common_game::protocols::messages::OrchestratorToPlanet::Asteroid as OtherAsteroid;
    use common_game::protocols::messages::StartPlanetAiMsg;
    use log::log;
    use super::*;
    struct TestContext{
        snd_orc_to_planet: mpsc::Sender<OrchestratorToPlanet>,
        snd_exp_to_planet: mpsc::Sender<ExplorerToPlanet>,
        rcv_planet_to_exp: mpsc::Receiver<PlanetToExplorer>,
        rcv_planet_to_orc: mpsc::Receiver<PlanetToOrchestrator>,
    }
    fn spawn_planet() -> TestContext{
        let ia = air_frier::PlanetAI::new();

        let mut gene:Vec<BasicResourceType> = Vec::new();
        gene.push(BasicResourceType::Carbon);

        let mut compl:Vec<ComplexResourceType> = Vec::new();
        compl.push(ComplexResourceType::Water);
        compl.push(ComplexResourceType::Life);
        compl.push(ComplexResourceType::Dolphin);
        compl.push(ComplexResourceType::Robot);
        compl.push(ComplexResourceType::AIPartner);

        let (sdr_expl_to_planet, rcv_expl_to_planet) = mpsc::channel::<ExplorerToPlanet>();
        let (sdr_planet_to_expl, rcv_planet_to_expl) = mpsc::channel::<PlanetToExplorer>();
        let (sdr_planet_to_orc, rcv_planet_to_orc) = mpsc::channel::<PlanetToOrchestrator>();
        let (sdr_orc_to_planet, rcv_orc_to_planet) = mpsc::channel::<OrchestratorToPlanet>();

        let planet = Planet::new(0, PlanetType::C, ia, gene, compl, (rcv_orc_to_planet, sdr_planet_to_orc), (rcv_expl_to_planet, sdr_planet_to_expl));
        sdr_orc_to_planet.send(OrchestratorToPlanet::StartPlanetAI(StartPlanetAiMsg));
        let t1 = thread::spawn(move ||{
            planet.unwrap().run();
        });
        sleep(Duration::from_millis(10));
        TestContext{
            snd_orc_to_planet: sdr_orc_to_planet,
            snd_exp_to_planet: sdr_expl_to_planet,
            rcv_planet_to_orc: rcv_planet_to_orc,
            rcv_planet_to_exp: rcv_planet_to_expl,
        }
    }

    #[test]
    ///Sends an asteroid to the planet and checks that the planet responde with a none
    fn test_asteroid_with_no_rocket() {
        let mut planet = spawn_planet();
        planet.snd_orc_to_planet.send(OrchestratorToPlanet::Asteroid(Asteroid::new()));
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(msg) => {
                match msg{
                    PlanetToOrchestrator::AsteroidAck { planet_id: _, rocket: r } => {
                        assert!(r.is_none());
                    }
                    _=>{}
                }
            }
            Err(_) => {}
        }
    }
    #[test]
    ///Sends a sunray to the planet, that makes a rocket with it, later it sends an asteroid and we check if che planet respond with a rocket
    fn test_asteroid_with_rocket() {
        let planet = spawn_planet();
        planet.snd_orc_to_planet.send(OrchestratorToPlanet::Sunray(Sunray::new()));
        planet.snd_orc_to_planet.send(OrchestratorToPlanet::Asteroid(Asteroid::new()));
        let res = planet.rcv_planet_to_orc.recv();
        let res = planet.rcv_planet_to_orc.recv();
        match res {
            Ok(msg) => {
                match msg {
                    PlanetToOrchestrator::AsteroidAck {planet_id: _, rocket: r } => {
                        assert!(r.is_some());
                    }
                    _=>{}
                }

            }
            Err(_) => {
                assert!(false);
            }
        }

    }
    #[test]
    fn ask_for_carbon_from_explorer() {
        let mut planet = spawn_planet();
        planet.snd_exp_to_planet.send(ExplorerToPlanet::GenerateResourceRequest { explorer_id: 0, resource: BasicResourceType::Carbon });
        let res = planet.rcv_planet_to_exp.recv();
        match res{
            Ok(msg)=>{
                match msg {
                    PlanetToExplorer::GenerateResourceResponse {resource} => {
                        if resource.is_some(){
                            println!("Resource generated successfully!");
                        }
                        else {
                            println!("Resource not generated!");
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {
                println!("Result error");
            }
        }
    }

}


