use bevy::prelude::*;
use crate::types::{Opening, RoundOutcome};

#[derive(Event, Clone, Copy, Debug)]
pub struct GoEvent {
    pub opening: Opening,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct EarlyInputEvent;

#[derive(Event, Clone, Copy, Debug)]
pub struct ResolveEvent {
    pub outcome: RoundOutcome,
    pub clash: bool,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct RoundTransitionEvent;

