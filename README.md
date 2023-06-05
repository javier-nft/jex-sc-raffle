# jex-sc-raffle

## Owner endpoints

### prepareRaffle (...)

  * name
  * burn percent
  * fees percent + receiver
  * pool prize percent


### configureTicketPrice (...)
  * ticket token identifier
  * ticket price


### startRaffle (...)

  * ticket sale duration (seconds)


### pickWinners ()

  * select winners randomly
  * send rewards to winners


### endRaffle ()

  * clear entries
  * send back NFTs to owners


## Public endpoints

### buyTickets (...)

  * nb tickets


## Views

### getRaffleStatus

  * name
  * state: State,
  * burn_rate_percent
  * fees_rate_percent
  * fees_address (option)
  * prize_pool_rate_percent
  * ticket_sale_end_timestamp
  * nb_entries
  * ticket_prices


### getEntries

return the list of entries (1 entry = 1 address)
