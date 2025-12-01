use common_game::components::planet;
use common_game::components::planet::PlanetType;

struct Planet{
    state: planet::PlanetState,
    a: planet::Planet<PlanetType::C>
}