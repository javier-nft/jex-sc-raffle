##
# Info
##

echo "Proxy: ${PROXY}"
echo "SC address: ${SC_ADDRESS:-Not deployed}"

buildSc() {
    if [ -d "${SCRIPT_DIR}/../output-docker" ]
    then
        read -p "Remove existing folder? (press Enter)"

        rm -rf "${SCRIPT_DIR}/../output-docker"
    fi

    pushd ..
    mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v7.0.0"
    popd
}

##
# Tools
##

allowBurn() {
    read -p "Token identifier: " TOKEN_IDENTIFIER
    read -e -p "Path to keyfile: " KEYFILE

    TOKEN_IDENTIFIER="$(echo -n "${TOKEN_IDENTIFIER}" | xxd -ps)"
    SC_ADDRESS_HEX="$(mxpy wallet bech32 --decode ${SC_ADDRESS})"
    ROLE="$(echo -n "ESDTRoleLocalBurn" | xxd -ps)"
    SET_ROLE_DATA="setSpecialRole@${TOKEN_IDENTIFIER}@${SC_ADDRESS_HEX}@${ROLE}"
    SET_ROLE_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
    mxpy tx new --recall-nonce --keyfile=${KEYFILE} --gas-limit=51000000 \
        --receiver ${SET_ROLE_ADDRESS} \
        --data "${SET_ROLE_DATA}" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

##
# Owner endpoints
##

pickWinners() {
    read -p "Nb winning tickets: " NB_WINNING_TICKETS

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} \
        --gas-limit=50000000 \
        --function="pickWinners" --arguments ${NB_WINNING_TICKETS} \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

prepareRaffle() {
    read -p "Raffle name: " RAFFLE_NAME
    read -p "Burn rate (0-100): " BURN_RATE
    read -p "Fees rate (0-100): " FEES_RATE
    read -p "Fees receiver: " FEES_RECEIVER
    read -p "Prize pool rate (0-100): " PRIZE_POOL_RATE

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} \
        --gas-limit=10000000 \
        --function="prepareRaffle" \
        --arguments "str:${RAFFLE_NAME}" \
            "${BURN_RATE}" \
            "${FEES_RATE}" "${FEES_RECEIVER}" \
            "${PRIZE_POOL_RATE}" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

configureTicketPrice() {
    read -p "Token identifier: " TOKEN_IDENTIFIER
    read -p "Amount: " TOKEN_AMOUNT

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} \
        --gas-limit=10000000 \
        --function="configureTicketPrice" \
        --arguments "str:${TOKEN_IDENTIFIER}" "${TOKEN_AMOUNT}" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

startRaffle() {
    read -p "Sale duration (sec): " SALE_DURATION

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} \
        --gas-limit=10000000 \
        --function="startRaffle" \
        --arguments ${SALE_DURATION} \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

endRaffle() {
    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} \
        --gas-limit=100000000 \
        --function="endRaffle" \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

clearEntries() {
    read -p "Count (eg 250): " COUNT

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} \
        --gas-limit=100000000 \
        --function="clearEntries" \
        --arguments ${COUNT} \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}


##
# Views
##

getEntries() {
    FROM=${1:-0}
    SIZE=${2:-50}

    count=$((${FROM}+1))
    mxpy contract query ${SC_ADDRESS} \
        --function "getEntries" --arguments "${FROM}" "${SIZE}" \
        --proxy=${PROXY} | jq -r .[].hex \
            | while read hex; do
                echo -n "#${count}: "
                mxpy wallet bech32 --encode ${hex}
                count=$((count+1))
            done
}

getNbEntries() {
    curl -s "${PROXY}/address/${SC_ADDRESS}/key/656e74726965732e6c656e"
    echo ""
}

getRaffleStatus() {
    mxpy contract query ${SC_ADDRESS} --function "getRaffleStatus" --proxy=${PROXY}
}

getWinners() {
    read -p "Raffle name: " RAFFLE_NAME

    RAFFLE_NAME="0x$(echo -n "${RAFFLE_NAME}" | xxd -ps)"

    mxpy contract query ${SC_ADDRESS} \
        --function "getWinners" --arguments "${RAFFLE_NAME}" \
        --proxy=${PROXY}
}

##
# Public endpoints
##

buyTickets() {
    read -p "Nb Tickets: " NB_TICKETS
    read -p "Tickets token identifier: " TICKETS_TOKEN_IDENTIFIER
    read -p "Total amount (in weis - no float): " TOTAL_AMOUNT

    TICKETS_TOKEN_IDENTIFIER="0x$(echo -n "${TICKETS_TOKEN_IDENTIFIER}" | xxd -ps)"
    METHOD="0x$(echo -n "buyTickets" | xxd -ps)"

    mxpy contract call ${SC_ADDRESS} --recall-nonce --pem=$1 \
        --gas-limit=10000000 \
        --function="ESDTTransfer" \
        --arguments ${TICKETS_TOKEN_IDENTIFIER} ${TOTAL_AMOUNT} \
                    ${METHOD} \
                    ${NB_TICKETS} \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}
