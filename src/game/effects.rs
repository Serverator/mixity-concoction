use crate::prelude::*;

#[derive(Debug, Clone, Copy, Reflect)]
pub enum Effect {
	Haste(f32),
	MaterialVision,
}

/// Effects and time left for them to wear off
#[derive(Clone, Copy, Debug)]
pub struct ActiveEffect {
	pub effect: Effect,
	pub time_left: f32, 
}

/// Added to the player to determine active effects
#[derive(Component, Clone, Debug, Default)]
pub struct ActiveEffects(pub Vec<ActiveEffect>);
