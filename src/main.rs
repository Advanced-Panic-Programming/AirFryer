mod air_fryer;
mod mock_planet;

fn main() {
    //This main represent the initial setup for our type of planet, and it's crafting and generating rules
    /*
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

    //
    // let planet = Planet::new(0, PlanetType::C, Box::new(ia), gene, rcv_expl_to_planet);    if planet.is_ok() {
    //    planet.unwrap().run();
    //}
    //Planet::new(0, PlanetType::C, (), vec![], vec![], ((), ()), ((), ()));
     */
}

#[cfg(test)]
mod tests;
