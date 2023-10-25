use crate::prelude::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(Startup, add_loading_text)
			.add_systems(OnExit(GameState::LoadingAssets), disable_loading_text);
	}
}

#[derive(Component)]
struct LoadingText;

fn add_loading_text(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	commands.spawn((
		LoadingText,
		Name::new("Loading Text"),
		TextBundle::from_section(
			"Loading...", 
			TextStyle { 
				font: asset_server.load("fonts/FiraSans-Medium.ttf"), 
				font_size: 80.0, 
				color: Color::WHITE, 
			}
		)
		.with_text_alignment(TextAlignment::Center)
		.with_style( Style { 
			position_type: PositionType::Absolute,
			..default()
		}),

	));
}

fn disable_loading_text(
	mut loading_text: Query<&mut Visibility, With<LoadingText>>
) {
	let mut vis = loading_text.single_mut();
	//*vis = Visibility::Hidden;
}