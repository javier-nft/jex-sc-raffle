PROJECT=..
KEYFILE="../../wallets/deployer-shard2.json"
PROXY=https://devnet-gateway.multiversx.com
SC_ADDRESS=$(mxpy data load --key=address-devnet)
CHAIN=D
SCRIPT_DIR=$(dirname $0)

source "${SCRIPT_DIR}/_common.snippets.sh"

deploy() {
    echo 'You are about to deploy SC on devnet (Ctrl-C to abort)'
    read answer

    mxpy contract deploy --project=${PROJECT} --metadata-payable \
        --keyfile=${KEYFILE} --gas-limit=150000000 --outfile="deploy-devnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send || return

    SC_ADDRESS=$(mxpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-devnet --value=${SC_ADDRESS}

    echo ""
    echo "Smart contract address: ${SC_ADDRESS}"
}

upgrade() {
    echo 'You are about to upgrade current SC on devnet (Ctrl-C to abort)'
    read answer

    mxpy contract upgrade --project=${PROJECT} --metadata-payable \
        --keyfile=${KEYFILE} --gas-limit=150000000 --outfile="deploy-devnet.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN} --recall-nonce --send ${SC_ADDRESS} || return

    echo ""
    echo "Smart contract upgraded: ${SC_ADDRESS}"
}

CMD=$1
shift

$CMD $*
