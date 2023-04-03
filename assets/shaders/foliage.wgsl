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
	// let uvee = position.xy / view; //vec2<f32>(view.0, view.1);
    return material.color; // textureSample(base_color_texture, base_color_sampler, uvee);
}