use crate::prelude::*;
use bevy::{gltf::Gltf, render::once_cell::sync::OnceCell};
pub use bevy_asset_loader::prelude::*;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app
			.add_loading_state(
                LoadingState::new(GameState::LoadingAssets)
                    .continue_to_state(GameState::InGame)
            )
			.add_collection_to_loading_state::<_, GameAssets>(GameState::LoadingAssets)
            .add_system(setup.in_schedule(OnExit(GameState::LoadingAssets)));
    }
} 

pub static SHADOW_BUNDLE: OnceCell<(Handle<StandardMaterial>,Handle<Mesh>)> = OnceCell::new();

fn setup(
    mut commands: Commands,
	mut mesh_assets: ResMut<Assets<Mesh>>,
	mut material_assets: ResMut<Assets<StandardMaterial>>,
	assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
	let plane_mesh = mesh_assets.add(Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 }));

	let shadow_material = material_assets.add(StandardMaterial {
		base_color_texture: Some(assets.circle_texture.clone()),
		alpha_mode: AlphaMode::Mask(0.5),
        base_color: Color::rgb(0.6 * 0.9, 0.8 * 0.9, 0.2 * 0.9),
        unlit: true,
		..default()
	});

	SHADOW_BUNDLE.set((shadow_material,plane_mesh)).unwrap();

    // Setup spawnable collection
    let mut spawnable_collection = SpawnableCollection::default();
    
    let tree_scenes = &gltfs.get(&assets.tree_gltf).unwrap().scenes;
    for (_i,tree) in tree_scenes.iter().enumerate() {
        let spawnable = Spawnable {
            stype: SpawnableType::Tree,
            scene: tree.clone(),
            is_ingridient: false,
            spawn_weight: 1.5 / tree_scenes.len() as f32,
            size: 2.8
        };
        spawnable_collection.0.push(spawnable);
    }

    let bush_scenes = &gltfs.get(&assets.bush_gltf).unwrap().scenes;
    for (i,scene) in bush_scenes.iter().enumerate() {
        let spawnable = Spawnable {
            stype: SpawnableType::Bush,
            scene: scene.clone(),
            is_ingridient: false,
            spawn_weight: 1.0 / tree_scenes.len() as f32,
            size: match i {
                1 => 2.0,
                _ => 1.5
            }
        };
        spawnable_collection.0.push(spawnable);
    }

    let ingridient_scenes = &gltfs.get(&assets.ingridients_gltf).unwrap().scenes;
    for (_i,scene) in ingridient_scenes.iter().enumerate() {
        let spawnable = Spawnable {
            stype: SpawnableType::Mushroom,
            scene: scene.clone(),
            is_ingridient: true,
            spawn_weight: 0.5 / ingridient_scenes.len() as f32,
            size: 0.6
        };
        spawnable_collection.0.push(spawnable);
    }

    commands.insert_resource(spawnable_collection);
}

#[derive(Default,Resource,Debug)]
pub struct SpawnableCollection(pub Vec<Spawnable>);

#[derive(Clone, Debug)]
pub struct Spawnable {
    pub stype: SpawnableType,
    pub scene: Handle<Scene>,
    pub is_ingridient: bool,
    pub spawn_weight: f32,
    pub size: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum SpawnableType {
    Tree,
    Bush,
    Mushroom,
}


#[derive(AssetCollection, Resource)]
pub struct GameAssets {
	#[asset(path = "textures/dither.png")]
    pub dither_texture: Handle<Image>,
    #[asset(path = "textures/circle.png")]
    pub circle_texture: Handle<Image>,
    #[asset(path = "models/tree.gltf")]
    pub tree_gltf: Handle<Gltf>,
    #[asset(path = "models/bush.gltf")]
    pub bush_gltf: Handle<Gltf>,
    #[asset(path = "models/ingridients.gltf")]
    pub ingridients_gltf: Handle<Gltf>,
}
