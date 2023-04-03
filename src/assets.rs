use crate::prelude::*;
pub use bevy_asset_loader::prelude::*;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app
			.add_loading_state(
            	LoadingState::new(GameState::Loading)
            	    .continue_to_state(GameState::InGame)
        	)
			.add_collection_to_loading_state::<_, GameAssets>(GameState::Loading);
    }
} 

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
	#[asset(path = "textures/dither.png")]
    pub dither_texture: Handle<Image>,
    #[asset(path = "models/tree.gltf#Scene0")]
    pub tree_model: Handle<Scene>,
}