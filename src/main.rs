use air_fryer::{PlanetAI, create_planet};
use crossbeam_channel::unbounded;

fn main() {
    // Create communication channels
    let (_orc_sender, orc_receiver) = unbounded();
    let (planet_sender, _planet_receiver) = unbounded();
    let (_explorer_sender, explorer_receiver) = unbounded();

    let _planet = create_planet(
        42,
        PlanetAI::new(),
        (orc_receiver, planet_sender),
        explorer_receiver,
    );
}
