mod air_frier;

use std::sync::mpsc;
use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
fn main() {
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
