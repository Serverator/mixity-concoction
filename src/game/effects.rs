use bevy::math::Vec3Swizzles;
use bevy_inspector_egui::egui::lerp;

use crate::{choice, prelude::*};

use super::{
	ingredient::{Grind, Ingredient},
	items::{DroppedItem, Item},
	player::Player,
	world::SpawnableInstance,
};

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems((spawn_arrows,).in_schedule(OnEnter(GameState::InGame)))
			.add_systems(
				(effect_tick, rotate_arrow, gravity_effects, earthquake)
					.in_set(OnUpdate(GameState::InGame)),
			)
			.insert_resource(ActiveEffects::default());
	}
}

pub fn effect_tick(mut effects: ResMut<ActiveEffects>, time: Res<Time>) {
	for effect in effects.0.iter_mut() {
		effect.time_left -= time.delta_seconds();
	}
	effects.0.retain(|e| e.time_left >= 0.0);
}

#[derive(Component)]
pub struct Arrow;

#[derive(Component)]
pub struct RareArrow;

pub fn spawn_arrows(mut commands: Commands, game_assets: Res<GameAssets>) {
	commands.spawn((
		Arrow,
		SceneBundle {
			visibility: Visibility::Hidden,
			scene: game_assets.arrow_scene.clone(),
			transform: Transform::from_scale(Vec3::new(0.2, 0.5, 0.2)),
			..default()
		},
		NamedMaterials(smallvec![NamedMaterial::new("Arrow", Color::WHITE)]),
	));

	commands.spawn((
		RareArrow,
		SceneBundle {
			visibility: Visibility::Hidden,
			scene: game_assets.arrow_scene.clone(),
			transform: Transform::from_scale(Vec3::new(0.3, 0.5, 0.3)),
			..default()
		},
		NamedMaterials(smallvec![NamedMaterial::new(
			"Arrow",
			Color::rgb(0.7, 0.7, 1.0)
		)]),
	));
}

pub fn rotate_arrow(
	player: Query<&Transform, (With<Player>, Without<Arrow>, Without<RareArrow>)>,
	mut arrow: Query<
		(&mut Transform, &mut Visibility),
		(With<Arrow>, Without<RareArrow>, Without<Player>),
	>,
	mut rare_arrow: Query<
		(&mut Transform, &mut Visibility),
		(With<RareArrow>, Without<Arrow>, Without<Player>),
	>,
	ingredient_query: Query<
		(&Transform, &Ingredient, &SpawnableInstance),
		(
			Without<DroppedItem>,
			Without<Player>,
			Without<Arrow>,
			Without<RareArrow>,
		),
	>,
	effects: Res<ActiveEffects>,
	time: Res<Time>,
) {
	let Ok((mut arrow_transform, mut arrow_visibility)) = arrow.get_single_mut() else { return; };
	let Ok((mut rare_arrow_transform, mut rare_arrow_visibility)) = rare_arrow.get_single_mut() else { return; };

	let rare_effect = effects.has_effect(EffectType::RareArrows);
	let effect = effects.has_effect(EffectType::Arrow);

	*arrow_visibility = if effect.is_some() {
		Visibility::Visible
	} else {
		Visibility::Hidden
	};
	*rare_arrow_visibility = if rare_effect.is_some() {
		Visibility::Visible
	} else {
		Visibility::Hidden
	};

	let player_transform = player.single();

	if rare_effect.is_some() {
		let closest_rare = ingredient_query
			.iter()
			.filter(|i| i.1.is_rare)
			.map(|a| {
				(
					a.0,
					a.0.translation
						.distance_squared(player_transform.translation),
				)
			})
			.min_by(|a, b| a.1.total_cmp(&b.1));

		if let Some((closest, distance)) = closest_rare {
			rare_arrow_transform.rotation = Quat::slerp(
				rare_arrow_transform.rotation,
				Quat::from_rotation_arc(
					Vec3::Z,
					(player_transform.translation - closest.translation)
						.xz()
						.extend(0.0)
						.xzy()
						.normalize(),
				),
				1.0 - 0.001f32.powf(time.delta_seconds()),
			);
			rare_arrow_transform.translation = player_transform.translation
				+ rare_arrow_transform.forward() * 1.0f32.min(distance.sqrt() - 1.5);
		} else {
			*rare_arrow_visibility = Visibility::Hidden;
		}
	}

	if effect.is_some() {
		let closest = ingredient_query
			.iter()
			.map(|a| {
				(
					a.0,
					a.0.translation
						.distance_squared(player_transform.translation),
				)
			})
			.min_by(|a, b| a.1.total_cmp(&b.1));
		if let Some((closest, distance)) = closest {
			arrow_transform.rotation = Quat::slerp(
				arrow_transform.rotation,
				Quat::from_rotation_arc(
					Vec3::Z,
					(player_transform.translation - closest.translation)
						.xz()
						.extend(0.0)
						.xzy()
						.normalize(),
				),
				1.0 - 0.001f32.powf(time.delta_seconds()),
			);
			arrow_transform.translation = player_transform.translation
				+ arrow_transform.forward() * 1.0f32.min(distance.sqrt() - 1.5);
		} else {
			*arrow_visibility = Visibility::Hidden;
		}
	}
}

