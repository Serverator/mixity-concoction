use bevy::{render::view::RenderLayers, math::{Vec4Swizzles, Vec3Swizzles}};

use crate::prelude::*;

use super::{ingredient::Ingredient, backpack::{InventoryItem, InventoryCamera}};

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
			item_grab_system
				).in_set(OnUpdate(GameState::InGame))
		)
		.init_resource::<GrabbedEntity>();
	}
}

#[derive(Resource, Default, Clone, Copy)]
pub struct GrabbedEntity(pub Option<Entity>);

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
	mut commands: Commands,
	mut materials: ResMut<Assets<FoliageMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>
) {
	commands.spawn((
		Mortar(false),
		Name::new("Mortar"),
		RigidBody::Dynamic,
		Velocity::default(),
		MaterialMeshBundle {
			mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 1.5, height: 0.3, resolution: 12, segments: 1 })),
			material: crate::assets::DEFAULT_FOLIAGE.get().unwrap().clone(),
			..default()
		},
		SecondWorldBundle::default()
	));

	commands.spawn((
		Pestle,
		Name::new("Pestle"),
		RigidBody::Dynamic,
		Velocity::default(),
		MaterialMeshBundle {
			mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.3, height: 1.0, resolution: 5, segments: 1 })),
			material: crate::assets::DEFAULT_FOLIAGE.get().unwrap().clone(),
			..default()
		},
		SecondWorldBundle::default()
	));
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

fn mash_ingridient(
	ingridient_query: Query<(Entity, &Ingredient, &Transform), With<InventoryItem>>,
	mortar_query: Query<(&Transform, &Velocity), (With<Mortar>, With<InventoryItem>)>,
	pestle_query: Query<(&Transform, &Velocity), (With<Pestle>, With<InventoryItem>)>,
) {

	for (ingredient_entity, ingredient_info, ingredient_transfrom) in &ingridient_query {

	}
	
}

fn item_grab_system(
	ingridient_query: Query<(Entity, &mut Transform, &GlobalTransform), With<InventoryItem>>,
	parent_query: Query<&Parent>,
	rapier_context: Res<RapierContext>,
	inventory_camera: Query<(&GlobalTransform, &Camera), With<InventoryCamera>>,
	windows: Query<&Window>,
	input: Query<&ActionState<Action>>,
	mut grabbed_entity: ResMut<GrabbedEntity>,
) {
	if input.single().just_released(Action::Click) { 
		if let Some(grabbed) = grabbed_entity.0 {
			// TODO: Deattach grabber
			info!("Ungrabbed {:?}", grabbed);
		}
		grabbed_entity.0 = None;
		return; 
	}

	// let .. else go brrrr
	if !input.single().just_pressed(Action::Click) { return; }

	let Some(mouse_position) = windows.single().cursor_position() else { return; };

	let Ok((camera_transform,camera)) = inventory_camera.get_single() else {
		warn!("Couldn't find inventory camera!");
		return;
	};

	let Some(ray) = camera.viewport_to_world(camera_transform, mouse_position) else { return; };

	let filter = QueryFilter::only_dynamic().exclude_sensors().groups(CollisionGroups::new(Group::GROUP_2, Group::GROUP_2));
	let Some((entity,distance)) = rapier_context.cast_ray(ray.origin, ray.direction, 100.0, true, filter) else { return; };

	let Some((entity, transform, global_transform)) = parent_query.iter_ancestors(entity).find_map(|parent| ingridient_query.get(parent).ok()) else { return; }; 
	
	grabbed_entity.0 = Some(entity);
	info!("Grabbed {:?}",entity);

}