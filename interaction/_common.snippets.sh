##
# Info
##

echo "Proxy: ${PROXY}"
echo "SC address: ${SC_ADDRESS:-Not deployed}"

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
        --gas-limit=10000000 \
        --function="pickWinners" --arguments ${NB_WINNING_TICKETS} \
        --proxy=${PROXY} --chain=${CHAIN} --send || return
}

startRaffle() {
    read -p "Raffle name: " RAFFLE_NAME
    read -p "Prize token identifier: " PRIZE_TOKEN_IDENTIFIER
    read -p "Prize amount (in weis - no float): " PRIZE_AMOUNT
    read -p "Sale duration (sec): " SALE_DURATION
    read -p "Tickets token identifier: " TICKETS_TOKEN_IDENTIFIER
    read -p "Tickets token nonce (decimal): " TICKETS_TOKEN_NONCE
    read -p "Price per ticket (in weis - no float): " PRICE_PER_TICKET
    read -p "Burn rate (0-100): " BURN_RATE
    read -p "Fees receiver: " FEES_RECEIVER

    RAFFLE_NAME="0x$(echo -n "${RAFFLE_NAME}" | xxd -ps)"
    PRIZE_TOKEN_IDENTIFIER="0x$(echo -n "${PRIZE_TOKEN_IDENTIFIER}" | xxd -ps)"
    TICKETS_TOKEN_IDENTIFIER="0x$(echo -n "${TICKETS_TOKEN_IDENTIFIER}" | xxd -ps)"
    FEES_RECEIVER="0x$(mxpy wallet bech32 --decode ${FEES_RECEIVER})"
    METHOD="0x$(echo -n "startRaffle" | xxd -ps)"

    mxpy contract call ${SC_ADDRESS} --recall-nonce --keyfile=${KEYFILE} \
        --gas-limit=10000000 \
        --function="ESDTTransfer" \
        --arguments ${PRIZE_TOKEN_IDENTIFIER} ${PRIZE_AMOUNT} \
                    ${METHOD} \
                    "${RAFFLE_NAME}" \
                    ${SALE_DURATION} ${TICKETS_TOKEN_IDENTIFIER} ${TICKETS_TOKEN_NONCE} \
                    ${PRICE_PER_TICKET} ${BURN_RATE} ${FEES_RECEIVER} \
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
    curl -s "https://gateway.multiversx.com/address/${SC_ADDRESS}/key/656e74726965732e6c656e"
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