fn gravity_effects(active_effects: Res<ActiveEffects>, mut config: ResMut<RapierConfiguration>) {
	if active_effects.has_effect(EffectType::NoGravity).is_some() {
		config.gravity = Vec3::splat(0.0);
	} else if let Some(gravity) = active_effects.has_effect(EffectType::LowGravity) {
		config.gravity = Vec3::Y * lerp(-6.0..=-2.0, gravity.potency);
	} else {
		config.gravity = Vec3::Y * -9.8;
	}
}

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum EffectType {
	Haste,            // DONE
	Slowness,         // DONE
	Arrow,            // DONE
	Earthquake,       // DONE
	SpawnBall,        // WIP
	BackpackBackflip, //WIP
	SmallInstruments, //WIP
	InvisibleIngridients,
	TreeRockets,
	NoGravity,  // DONE
	LowGravity, // DONE
	WhackyGravity,
	LuckyHands,
	Hallucinations, // DONE
	Thief,
	RareArrows, // DONE
	GodMode,
}

pub fn earthquake(
	mut velocities: Query<&mut Velocity, With<DroppedItem>>,
	mut velocities_ii: Query<&mut Velocity, (With<Item>, Without<DroppedItem>)>,
	active_effects: Res<ActiveEffects>,
	time: Res<Time>,
	mut last_boom: Local<f32>,
) {
	if let Some(earthquake) = active_effects.has_effect(EffectType::Earthquake) {
		if time.elapsed_seconds() > *last_boom {
			let mut rng = thread_rng();
			*last_boom = time.elapsed_seconds() + rng.gen_range(0.5..3.0);
			for mut velocity in &mut velocities {
				velocity.linvel += Vec3::new(
					rng.gen_range(-1.0..1.0),
					rng.gen_range(-1.0..1.0),
					rng.gen_range(-1.0..1.0),
				) * lerp(1.0..=5.0, earthquake.potency);
			}
			for mut velocity in &mut velocities_ii {
				velocity.linvel +=
					Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0)
						* lerp(1.0..=5.0, earthquake.potency);
			}
		}
	}
}

/// Effects and time left for them to wear off
#[derive(Clone, Copy, Debug, Reflect, FromReflect)]
pub struct Effect {
	pub effect: EffectType,
	pub potency: f32,
	pub time_left: f32,
}

/// Added to the player to determine active effects
#[derive(Resource, Clone, Debug, Default)]
pub struct ActiveEffects(pub SmallVec<[Effect; 12]>);

impl ActiveEffects {
	pub fn has_effect(&self, effect_type: EffectType) -> Option<Effect> {
		self.0
			.iter()
			.filter(|x| x.effect == effect_type)
			.max_by(|a, b| a.potency.total_cmp(&b.potency))
			.copied()
	}
}

