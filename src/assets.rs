use crate::prelude::*;
use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::{
	gltf::Gltf,
	reflect::TypeUuid,
	render::{once_cell::sync::OnceCell, view::RenderLayers},
	scene::SceneInstance,
};
pub use bevy_asset_loader::prelude::*;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
	fn build(&self, app: &mut App) {
		app.add_asset::<Spawnable>()
			.add_loading_state(
				LoadingState::new(GameState::LoadingAssets)
					.continue_to_state(GameState::GeneratingWorld),
			)
			.add_collection_to_loading_state::<_, GameAssets>(GameState::LoadingAssets)
			.add_system(setup.in_schedule(OnExit(GameState::LoadingAssets)))
			.add_systems((
				check_scene_init,
				update_scene_children::<RenderLayers, With<Handle<Mesh>>>,
				update_scene_children::<CollisionGroups, With<Collider>>,
			));
	}
}

#[derive(Resource)]
pub struct CalculatedColliders {
	pub cauldron_collider: Collider,
	pub mortar_collider: Collider,
	pub potions: Vec<Collider>,
}

pub static SHADOW_BUNDLE: OnceCell<(Handle<StandardMaterial>, Handle<Mesh>)> = OnceCell::new();

pub static DEFAULT_FOLIAGE: OnceCell<Handle<FoliageMaterial>> = OnceCell::new();

fn setup(
	mut commands: Commands,
	mut spawnable_assets: ResMut<Assets<Spawnable>>,
	mut mesh_assets: ResMut<Assets<Mesh>>,
	mut material_assets: ResMut<Assets<StandardMaterial>>,
	mut foliage_assets: ResMut<Assets<FoliageMaterial>>,
	game_assets: Res<GameAssets>,
	scene_assets: Res<Assets<Scene>>,
	gltfs: Res<Assets<Gltf>>,
) {
	DEFAULT_FOLIAGE
		.set(foliage_assets.add(FoliageMaterial::default()))
		.unwrap();

	let plane_mesh = mesh_assets.add(Mesh::from(shape::Plane {
		size: 1.0,
		subdivisions: 0,
	}));

	let shadow_material = material_assets.add(StandardMaterial {
		base_color_texture: Some(game_assets.circle_texture.clone()),
		alpha_mode: AlphaMode::Mask(0.5),
		base_color: Color::rgb(0.6 * 0.9, 0.8 * 0.9, 0.2 * 0.9),
		unlit: true,
		..default()
	});

	SHADOW_BUNDLE.set((shadow_material, plane_mesh)).unwrap();

	let tree_scenes = &gltfs.get(&game_assets.tree_gltf).unwrap().scenes;

	for (i, scene) in tree_scenes.iter().enumerate() {
		let spawnable = Spawnable {
			id: i,
			archetype: SpawnableArchetype::Tree,
			scene: scene.clone(),
			ingredient: None,
			spawn_weight: 1.2 / tree_scenes.len() as f32,
			size: 2.8,
			collider: Some(Collider::capsule(Vec3::ZERO, Vec3::Y * 2.0, 0.4)),
		};
		spawnable_assets.add(spawnable);
	}

	let bush_scenes = &gltfs.get(&game_assets.bush_gltf).unwrap().scenes;
	for (i, scene) in bush_scenes.iter().enumerate() {
		let spawnable = Spawnable {
			id: i,
			archetype: SpawnableArchetype::Bush,
			scene: scene.clone(),
			ingredient: match i {
				2 => Some(SpawnableIngredient {
					pick_event: PickUpEvent::RemoveNamedChild("Berry"),
					inventory_scene: game_assets.berry_scene.clone(),
					collider: Collider::compound(vec![
						(
							Vec3::new(0.0, 0.1, 0.0),
							Quat::IDENTITY,
							Collider::round_cuboid(0.14, 0.08, 0.18, 0.04),
						),
						(
							Vec3::new(0.0, 0.2, 0.0),
							Quat::IDENTITY,
							Collider::ball(0.1),
						),
					]),
				}),
				_ => None,
			},
			spawn_weight: match i {
				2 => 0.3,
				_ => 1.0,
			} / bush_scenes.len() as f32,
			size: match i {
				0 => 1.5,
				_ => 2.0,
			},
			collider: Some(Collider::capsule(Vec3::ZERO, Vec3::Y * 2.0, 0.2)),
		};
		spawnable_assets.add(spawnable);
	}

	let ingredient_scenes = &gltfs.get(&game_assets.mushrooms_gltf).unwrap().scenes;
	for (i, scene) in ingredient_scenes.iter().enumerate() {
		let spawnable = Spawnable {
			id: i,
			archetype: SpawnableArchetype::Mushroom,
			scene: scene.clone(),
			ingredient: Some(SpawnableIngredient {
				pick_event: PickUpEvent::Destroy,
				inventory_scene: scene.clone(),
				collider: match i {
					0 => {
						Collider::compound(vec![
							//(Vec3::new(0.0, 0.21, 0.0),Quat::IDENTITY,Collider::ball(0.25)),
							(
								Vec3::new(0.0, 0.38, 0.0),
								Quat::IDENTITY,
								Collider::round_cone(0.08, 0.26, 0.04),
							),
							(
								Vec3::new(0.0, 0.02, 0.0),
								Quat::IDENTITY,
								Collider::capsule(Vec3::ZERO, Vec3::Y * 0.26, 0.09),
							),
						])
					}
					1 => {
						Collider::compound(vec![
							//(Vec3::new(0.0, 0.21, 0.0),Quat::IDENTITY,Collider::ball(0.25)),
							(
								Vec3::new(0.0, 0.44, 0.0),
								Quat::IDENTITY,
								Collider::round_cone(0.03, 0.26, 0.04),
							),
							(
								Vec3::new(0.0, 0.02, 0.0),
								Quat::IDENTITY,
								Collider::capsule(Vec3::ZERO, Vec3::Y * 0.30, 0.06),
							),
						])
					}
					_ => Collider::default(),
				},
			}),
			spawn_weight: 0.3 / ingredient_scenes.len() as f32,
			size: 0.6,
			collider: None,
		};
		spawnable_assets.add(spawnable);
	}

	// YES I DO IT AT RUNTIME, NO TIME TO FIX BUCK OFF
	// SERDE DOES NOT WANT TO COOPERATE, SO YOU'LL HAVE TO WAIT 10 SECONDS OF LOADING SCREEN
	commands.insert_resource(CalculatedColliders {
		cauldron_collider: compute_collider(
			&game_assets.cauldron_scene,
			&scene_assets,
			&mesh_assets,
			Some("Cauldron"),
		),
		mortar_collider: compute_collider(
			&game_assets.mortar_scene,
			&scene_assets,
			&mesh_assets,
			None,
		),
		potions: gltfs
			.get(&game_assets.potions_gltf)
			.unwrap()
			.scenes
			.iter()
			.map(|s| compute_collider(s, &scene_assets, &mesh_assets, None))
			.collect(),
	});
}

