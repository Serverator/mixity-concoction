use crate::prelude::*;

use super::effects::Effect;


#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct Ingredient {
	pub ingredient_type: IngredientType,
	pub name: String,
	pub is_rare: bool,
	pub effects: SmallVec<[IngredientEffect;4]>,
}

#[derive(Clone, Copy, Debug, Reflect, FromReflect)]
pub struct IngredientEffect {
	pub effect_type: Effect,
	pub duration: f32,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Reflect, FromReflect)]
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