pub fn generate_qp_from_ingredients(ingridients: &[Ingredient]) -> (f32, f32) {
	let mut quality: f32 = 0.5;
	let mut potency: f32 = 0.5;

	let len = ingridients.len();

	potency += match len {
		0 => return (0.0, 0.0),
		1..=3 => (4 - len) as f32 * -0.1,
		4 => 0.1,
		5..=6 => 0.3,
		7 => 0.1,
		8.. => (8 - len) as f32 * 0.1,
		_ => 0.0,
	};
	potency *= ingridients.iter().fold(1.0, |acc, c| acc * c.size);

	quality -= len as f32 * 0.05;
	quality += ingridients.iter().filter(|c| c.is_rare).count() as f32 * 0.20;
	quality += ingridients
		.iter()
		.filter(|c| c.grind == Grind::Grinded)
		.count() as f32
		* 0.10;

	(quality.clamp(0.1, 1.0), potency.clamp(0.1, 1.0))
}

// Potion quality and potency goes from 0.0 to 1.0
pub fn generate_effects_from_qp(quality: f32, potency: f32) -> Vec<Effect> {
	debug_assert!((0.0..=1.0).contains(&quality) || (0.0..=1.0).contains(&potency));

	if quality == 0.0 {
		return vec![];
	}

	let mut rng = thread_rng();

	let mut effect_quality = vec![];

	for _i in 0..(potency / 0.45).ceil() as usize {
		let roll = rng.gen::<f32>();

		let weighted_roll = if quality < 0.5 {
			lerp(roll..=0.0, (0.5 - quality) * 2.0)
		} else {
			lerp(roll..=1.0, (quality - 0.5) * 2.0)
		};

		effect_quality.push(match weighted_roll {
			x if x <= 0.1 => EffectQuality::Catastrophic,
			x if x > 0.1 && x <= 0.4 => EffectQuality::Negative,
			x if x > 0.4 && x <= 0.6 => EffectQuality::Neutral,
			x if x > 0.6 && x <= 0.9 => EffectQuality::Positive,
			x if x > 0.9 => EffectQuality::Exceptional,
			_ => panic!(),
		});
	}

	let roll = rng.gen::<f32>();

	let potency = if quality < 0.5 {
		lerp(roll..=0.0, (0.5 - quality) * 2.0)
	} else {
		lerp(roll..=1.0, (quality - 0.5) * 2.0)
	};

	effect_quality
		.iter()
		.map(|i| Effect {
			effect: generate_effect(i, &mut rng),
			potency,
			time_left: (rng.gen_range(30.0..120.0) * potency).max(10.0),
		})
		.collect()
}

pub enum EffectQuality {
	Catastrophic,
	Negative,
	Neutral,
	Positive,
	Exceptional,
}

pub fn generate_effect(effect_quality: &EffectQuality, rng: &mut impl Rng) -> EffectType {
	use EffectType::*;

	let choice = match effect_quality {
		EffectQuality::Catastrophic => {
			const CATASTROPHIC: Choices<EffectType> = choice!(Earthquake,);
			CATASTROPHIC
		}
		EffectQuality::Negative => {
			const NEGATIVE: Choices<EffectType> = choice!(Slowness, Hallucinations,);
			NEGATIVE
		}
		EffectQuality::Neutral => {
			const NEUTRAL: Choices<EffectType> = choice!(
				//SpawnBall,
				//TreeRockets,
				NoGravity,
			);
			NEUTRAL
		}
		EffectQuality::Positive => {
			const POSITIVE: Choices<EffectType> = choice!(
				Haste, Arrow,
				//LuckyHands,
			);
			POSITIVE
		}
		EffectQuality::Exceptional => {
			const EXCEPTIONAL: Choices<EffectType> = choice!(
				//GodMode,
				RareArrows,
			);
			EXCEPTIONAL
		}
	};

	*choice.random(rng)
}