#[derive(Clone, Debug, Default)]
pub struct SpawnableIngredient {
	pub pick_event: PickUpEvent,
	pub inventory_scene: Handle<Scene>,
	pub collider: Collider,
}

pub fn compute_collider(
	scene: &Handle<Scene>,
	scenes: &Assets<Scene>,
	meshes: &Assets<Mesh>,
	with_name: Option<&'static str>,
) -> Collider {
	let scene = scenes.get(scene).unwrap();
	let collider_shape = ComputedColliderShape::ConvexDecomposition(VHACDParameters::default());
	// I WILL NEST AS MUCH SHAPES AS I WANT BITCHES
	let shapes = scene
		.world
		.iter_entities()
		.filter(|entity| {
			if let Some(with_name) = with_name {
				entity
					.get::<Name>()
					.map(|name| name.contains(with_name))
					.unwrap_or_default()
			} else {
				true
			}
		})
		.filter_map(|entity| entity.get::<Handle<Mesh>>())
		.filter_map(|mesh_handle| meshes.get(mesh_handle))
		.filter_map(|mesh| Collider::from_bevy_mesh(mesh, &collider_shape))
		.filter_map(|col| {
			col.as_compound()
				.map(|comp| {
					comp.raw.shapes().iter().map(|(pos, shape)| {
						let (tra, rot) = (*pos).into();
						(tra, rot, shape.clone().into())
					})
				})
				.map(|a| a.collect::<Vec<_>>())
		})
		.fold(vec![], |mut acc, iter| {
			acc.extend(iter);
			acc
		});
	Collider::compound(shapes)
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum PickUpEvent {
	#[default]
	/// Destroys the original entity entirely
	Destroy,
	/// Removes a child with specified name
	RemoveNamedChild(&'static str),
	/// Replaces scene from one to another
	Replace(Handle<Scene>),
}

#[derive(TypeUuid)]
#[uuid = "2e680e06-a271-4804-8f5a-73927db8dec4"]
pub struct Spawnable {
	pub id: usize,
	pub archetype: SpawnableArchetype,
	pub scene: Handle<Scene>,
	pub ingredient: Option<SpawnableIngredient>,
	pub spawn_weight: f32,
	pub size: f32,
	pub collider: Option<Collider>,
}

impl PartialEq for Spawnable {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id && self.archetype == other.archetype
	}
}

// Use hash from Scene handle. Each scene handle SHOULD have only one spawnable.
impl std::hash::Hash for Spawnable {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
		self.archetype.hash(state);
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpawnableArchetype {
	Tree,
	Bush,
	Mushroom,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
	// Textures
	#[asset(path = "textures/dither.png")]
	pub dither_texture: Handle<Image>,
	#[asset(path = "textures/circle.png")]
	pub circle_texture: Handle<Image>,
	// World spawnables
	#[asset(path = "models/tree.gltf")]
	pub tree_gltf: Handle<Gltf>,
	#[asset(path = "models/bush.gltf")]
	pub bush_gltf: Handle<Gltf>,
	#[asset(path = "models/mushrooms.gltf")]
	pub mushrooms_gltf: Handle<Gltf>,
	#[asset(path = "models/floating_island.gltf#Scene0")]
	pub floating_island_scene: Handle<Scene>,
	#[asset(path = "models/arrow.gltf#Scene0")]
	pub arrow_scene: Handle<Scene>,
	// Inventory spawnables
	#[asset(path = "models/backpack.gltf")]
	pub backpack_gltf: Handle<Gltf>,
	#[asset(path = "models/potions.gltf")]
	pub potions_gltf: Handle<Gltf>,
	// Alchemy tools
	#[asset(path = "models/alchemy.gltf#Scene0")]
	pub cauldron_scene: Handle<Scene>,
	#[asset(path = "models/alchemy.gltf#Scene1")]
	pub mortar_scene: Handle<Scene>,
	#[asset(path = "models/alchemy.gltf#Scene2")]
	pub pestle_scene: Handle<Scene>,
	#[asset(path = "models/alchemy.gltf#Scene3")]
	// Ingredients
	pub table_scene: Handle<Scene>,
	#[asset(path = "models/ingredients.gltf#Scene0")]
	pub crushed_ingredient_scene: Handle<Scene>,
	#[asset(path = "models/ingredients.gltf#Scene1")]
	pub berry_scene: Handle<Scene>,
	// Player
	#[asset(path = "models/player.gltf#Scene1")]
	pub player_scene: Handle<Scene>,
	#[asset(path = "models/player.gltf#Scene0")]
	pub player_head_scene: Handle<Scene>,
	// Music-ish
	#[asset(path = "sounds/music.ogg")]
	pub music: Handle<AudioSource>,
	// Sounds
	#[asset(
		paths(
			"sounds/pickup.ogg",
			"sounds/pickup2.ogg",
			"sounds/pickup3.ogg",
			"sounds/pickup4.ogg"
		),
		collection(typed)
	)]
	pub pickup_sound: Vec<Handle<AudioSource>>,
	#[asset(path = "sounds/insanity.ogg")]
	pub insanity_sound: Handle<AudioSource>,
	#[asset(path = "sounds/drink.ogg")]
	pub drink_sound: Handle<AudioSource>,
	#[asset(path = "sounds/wha.ogg")]
	pub wha_sound: Handle<AudioSource>,
	#[asset(path = "sounds/item_drop.ogg")]
	pub drop_item_sound: Handle<AudioSource>,
	#[asset(path = "sounds/eat.ogg")]
	pub eat_sound: Handle<AudioSource>,
	#[asset(path = "sounds/sploosh.ogg")]
	pub sploosh_sound: Handle<AudioSource>,
	#[asset(path = "sounds/suck_air.ogg")]
	pub suck_air_sound: Handle<AudioSource>,
	#[asset(path = "sounds/blah.ogg")]
	pub blah_sound: Handle<AudioSource>,
	#[asset(path = "sounds/rare.ogg")]
	pub rare_sound: Handle<AudioSource>,
	#[asset(path = "sounds/delishs.ogg")]
	pub delishs_sound: Handle<AudioSource>,
	#[asset(path = "sounds/filling_potion.ogg")]
	pub filling_potion_sound: Handle<AudioSource>,
	#[asset(
		paths(
			"sounds/grind_1.ogg",
			"sounds/grind_2.ogg",
			"sounds/grind_3.ogg",
			"sounds/grind_4.ogg"
		),
		collection(typed)
	)]
	pub grind_sound: Vec<Handle<AudioSource>>,
}

