use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn run_test() {
    world().run("scenarios/buy_tickets.scen.json");
}
