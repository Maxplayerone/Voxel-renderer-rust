struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct LightUniform{
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(1) @binding(0)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.normal = model.normal;
    out.world_pos = model.position;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //ambient
    let ambient_value = 0.1;
    let ambient = light.color * ambient_value;
    //diffuse 
    let light_dir = normalize(light.position - in.world_pos);
    let diff = max(dot(in.normal, light_dir), 0.0);
    let diffuse = light.color * diff;
    //specular
    let view_dir = normalize(camera.view_pos.xyz - in.world_pos);
    let reflect_dir = reflect(-light_dir, in.normal);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular = specular_strength * light.color;
    //adding everything together
    let color = in.color * (ambient + diffuse + specular);
    return vec4<f32>(color, 1.0);
}