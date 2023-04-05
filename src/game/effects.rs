use crate::prelude::*;

#[derive(Debug, Clone, Copy, Reflect)]
pub enum Effect {
	Haste(f32),
	MaterialVision,
}

#[derive(Clone, Copy, Debug)]
pub struct ActiveEffect {
	pub effect: Effect,
	pub time_left: f32, 
}

#[derive(Component, Clone, Debug, Default)]
pub struct ActiveEffects(pub Vec<ActiveEffect>);

#[derive(Debug, Clone, Component, Default)]
pub struct Ingridient {
	pub name: String,
	pub effects: SmallVec<[IngridientEffect;4]>,
}


#[derive(Clone, Copy, Debug)]
pub struct IngridientEffect {
	pub effect_type: Effect,
	pub duration: f32,
}