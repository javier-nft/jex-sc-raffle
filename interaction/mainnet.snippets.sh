PROJECT=..
KEYFILE="../../wallets/deployer.json"
PROXY=https://gateway.multiversx.com
SC_ADDRESS=$(mxpy data load --key=address-mainnet)
CHAIN=1
SCRIPT_DIR=$(dirname $0)

source "${SCRIPT_DIR}/_common.snippets.sh"

# Reproducible build using:
# mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v5.0.0"
deploy() {
    echo 'You are about to deploy SC on mainnet (Ctrl-C to abort)'
    read answer

    mxpy contract deploy --bytecode ${PROJECT}/output-docker/jex-sc-raffle/jex-sc-raffle.wasm \
         --keyfile=${KEYFILE} --gas-limit=150000000 --outfile="deploy-mainnet.interaction.json" \
         --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send || return

    SC_ADDRESS=$(mxpy data parse --file="deploy-mainnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-mainnet --value=${SC_ADDRESS}

    echo ""
    echo "Smart contract address: ${SC_ADDRESS}"
}

upgrade() {
    echo 'You are about to upgrade current SC on mainnet (Ctrl-C to abort)'
    read answer

    mxpy contract upgrade --bytecode ${PROJECT}/output-docker/jex-sc-raffle/jex-sc-raffle.wasm \
        --keyfile=${KEYFILE} --gas-limit=150000000 --outfile="deploy-mainnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send ${SC_ADDRESS} || return

    echo ""
    echo "Smart contract upgraded: ${SC_ADDRESS}"
}

verify() {
    mxpy contract verify "${SC_ADDRESS}" \
        --packaged-src=${PROJECT}/output-docker/jex-sc-raffle/jex-sc-raffle-0.0.0.source.json \
        --verifier-url="https://play-api.multiversx.com" \
        --docker-image="multiversx/sdk-rust-contract-builder:v4.1.3" \
        --keyfile=${KEYFILE}
}

CMD=$1
shift

$CMD $*
