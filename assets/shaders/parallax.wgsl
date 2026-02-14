#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: Material;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var back_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var back_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var middle_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var middle_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var front_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var front_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var light_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var light_sampler: sampler;

struct Material {
    scale: vec2<f32>,
    offset: vec2<f32>,
    camera: vec2<f32>,
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let flip_y = vec2<f32>(1, -1);

    let p = mesh.world_position.xy + flip_y * material.offset;
    let s = vec2<f32>(textureDimensions(back_texture)) * flip_y * material.scale;

    let uv_back = (p - material.camera * vec2<f32>(0.4, 0)) / s;
    let uv_light = (p - material.camera * vec2<f32>(0.6, 0)) / s;
    let uv_mid = (p - material.camera * vec2<f32>(0.3, 0)) / s;
    let uv_front = (p - material.camera * vec2<f32>(0.2, 0)) / s;

    if uv_back.y > 1.0 || uv_back.y < 0.0 {
        discard;
    }

    var c = textureSample(front_texture, front_sampler, uv_front);
    if c.a > 0.001 {
        return c;
    }

    c = textureSample(middle_texture, middle_sampler, uv_mid);
    if c.a > 0.001 {
        return c;
    }

    c = textureSample(light_texture, light_sampler, uv_light);
    if c.a > 0.001 {
        return c;
    }

    c = textureSample(back_texture, back_sampler, uv_back);
    if c.a > 0.001 {
        return c;
    }

    discard;
}
