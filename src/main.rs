use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

fn main() {
    App::default()
        .add_plugins(
            DefaultPlugins
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
        )
        .run();
}
