#import bevy_pbr::mesh_view_bindings

struct Foliage {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: Foliage;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
	@builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let directional_light = lights.directional_lights[0];
    let ambient_light = lights.ambient_color;
    let screen_uv = position.xy / view.viewport.zw;

    let lighting = (dot(directional_light.direction_to_light,world_normal) + 1.0) / 2.0;

    let center = view.viewport.zw / 2.0;
    let distance = 1.5 - distance(position.xy,center) / 100.0;

	let uv = position.xy / 16.0;
    let alpha = textureSample(base_color_texture, base_color_sampler, uv).x;

    // if alpha < distance {
        // discard;
    // }

    var color = material.color.xyz;
    if lighting < 0.5 {
        color /= 2.0;
    }
    if lighting < 0.75 {
        color /= 2.0;
    }
    if lighting < 0.25 {
        color /= 2.0;
    }

    return vec4<f32>(color,1.0);
}