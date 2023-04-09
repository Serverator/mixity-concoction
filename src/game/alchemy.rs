use bevy::{render::view::RenderLayers, math::{Vec4Swizzles, Vec3Swizzles}};

use crate::prelude::*;

pub struct AlchemyPlugin;
impl Plugin for AlchemyPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_systems(( // Run on game start
			init_alchemy_table,
				).in_schedule(OnEnter(GameState::InGame))
		)
		.add_systems(( // Update in game state
			check_mortar_chushing,
				).in_set(OnUpdate(GameState::InGame))
		);
	}
}

#[derive(Component)]
pub struct AlchemyTable;

#[derive(Component)]
pub struct Mortar(bool);

#[derive(Component)]
pub struct Pestle;

#[derive(Bundle)]
pub struct SecondWorldBundle {
	pub render_layer: RenderLayers,
	pub collision_group: CollisionGroups,
}

impl Default for SecondWorldBundle {
fn default() -> Self {
        Self {  
			render_layer: RenderLayers::layer(2),
			collision_group: CollisionGroups::new(Group::GROUP_2,Group::GROUP_2)
		}
    }
}

fn init_alchemy_table(
	_commands: Commands,
	_materials: ResMut<Assets<FoliageMaterial>>,
	_meshes: ResMut<Assets<Mesh>>
) {
	//commands.spawn((
	//	Mortar(false),
	//	Name::new("Mortar"),
	//	RigidBody::Dynamic,
	//	Velocity::default(),
	//	MaterialMeshBundle {
	//		mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 1.5, height: 0.3, resolution: 12, segments: 1 })),
	//		material: crate::assets::DEFAULT_FOLIAGE.get().unwrap().clone(),
	//		..default()
	//	},
	//	SecondWorldBundle::default()
	//));
//
	//commands.spawn((
	//	Pestle,
	//	Name::new("Pestle"),
	//	RigidBody::Dynamic,
	//	Velocity::default(),
	//	MaterialMeshBundle {
	//		mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.3, height: 1.0, resolution: 5, segments: 1 })),
	//		material: crate::assets::DEFAULT_FOLIAGE.get().unwrap().clone(),
	//		..default()
	//	},
	//	SecondWorldBundle::default()
	//));
}

#[allow(clippy::type_complexity)]
fn check_mortar_chushing(
	mut mortar_query: Query<(&GlobalTransform, &mut Mortar)>,
	pestle_query: Query<(&GlobalTransform, &Velocity), (With<Pestle>, Without<Mortar>)>,
) {
	const MORTAR_RADIUS: f32 = 0.6;

	for (mortar_transfrom, mut mortar) in &mut mortar_query {
		let mut is_crushing = false;
		let (_, mortar_rotation, _) = mortar_transfrom.to_scale_rotation_translation();

		for (pestle_transfrom,pestle_velocity) in &pestle_query {
			// Velocity relative to the mortar normal
			let relative_vel = mortar_rotation * pestle_velocity.linvel;
			// Return early if velocity perpendicular to the mortar normal is greater, than parallel velocity.
			if relative_vel.y > -1.0 || relative_vel.xz().length_squared() > relative_vel.y.powi(2) {
				continue;
			}
			
			// Pestle translation, relative to mortar (Y is mortar normal)
			let peslte_relative_translation = (mortar_transfrom.compute_matrix().inverse() * pestle_transfrom.translation().extend(1.0)).xyz();

			// Check if pestle is located within mortar radius
			if !(0.0..1.0).contains(&peslte_relative_translation.y) || peslte_relative_translation.xz().length_squared() > MORTAR_RADIUS.powi(2) {
				continue;
			}

			is_crushing = true;
		}
		mortar.0 = is_crushing
	}
} 


// #[allow(clippy::type_complexity)]
// fn mash_ingridient(
// 	ingridient_query: Query<(Entity, &Ingredient, &Transform), With<InventoryItem>>,
// 	_mortar_query: Query<(&Transform, &Velocity), (With<Mortar>, With<InventoryItem>)>,
// 	_pestle_query: Query<(&Transform, &Velocity), (With<Pestle>, With<InventoryItem>)>,
// ) {

// 	for (_ingredient_entity, _ingredient_info, _ingredient_transfrom) in &ingridient_query {

// 	}
	
// }

