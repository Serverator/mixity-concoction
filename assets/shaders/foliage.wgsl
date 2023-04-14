#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
//#import bevy_pbr::mesh_functions
#import bevy_pbr::utils
//#import bevy_pbr::shadows

struct Foliage {
    color: vec4<f32>,
    sss: u32,
};

fn remap(val: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
    return low2 + (val - low1) * (high2 - low2) / (high1 - low1);
}

@group(1) @binding(0)
var<uniform> material: Foliage;
// @group(1) @binding(1)
// var base_color_texture: texture_2d<f32>;
// @group(1) @binding(2)
// var base_color_sampler: sampler;



struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
// #ifdef VERTEX_UVS
//     @location(2) uv: vec2<f32>,
// #endif
// #ifdef VERTEX_TANGENTS
//     @location(3) tangent: vec4<f32>,
// #endif
// #ifdef VERTEX_COLORS
//     @location(4) color: vec4<f32>,
// #endif
// #ifdef SKINNED
//     @location(5) joint_indices: vec4<u32>,
//     @location(6) joint_weights: vec4<f32>,
// #endif
};

struct VertexOutput {
    //@builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
// #ifdef VERTEX_UVS
//     @location(2) uv: vec2<f32>,
// #endif
// #ifdef VERTEX_TANGENTS
//     @location(3) world_tangent: vec4<f32>,
// #endif
// #ifdef VERTEX_COLORS
//     @location(4) color: vec4<f32>,
// #endif
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.world_normal = normalize(
        mat3x3<f32>(
            mesh.inverse_transpose_model[0].xyz,
            mesh.inverse_transpose_model[1].xyz,
            mesh.inverse_transpose_model[2].xyz
        ) * in.normal
    );
    out.world_position = mesh.model * vec4<f32>(in.position, 1.0);

    // #ifdef SKINNED
    // var model = skin_model(in.joint_indices, in.joint_weights);
    // out.world_normal = skin_normals(model, in.normal);
    // #else
    // var model = mesh.model;
    // out.world_normal = mesh_normal_local_to_world(in.normal);
    // #endif
    // out.world_position = mesh_position_local_to_world(model, vec4<f32>(in.position, 1.0));
    // #ifdef VERTEX_UVS
    //     out.uv = in.uv;
    // #endif
    // #ifdef VERTEX_TANGENTS
    //     out.world_tangent = mesh_tangent_local_to_world(model, in.tangent);
    // #endif
    // #ifdef VERTEX_COLORS
    //     out.color = in.color;
    // #endif

    return out;
}

@fragment
fn fragment(
	@builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
    //@location(0) world_position: vec4<f32>,
    //@location(1) world_normal: vec3<f32>,
) -> @location(0) vec4<f32> {

    //let ambient_light = lights.ambient_color;
    //let screen_uv = position.xy / view.viewport.zw;

    //let center = view.viewport.zw / 2.0;
    //let distance = 1.5 - distance(position.xy,center) / 100.0;

	//let uv = position.xy / 16.0;
    //let alpha = textureSample(base_color_texture, base_color_sampler, uv).x;

    //let camera_direction = (vec4<f32>(0.0,0.0,-1.0,1.0) * view.inverse_view).xyz;

    let pixel_direction = normalize(world_position.xyz - view.world_position);

    let view_z = dot(vec4<f32>(view.inverse_view[0].z,
        view.inverse_view[1].z,
        view.inverse_view[2].z,
        view.inverse_view[3].z
    ), world_position);

    var shadow: f32 = 0.0;

    // let n_directional_lights = lights.n_directional_lights;
    // for (var i: u32 = 0u; i < n_directional_lights; i = i + 1u) {
    //     shadow = 1.0-fetch_directional_shadow(i, world_position, world_normal, view_z);
    // }

    let light_direction = lights.directional_lights[0].direction_to_light;

    let lighting = (dot(light_direction,world_normal) + 1.0) / 2.0;

    if shadow < 0.5 {
        shadow = 0.0;
    } else {
        shadow = 1.0;
    }

    if lighting < 0.75 {
        shadow = max(0.5,shadow);

        if lighting < 0.5 {
            shadow = 1.0;
        }
    }

    var color = material.color.xyz;

    var fake_sss = 0.0;

    if material.sss == 1u {
        let lighting_dir = (dot(pixel_direction, light_direction) + 1.0 / 2.0);
        fake_sss = max(remap((dot(world_normal,pixel_direction) + 1.0) / 2.0, 0.2, 1.0, 0.0, 1.0),0.0) * lighting_dir;
    }

    if (lighting > 0.93 || fake_sss > 0.15) && shadow == 0.0 {
        color = mix(color,vec3<f32>(1.0,1.0,0.5),0.07);
    }

    if fake_sss > 0.15 {
        shadow = max(shadow - 0.5, 0.0);
    }

    return vec4<f32>(mix(color, color * 0.4, shadow),1.0);
}