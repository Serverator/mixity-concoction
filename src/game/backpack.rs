use bevy::{render::{view::RenderLayers, camera::ScalingMode}, core_pipeline::{clear_color::ClearColorConfig, fxaa::Fxaa}};

use crate::{prelude::*, assets::{self}};

pub struct BackpackPlugin;
impl Plugin for BackpackPlugin {
	fn build(&self, app: &mut App) {
		app
		.init_resource::<BackpackLocation>()
		.add_systems((
			spawn,
				).in_schedule(OnEnter(GameState::InGame))
		)
		//.add_system(update_inventory_items)
		.add_system(drop_items);
	}
}

#[derive(Resource,Default)]
pub struct BackpackLocation(pub Vec2);

#[derive(Component, Clone, Debug, Default, Reflect, FromReflect)]
pub struct Inventory(pub Vec<Entity>);

#[derive(Default, Component, Debug, Clone, Copy)]
pub struct InventoryItem {
	pub inventory: Option<Entity>,
	pub size: f32,
}

// fn update_inventory_items(
// 	mut inventory_item_query: Query<(&mut InventoryItem, &mut Transform)>,
// 	time: Res<Time>,
// ) {
// 	for (mut ii, mut transform) in &mut inventory_item_query {
// 		ii.current_size = (ii.current_size + time.delta_seconds() * 2.0).min(1.0);
// 		transform.scale = Vec3::splat(ii.current_size * ii.size)
// 	}
// }

#[derive(Default, Component)]
pub struct DroppedItem;

fn drop_items(
	mut commands: Commands,
	mut inventory_item_query: Query<(Entity, &mut Transform, &mut InventoryItem)>,
	transform_query: Query<&Transform, Without<InventoryItem>>,
) {
	for (dropped_item, mut transform, mut inventory_item) in inventory_item_query.iter_mut().filter(|(_,t,_)| t.translation.y < -12.0) {
		info!("Oops dropped an item!");
		let Some(inventory) = inventory_item.inventory else { continue; };
		let Ok(parent_transfrom) = transform_query.get(inventory) else {
			error!("Couldn't find transform for parent entity {:?}! Destroying dropped item...", inventory_item.inventory);
			commands.entity(dropped_item).despawn();
			continue;
		};

		inventory_item.inventory = None;

		commands.entity(dropped_item)
			.insert(DroppedItem::default())
			.insert(RenderLayers::layer(0))
			.insert(CollisionGroups::new(Group::GROUP_1 | Group::GROUP_3, Group::GROUP_1 | Group::GROUP_3));

		transform.translation = parent_transfrom.translation + Vec3::Y;
	}
}

#[derive(Bundle,Debug)]
struct BackpackWallBundle {
	pub mesh: Handle<Mesh>,
    pub material: Handle<FoliageMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
	collider: Collider,
	render_layer: RenderLayers,
	collision_group: CollisionGroups,
}

impl Default for BackpackWallBundle {
    fn default() -> Self {
        Self { 
			mesh: Default::default(), 
			transform: Default::default(), 
			global_transform: Default::default(), 
			visibility: Default::default(), 
			computed_visibility: Default::default(), 
			material: assets::DEFAULT_FOLIAGE.get().unwrap().clone(), 
			collider: Collider::cuboid(0.5,0.5,0.5),
			render_layer: RenderLayers::layer(2),
			collision_group: CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
		}
    }
}

#[derive(Component)]
pub struct InventoryCamera;

fn spawn(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut foliage_mat: ResMut<Assets<FoliageMaterial>>,
	mut backpack_location: ResMut<BackpackLocation>,
) {
	// Inventory camera
	commands.spawn((
		Name::new("Inventory Camera"),
		InventoryCamera,
		Camera3dBundle {
			projection: Projection::Orthographic(OrthographicProjection {
				scale: 1.0,
				viewport_origin: Vec2::new(0.5,0.5),
				scaling_mode: ScalingMode::FixedVertical(10.0),
				..default()
			}),
			transform: Transform::from_translation(Vec3::Z * 100.0),
			camera_3d: Camera3d {
				clear_color: ClearColorConfig::None,
				..default()
			},
			camera: Camera {
				
				order: 1,
				..default()
			},

			..default()
		},
		RenderLayers::layer(2),
		Fxaa::default(),
	));

	let cube = meshes.add(Mesh::from(shape::Cube::new(1.0)));

	backpack_location.0 = Vec2::new(-5.0, 3.5);


	commands.spawn((
		Name::new("Backpack"),
		RigidBody::Fixed,
		VisibilityBundle::default(),
		TransformBundle::from_transform(Transform::from_xyz(-5.0, 0.0, 0.0)),
		RenderLayers::layer(2),
		CollisionGroups::new(Group::GROUP_2,Group::GROUP_2),
	)).with_children(|commands| {

		commands.spawn(BackpackWallBundle {
			mesh: cube.clone(),
			transform: Transform::from_translation(Vec3::NEG_Y).with_scale(Vec3::new(2.5,0.2,2.0)),
			..default()
		});

		commands.spawn(BackpackWallBundle {
			mesh: cube.clone(),
			transform: Transform::from_translation(Vec3::new(-1.25,0.5,0.0)).with_scale(Vec3::new(0.2,3.0,2.0)),
			..default()
		});

		commands.spawn(BackpackWallBundle {
			mesh: cube.clone(),
			transform: Transform::from_translation(Vec3::new(1.25,0.5,0.0)).with_scale(Vec3::new(0.2,3.0,2.0)),
			..default()
		});

		commands.spawn(BackpackWallBundle {
			mesh: cube.clone(),
			material: foliage_mat.add(FoliageMaterial { color: Color::GRAY, ..default() }),
			transform: Transform::from_translation(Vec3::new(0.0,0.5,-0.75)).with_scale(Vec3::new(2.5,3.0,0.2)),
			..default()
		});

		commands.spawn(BackpackWallBundle {
			transform: Transform::from_translation(Vec3::new(0.0,0.5,0.75)).with_scale(Vec3::new(2.5,3.0,0.2)),
			..default()
		});
	});
}