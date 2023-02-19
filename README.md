# jex-sc-raffle

## Owner endpoints

startRaffle (...)

  * rewards (tokens)
  * ticket sale duration (seconds)
  * ticket price
  * burn rate percent
  * fees address


enableBuyWithNFT (...)

  * collection
  * nonce
  * nb tickets


selectWinners ()

  * select winners
  * send rewards to winners

endRaffle ()

  * clear entries
  * send back NFTs to owners


## Public endpoints

buyTickets (...)

  * nb tickets


buyTicketWithNFT (...)

  * NFT transfer
