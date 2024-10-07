#![no_std]

use multiversx_sc::hex_literal::hex;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Ended,
    Started,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct RaffleStatus<M: ManagedTypeApi> {
    name: ManagedBuffer<M>,
    state: State,
    burn_rate_percent: u32,
    fees_rate_percent: u32,
    fees_address: Option<ManagedAddress<M>>,
    prize_pool_rate_percent: u32,
    ticket_sale_end_timestamp: u64,
    nb_entries: usize,
    ticket_prices: ManagedVec<M, EsdtTokenPayment<M>>,
}

#[multiversx_sc::contract]
pub trait JexScRaffleContract {
    #[init]
    fn init(&self) {
        self.state().set_if_empty(State::Ended);
    }

    #[upgrade]
    fn upgrade(&self) {
        self.dead_address().set_if_empty(ManagedAddress::from(hex!(
            "6e7ad6e7ad6e7ad6e7ad6e7ad6e7ad6e7ad6e7ad6e7ad6e7ad6e7ad6e7ad6e7a"
        )));
    }

    // Owner endpoints

    #[endpoint(prepareRaffle)]
    #[only_owner]
    fn prepare_raffle(
        &self,
        raffle_name: ManagedBuffer,
        burn_rate_percent: u32,
        fees_rate_percent: u32,
        fees_address: ManagedAddress,
        prize_pool_rate_percent: u32,
    ) {
        require!(
            self.state().get() == State::Ended,
            "Current raffle not ended"
        );

        require!(self.winners(&raffle_name).is_empty(), "Name already used");
        self.raffle_name().set(&raffle_name);

        require!(
            burn_rate_percent + fees_rate_percent + prize_pool_rate_percent == 100u32,
            "Sum of rates should be 100"
        );

        self.burn_rate_percent().set(burn_rate_percent);
        self.fees_rate_percent().set(fees_rate_percent);
        self.fees_address().set(&fees_address);
        self.prize_pool_rate_percent().set(prize_pool_rate_percent);
    }

    #[endpoint(configureTicketPrice)]
    #[only_owner]
    fn configure_ticket_price(
        &self,
        ticket_token_identifier: TokenIdentifier,
        ticket_price: BigUint,
    ) {
        self.ticket_tokens().insert(ticket_token_identifier.clone());
        self.ticket_price(&ticket_token_identifier)
            .set(&ticket_price);
    }

    #[endpoint(startRaffle)]
    #[only_owner]
    fn start_raffle(&self, ticket_sale_duration: u64) {
        require!(
            self.state().get() == State::Ended,
            "Current raffle not ended"
        );

        self.ticket_sale_end_timestamp()
            .set(self.blockchain().get_block_timestamp() + ticket_sale_duration);

        self.state().set(State::Started);
    }

    #[endpoint(pickWinners)]
    #[only_owner]
    fn pick_winners(&self, nb_winners: u16) {
        require!(
            self.blockchain().get_block_timestamp() > self.ticket_sale_end_timestamp().get(),
            "Still in tickets sale period"
        );

        let entries_mapper = self.entries();
        if !entries_mapper.is_empty() {
            self.send_rewards_to_winners(nb_winners);
        }

        self.send_leftovers_to_owner();
    }

    #[endpoint(clearEntries)]
    #[only_owner]
    fn clear_entries(&self, count: u32) {
        let raffle_name = self.raffle_name().get();

        require!(
            self.winners(&raffle_name).len() > 0,
            "Rewards not distributed"
        );

        for _ in 0..count {
            self.entries().swap_remove(1);
        }
    }

    #[endpoint(endRaffle)]
    #[only_owner]
    fn end_raffle(&self) {
        require!(self.state().get() == State::Started, "Raffle not started");

        let raffle_name = self.raffle_name().get();
        require!(
            self.winners(&raffle_name).len() > 0,
            "Rewards not distributed"
        );

        self.burn_rate_percent().clear();
        self.entries().clear();
        self.fees_rate_percent().clear();
        self.fees_address().clear();
        self.prize_pool_rate_percent().clear();
        self.raffle_name().clear();
        self.ticket_sale_end_timestamp().clear();

        for token in self.ticket_tokens().iter() {
            self.ticket_price(&token).clear();
        }
        self.ticket_tokens().clear();

        self.state().set(State::Ended);
    }

    // Public endpoints

    #[endpoint(buyTickets)]
    #[payable("*")]
    fn buy_tickets(&self, nb: u32) {
        require!(self.state().get() == State::Started, "Raffle not started");

        require!(
            self.blockchain().get_block_timestamp() <= self.ticket_sale_end_timestamp().get(),
            "Not in tickets sale period"
        );

        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();

        require!(
            !self.ticket_price(&payment_token).is_empty(),
            "Invalid payment token"
        );

        let ticket_price = self.ticket_price(&payment_token).get();
        require!(
            payment_amount == ticket_price * nb,
            "Invalid payment amount"
        );

        let caller = self.blockchain().get_caller();
        let mut entries_mapper = self.entries();
        for _ in 0..nb {
            entries_mapper.push(&caller);
        }

        let burn_percent = self.burn_rate_percent().get();
        if burn_percent > 0u32 {
            let burn_amount = (&payment_amount * burn_percent) / 100u32;

            self.burn(&payment_token, &burn_amount);
        }

        let fees_percent = self.fees_rate_percent().get();
        if fees_percent > 0u32 {
            let fee_amount = (&payment_amount * fees_percent) / 100u32;
            self.send().direct_esdt(
                &self.fees_address().get(),
                &payment_token,
                0u64,
                &fee_amount,
            );
        }
    }

