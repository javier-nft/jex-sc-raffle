use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "file:output/jex-sc-raffle.wasm",
        jex_sc_raffle::ContractBuilder,
    );
    blockchain
}

#[test]
fn run_test() {
    world().run("scenarios/buy_tickets.scen.json");
}
