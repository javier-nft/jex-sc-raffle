#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Started,
    Ended,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct RaffleStatus<M: ManagedTypeApi> {
    state: State,
    prize: Option<EsdtTokenPayment<M>>,
    ticket_price: Option<EsdtTokenPayment<M>>,
    ticket_sale_end_timestamp: u64,
    burn_rate_percent: u32,
    fees_address: Option<ManagedAddress<M>>,
    nb_entries: usize,
}

const MAX_BURN_RATE_PERCENT: u32 = 100;

#[multiversx_sc::contract]
pub trait JexScRaffleContract {
    #[init]
    fn init(&self) {
        self.state().set(State::Ended);
    }

    // Owner endpoints

    #[endpoint(startRaffle)]
    #[only_owner]
    #[payable("*")]
    fn start_raffle(
        &self,
        raffle_name: ManagedBuffer,
        ticket_sale_duration: u64,
        ticket_token_identifier: TokenIdentifier,
        ticket_token_nonce: u64,
        ticket_price: BigUint,
        burn_rate_percent: u32,
        fees_address: ManagedAddress,
    ) {
        require!(
            self.state().get() == State::Ended,
            "Current raffle not ended"
        );

        require!(self.winners(&raffle_name).is_empty(), "Name already used");
        self.raffle_name().set(&raffle_name);

        let payment = self.call_value().single_esdt();

        self.fees_address().set(&fees_address);

        require!(
            burn_rate_percent <= MAX_BURN_RATE_PERCENT,
            "Invalid burn rate percent"
        );

        self.burn_rate_percent().set(burn_rate_percent);
        if burn_rate_percent > 0u32 {
            let roles = self
                .blockchain()
                .get_esdt_local_roles(&ticket_token_identifier.clone());
            require!(roles.has_role(&EsdtLocalRole::Burn), "Missing burn role");
        }

        self.prize().set(EsdtTokenPayment::new(
            payment.token_identifier,
            payment.token_nonce,
            payment.amount,
        ));

        self.ticket_price().set(EsdtTokenPayment::new(
            ticket_token_identifier,
            ticket_token_nonce,
            ticket_price,
        ));

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

        require!(
            entries_mapper.len() >= nb_winners.into(),
            "Too many winners"
        );

        let ticket_price = self.ticket_price().get();
        let mut tickets_token_balance = self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(ticket_price.token_identifier.clone()),
            ticket_price.token_nonce,
        );

        self.do_burn(
            &ticket_price.token_identifier,
            ticket_price.token_nonce,
            &mut tickets_token_balance,
        );

        if tickets_token_balance > 0 {
            self.send().direct_esdt(
                &self.fees_address().get(),
                &ticket_price.token_identifier,
                ticket_price.token_nonce,
                &tickets_token_balance,
            );
        }

        let prize = self.prize().get();
        let rewards_per_ticket = prize.amount / BigUint::from(nb_winners);

        let raffle_name = self.raffle_name().get();
        let mut rnd = RandomnessSource::new();
        for _ in 0..nb_winners {
            let num = 1 + rnd.next_usize_in_range(0, entries_mapper.len());

            let winner = entries_mapper.get(num);

            if rewards_per_ticket > 0 {
                self.send().direct_esdt(
                    &winner,
                    &prize.token_identifier,
                    prize.token_nonce,
                    &rewards_per_ticket,
                );
            }

            self.winners(&raffle_name).push(&winner);
        }

        self.send_leftovers_to_owner();
    }

    #[endpoint(endRaffle)]
    #[only_owner]
    fn end_raffle(&self) {
        require!(self.state().get() == State::Started, "Raffle not started");

        let leftover_balance = self.get_leftover_balance();

        require!(leftover_balance == 0u32, "Rewards not distributed");

        self.burn_rate_percent().clear();
        self.entries().clear();
        self.fees_address().clear();
        self.prize().clear();
        self.raffle_name().clear();
        self.ticket_price().clear();
        self.ticket_sale_end_timestamp().clear();

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

        let payment = self.call_value().single_esdt();

        let ticket_price = self.ticket_price().get();
        require!(
            payment.token_identifier == ticket_price.token_identifier
                && payment.token_nonce == ticket_price.token_nonce,
            "Invalid token"
        );

        require!(payment.amount == ticket_price.amount * nb, "Invalid amount");

        let caller = self.blockchain().get_caller();
        let mut entries_mapper = self.entries();
        for _ in 0..nb {
            entries_mapper.push(&caller);
        }
    }

    // Functions

    fn do_burn(
        &self,
        ticket_token_identifier: &TokenIdentifier,
        ticket_token_nonce: u64,
        tickets_token_balance: &mut BigUint,
    ) {
        let burn_rate_mapper = self.burn_rate_percent();
        let burn_rate = burn_rate_mapper.get();

        if burn_rate > 0u32 {
            let burn_amount = tickets_token_balance.clone() * burn_rate / 100u32;

            *tickets_token_balance -= &burn_amount;

            self.send()
                .esdt_local_burn(ticket_token_identifier, ticket_token_nonce, &burn_amount);
        }
    }

    fn get_leftover_balance(&self) -> BigUint {
        let prize = self.prize().get();

        let leftover_balance = self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(prize.token_identifier.clone()),
            prize.token_nonce,
        );

        leftover_balance
    }

    fn send_leftovers_to_owner(&self) {
        let leftover_balance = self.get_leftover_balance();

        let prize = self.prize().get();
        if leftover_balance > 0u32 {
            self.send().direct_esdt(
                &self.blockchain().get_owner_address(),
                &prize.token_identifier,
                prize.token_nonce,
                &leftover_balance,
            );
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
        let prize = if self.prize().is_empty() {
            Option::None
        } else {
            Option::Some(self.prize().get())
        };
        let ticket_price = if self.ticket_price().is_empty() {
            Option::None
        } else {
            Option::Some(self.ticket_price().get())
        };

        return RaffleStatus {
            burn_rate_percent: self.burn_rate_percent().get(),
            fees_address,
            nb_entries: self.entries().len(),
            prize,
            state: self.state().get(),
            ticket_price,
            ticket_sale_end_timestamp: self.ticket_sale_end_timestamp().get(),
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

    #[storage_mapper("entries")]
    fn entries(&self) -> VecMapper<ManagedAddress>;

    #[view(getFeesAddress)]
    #[storage_mapper("fees_address")]
    fn fees_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getRaffleName)]
    #[storage_mapper("raffle_name")]
    fn raffle_name(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getPrize)]
    #[storage_mapper("prize")]
    fn prize(&self) -> SingleValueMapper<EsdtTokenPayment>;

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[view(getTicketPrice)]
    #[storage_mapper("ticket_price")]
    fn ticket_price(&self) -> SingleValueMapper<EsdtTokenPayment>;

    #[view(getTicketSaleEndTimestamp)]
    #[storage_mapper("ticket_sale_end_timestamp")]
    fn ticket_sale_end_timestamp(&self) -> SingleValueMapper<u64>;

    #[view(getWinners)]
    #[storage_mapper("winners")]
    fn winners(&self, raffle_name: &ManagedBuffer) -> VecMapper<Self::Api, ManagedAddress>;
}
