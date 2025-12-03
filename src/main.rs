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
    //Planet::new(0, PlanetType::C, (), vec![], vec![], ((), ()), ((), ()));

}
#[cfg(test)]
mod tests {
    use std::os::macos::raw::pthread_t;
    use std::sync::mpsc;
    use std::thread;
    use common_game::components::planet::{Planet, PlanetType};
    use common_game::components::resource::{BasicResource, BasicResourceType, Carbon, ComplexResourceType};
    use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator, StartPlanetAiMsg};
    use crate::air_frier;

    #[test]
    fn ask_for_carbon_from_explorer() {
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
        sdr_expl_to_planet.send(ExplorerToPlanet::GenerateResourceRequest {explorer_id: 0, resource: BasicResourceType::Carbon});
        let t1 = thread::spawn(move ||{
            planet.unwrap().run();
        });
        let res = rcv_planet_to_expl.recv();
        match res{
            Ok(msg) => {
                match msg {
                    PlanetToExplorer::SupportedResourceResponse { .. } => {}
                    PlanetToExplorer::SupportedCombinationResponse { .. } => {}
                    PlanetToExplorer::GenerateResourceResponse { resource } => {
                        if resource.is_some(){
                            println!("Resource generated successfully");
                        }
                        else {
                            println!("Resource not generated");
                        }
                    }
                    PlanetToExplorer::CombineResourceResponse { .. } => {}
                    PlanetToExplorer::AvailableEnergyCellResponse { .. } => {}
                    PlanetToExplorer::InternalStateResponse { .. } => {}
                }
            }
            Err(_) => {
                println!("Result error");
            }
        }
    }
}


