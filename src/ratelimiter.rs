use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

use axum::http::HeaderMap;
use dashmap::DashMap;

#[derive(Debug)]
pub struct TicketState {
    tickets: u64,
    last_refill_at: Instant,
}

#[derive(Debug)]
pub struct Ratelimiter {
    pub max_tickets: u64,
    pub refill_after: Duration,
    state: DashMap<IpAddr, TicketState>,
}

pub struct RatelimiterResponse {
    pub allowed: bool,
    limit: u64,
    used: u64,
    remaining: u64,
    reset: u64,
}

impl Ratelimiter {
    pub fn new(max_tickets: u64, refill_after: Duration) -> Self {
        Self {
            max_tickets,
            refill_after,
            state: DashMap::new(),
        }
    }

    pub fn adquire(&self, ip: IpAddr) -> RatelimiterResponse {
        let now = Instant::now();
        let mut entry = self.state.entry(ip).or_insert(TicketState {
            tickets: self.max_tickets,
            last_refill_at: now,
        });

        let since_last_refill = now.duration_since(entry.last_refill_at);
        let refill_count =
            (since_last_refill.as_secs_f64() / self.refill_after.as_secs_f64()).floor() as u64;

        let refill_count = refill_count.min(self.max_tickets); // DUMB BIRD. ITS BACKWARDS!!!! MIN = MAX ; MAX = MIN
        if refill_count > 0 {
            entry.tickets = entry
                .tickets
                .saturating_add(refill_count)
                // god DADMMIT FUCKING SHIT PROGRAMMER I AM GODDAMIT AGH. if an user has more than the max tickets.. we still add more.
                // AND if we still add more, ANDDD this is STILL a u64 NOT to be confused with a I64. We underflow it lol in the sub step below.
                // Practically, "left number small and right number small equals negative numbers which equals to poo poo in u64"
                .min(self.max_tickets);
            entry.last_refill_at = now;
        }

        if entry.tickets == 0 {
            return RatelimiterResponse {
                allowed: false,
                limit: self.max_tickets,
                used: self.max_tickets,
                remaining: 0,
                reset: self.refill_after.as_secs(),
            };
        }

        entry.tickets = entry.tickets.saturating_sub(1);
        let used_tickets = self.max_tickets - entry.tickets;

        RatelimiterResponse {
            allowed: true,
            limit: self.max_tickets,
            used: used_tickets,
            remaining: entry.tickets,
            reset: self.refill_after.as_secs(),
        }
    }
}

impl RatelimiterResponse {
    pub fn header_map(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert("x-ratelimit-limit", self.limit.into());
        headers.insert("x-ratelimit-used", self.used.into());
        headers.insert("x-ratelimit-remaining", self.remaining.into());
        headers.insert("x-ratelimit-reset", self.reset.into());
        headers
    }
}
