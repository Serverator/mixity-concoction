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
	Root,
}

impl Ingredient {
	#[allow(dead_code)]
	// TODO_OLEG: Generate random ingredients
	pub fn generate_random_ingredient(rng: &mut impl Rng, ingredient_type: IngredientType, is_rare: bool) -> Self {

		let name;
		let effects;

		match ingredient_type {
    		IngredientType::Mushroom => {
				const NAME_1: &[&str] = &["Smelly", "Witches", "Amoria", "Sarconia", "Omamita", "Delecia", "Pcilocube"];
				const NAME_2: &[&str] = &["Toe", "Falloides", "Uscaria", "Ubensis", "Azuresense"];

				name = format!("{} {}", NAME_1[rng.gen_range(0..NAME_1.len())], NAME_2[rng.gen_range(0..NAME_2.len())]);
				effects = SmallVec::new();
			},
			// More types here...
			IngredientType::Berry => {
				const NAME_1: &[&str] = &["Badapple", "Strong", "Small", "Vomit", "Run", "Watch", "Bevy", "Sheet", "Rust", "Wolf", "Fox", "Cram", "Don't_eat_me-"];
				const NAME_2: &[&str] = &["berry"];

				name = format!("{}{}", NAME_1[rng.gen_range(0..NAME_1.len())], NAME_2[rng.gen_range(0..NAME_2.len())]);
				effects = SmallVec::new();
			},
			IngredientType::Plant => {
				const NAME_1: &[&str] = &["Bat", "Fox", "Bear", "Wolf", "Troll", "Ogre", "Moose", "Slime", "Rabbit"];
				const NAME_2: &[&str] = &["ear", "tongue", "eye", "skin", "claw", "finger", "tail", "wing", "scale", "fang", "horn"];

				name = format!("{} {}", NAME_1[rng.gen_range(0..NAME_1.len())], NAME_2[rng.gen_range(0..NAME_2.len())]);
				effects = SmallVec::new();
			},
			IngredientType::Root => {
				const NAME_1: &[&str] = &["Some ", "Another ", "This ", "That ", "Any ", "Car", "The ", "Smog", "Melon", "Slimy ", "Straight ", "Curvy "];
				const NAME_2: &[&str] = &["root"];

				name = format!("{}{}", NAME_1[rng.gen_range(0..NAME_1.len())], NAME_2[rng.gen_range(0..NAME_2.len())]);
				effects = SmallVec::new();
			},
		}
		
		Ingredient { 
			name, 
			effects, 
			is_rare,
			ingredient_type,
		}
	}
}