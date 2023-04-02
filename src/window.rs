use bevy::prelude::*;

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(cursor_grab_system)
			.init_resource::<CursorMode>();
	}
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum CursorMode {
	Locked,
	#[default]
	Unlocked
}

impl CursorMode {
	pub fn locked(&self) -> bool {
		*self == CursorMode::Locked
	}
}

pub fn get_window_plugin() -> bevy::prelude::WindowPlugin {
	bevy::prelude::WindowPlugin {
		primary_window: 
			Some(Window {
				title: "Project Concoction".to_string(),
				present_mode: bevy::window::PresentMode::Immediate,
				..default()
		}),
		..default()
	}
}

pub fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
	mut cursor_mode: ResMut<CursorMode>,
	#[cfg(debug_assertions)]
    mut gui: Query<&mut bevy_inspector_egui::bevy_egui::EguiContext>,
) {
    let mut window = windows.single_mut();

	#[cfg(debug_assertions)]
	{
		let mut gui = gui.single_mut();
		if mouse.just_pressed(MouseButton::Left) && !gui.get_mut().is_pointer_over_area() {
			window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
			window.cursor.visible = false;
			*cursor_mode = CursorMode::Locked;
		}
	}
    
	#[cfg(not(debug_assertions))]
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
		*cursor_mode = CursorMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) || key.just_pressed(KeyCode::LAlt) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor.visible = true;
		*cursor_mode = CursorMode::Unlocked;
    }
}