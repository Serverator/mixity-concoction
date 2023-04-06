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

#[derive(Debug, Clone, Component, Default)]
pub struct Ingridient {
	pub ingridient_type: IngridientType,
	pub name: String,
	pub is_rare: bool,
	pub effects: SmallVec<[IngridientEffect;4]>,
}

#[derive(Clone, Copy, Debug)]
pub struct IngridientEffect {
	pub effect_type: Effect,
	pub duration: f32,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
// TODO_OLEG: Add more ingridient types
pub enum IngridientType {
	#[default]
	Plant,
	Mushroom,
	Berry,
}

impl Ingridient {
	#[allow(dead_code)]
	// TODO_OLEG: Generate random ingridients
	pub fn generate_random_ingridient(rng: &mut impl Rng, ingridient_type: IngridientType, is_rare: bool) -> Self {

		let name;
		let effects;

		match ingridient_type {
    		IngridientType::Mushroom => {
				const NAME_1: &[&str] = &["Smelly", "Witches"];
				const NAME_2: &[&str] = &["Toe"];

				name = format!("{} {}", NAME_1[rng.gen_range(0..NAME_1.len())], NAME_2[rng.gen_range(0..NAME_2.len())]);
				effects = SmallVec::new();
			},
			// More types here...
			IngridientType::Berry => todo!(),
			IngridientType::Plant => todo!(),
		}
		
		Ingridient { 
			name, 
			effects, 
			is_rare,
			ingridient_type,
		}
	}
}


