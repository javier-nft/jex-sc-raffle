# jex-sc-raffle

## Owner endpoints

startRaffle (...)

  * rewards (tokens)
  * ticket sale duration (seconds)
  * ticket price
  * burn rate percent
  * fees address


enableBuyWithNFT (...) **not implemented**

  * collection
  * nonce
  * nb tickets


pickWinners ()

  * select winners randomly
  * send rewards to winners

endRaffle ()

  * clear entries
  * send back NFTs to owners


## Public endpoints

buyTickets (...)

  * nb tickets


buyTicketWithNFT (...) **not implemented**

  * NFT transfer