    // Functions

    fn burn(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        let roles = self.blockchain().get_esdt_local_roles(&token_id);

        if roles.has_role(&EsdtLocalRole::Burn) {
            self.send().esdt_local_burn(&token_id, 0u64, &amount);
        } else {
            self.send()
                .direct_esdt(&self.dead_address().get(), token_id, 0u64, &amount);
        }
    }

    fn send_leftovers_to_owner(&self) {
        for token_identifier in self.ticket_tokens().iter() {
            let leftover_balance = self.blockchain().get_sc_balance(
                &EgldOrEsdtTokenIdentifier::esdt(token_identifier.clone()),
                0u64,
            );

            if leftover_balance > 0u32 {
                self.send().direct_esdt(
                    &self.blockchain().get_owner_address(),
                    &token_identifier,
                    0u64,
                    &leftover_balance,
                );
            }
        }
    }

    fn send_rewards_to_winners(&self, nb_winners: u16) {
        let mut entries_mapper = self.entries();

        require!(
            entries_mapper.len() >= nb_winners.into(),
            "Too many winners"
        );

        let raffle_name = self.raffle_name().get();
        let mut rnd = RandomnessSource::new();
        for _ in 0..nb_winners {
            let num = 1 + rnd.next_usize_in_range(0, entries_mapper.len());

            let winner = entries_mapper.get(num);
            self.winners(&raffle_name).push(&winner);

            entries_mapper.swap_remove(num);
        }

        let winners: ManagedVec<Self::Api, ManagedAddress<Self::Api>> =
            self.winners(&raffle_name).iter().collect();

        for token_identifier in self.ticket_tokens().iter() {
            let balance = self.blockchain().get_sc_balance(
                &EgldOrEsdtTokenIdentifier::esdt(token_identifier.clone()),
                0u64,
            );

            let rewards_per_ticket = balance / BigUint::from(nb_winners);
            if rewards_per_ticket > 0 {
                for winner in winners.iter() {
                    self.send()
                        .direct_esdt(&winner, &token_identifier, 0u64, &rewards_per_ticket);
                }
            }
        }
    }

    // Storage & Views

    #[view(getRaffleStatus)]
    fn get_raffle_status(&self) -> RaffleStatus<Self::Api> {
        let fees_address = if self.fees_address().is_empty() {
            Option::None
        } else {
            Option::Some(self.fees_address().get())
        };

        // let ticket_prices = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        let ticket_prices: ManagedVec<Self::Api, EsdtTokenPayment> = self
            .ticket_tokens()
            .iter()
            .map(|token| {
                EsdtTokenPayment::new(token.clone(), 0u64, self.ticket_price(&token).get())
            })
            .collect();

        return RaffleStatus {
            name: self.raffle_name().get(),
            burn_rate_percent: self.burn_rate_percent().get(),
            fees_rate_percent: self.fees_rate_percent().get(),
            fees_address,
            nb_entries: self.entries().len(),
            prize_pool_rate_percent: self.prize_pool_rate_percent().get(),
            state: self.state().get(),
            ticket_sale_end_timestamp: self.ticket_sale_end_timestamp().get(),
            ticket_prices,
        };
    }

    #[view(getEntries)]
    fn get_entries(
        &self,
        from: usize,
        size: usize,
    ) -> MultiValueEncoded<Self::Api, ManagedAddress> {
        let entries: ManagedVec<ManagedAddress> =
            self.entries().iter().skip(from).take(size).collect();
        entries.into()
    }

    #[view(getBurnRatePercent)]
    #[storage_mapper("burn_rate_percent")]
    fn burn_rate_percent(&self) -> SingleValueMapper<u32>;

    #[view(getDeadAddress)]
    #[storage_mapper("dead_address")]
    fn dead_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("entries")]
    fn entries(&self) -> VecMapper<ManagedAddress>;

    #[view(getFeesRatePercent)]
    #[storage_mapper("fees_rate_percent")]
    fn fees_rate_percent(&self) -> SingleValueMapper<u32>;

    #[view(getFeesAddress)]
    #[storage_mapper("fees_address")]
    fn fees_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getPrizePoolPercent)]
    #[storage_mapper("prize_pool_rate_percent")]
    fn prize_pool_rate_percent(&self) -> SingleValueMapper<u32>;

    #[view(getRaffleName)]
    #[storage_mapper("raffle_name")]
    fn raffle_name(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[view(getTicketPrice)]
    #[storage_mapper("ticket_price")]
    fn ticket_price(&self, token: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getTicketSaleEndTimestamp)]
    #[storage_mapper("ticket_sale_end_timestamp")]
    fn ticket_sale_end_timestamp(&self) -> SingleValueMapper<u64>;

    #[view(getTicketTokens)]
    #[storage_mapper("ticket_tokens")]
    fn ticket_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getWinners)]
    #[storage_mapper("winners")]
    fn winners(&self, raffle_name: &ManagedBuffer) -> VecMapper<Self::Api, ManagedAddress>;
}
