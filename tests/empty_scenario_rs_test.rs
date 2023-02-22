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
fn raffle_rs() {
    multiversx_sc_scenario::run_rs("scenarios/pick_winners_too_many_winners.scen.json", world());
}