#[derive(Component)]
pub struct SceneInstanceReady;

/// Checks if scene instance is ready and adds a `SceneInstanceReady` component to it
pub fn check_scene_init(
	mut commands: Commands,
	scene_manager: Res<SceneSpawner>,
	scene_query: Query<(Entity, &SceneInstance), Without<SceneInstanceReady>>,
	//changed_scene:  Query<(Entity, &SceneInstance), (With<SceneInstanceReady>, Changed<SceneInstance>)>,
) {
	for (entity, scene_id) in &scene_query {
		if scene_manager.instance_is_ready(**scene_id) {
			commands.entity(entity).insert(SceneInstanceReady);
		}
	}
}

/// Gtlf importer is absolute shite. <br>
/// To metigate this, I created this little system, that will apply some of the components from the original scene entity to all of it's descendants (With filters) <br>
/// It janky. It work. It stay.
pub fn update_scene_children<T: Component + Clone, F: ReadOnlyWorldQuery>(
	mut commands: Commands,
	children_query: Query<&Children>,
	scene_query: Query<
		(Entity, &T),
		(
			With<SceneInstanceReady>,
			Or<(Added<SceneInstanceReady>, Changed<T>)>,
		),
	>,
	mut t_query: Query<Option<&mut T>, (F, Without<SceneInstanceReady>)>,
) {
	for (parent, parent_t) in &scene_query {
		for child in children_query.iter_descendants(parent) {
			// Check filters
			let Ok(maybe_t) = t_query.get_mut(child) else {
                continue;
            };

			// Check if T already exists
			if let Some(mut t) = maybe_t {
				*t = parent_t.clone();
			} else if let Some(mut entity_command) = commands.get_entity(child) {
				entity_command.insert(parent_t.clone());
			}
		}
	}
}
