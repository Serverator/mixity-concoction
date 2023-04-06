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
pub struct Ingredient {
	pub ingredient_type: IngredientType,
	pub name: String,
	pub is_rare: bool,
	pub effects: SmallVec<[IngredientEffect;4]>,
}

#[derive(Clone, Copy, Debug)]
pub struct IngredientEffect {
	pub effect_type: Effect,
	pub duration: f32,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
// TODO_OLEG: Add more ingredient types
pub enum IngredientType {
	#[default]
	Plant,
	Mushroom,
	Berry,
}

impl Ingredient {
	#[allow(dead_code)]
	// TODO_OLEG: Generate random ingredients
	pub fn generate_random_ingredient(rng: &mut impl Rng, ingredient_type: IngredientType, is_rare: bool) -> Self {

		let name;
		let effects;

		match ingredient_type {
    		IngredientType::Mushroom => {
				const NAME_1: &[&str] = &["Smelly", "Witches"];
				const NAME_2: &[&str] = &["Toe"];

				name = format!("{} {}", NAME_1[rng.gen_range(0..NAME_1.len())], NAME_2[rng.gen_range(0..NAME_2.len())]);
				effects = SmallVec::new();
			},
			// More types here...
			IngredientType::Berry => todo!(),
			IngredientType::Plant => todo!(),
		}
		
		Ingredient { 
			name, 
			effects, 
			is_rare,
			ingredient_type,
		}
	}
}


