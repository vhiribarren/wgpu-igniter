struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @builtin(instance_index) index: u32,
};

struct FragmentInput {
    @location(0) normal: vec3<f32>,
    @location(1) space_position: vec3<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

const SUN_LIGHT_DIRECTION = vec3<f32>(0., -1., 1.);
const SUN_LIGHT_COLOR = vec3<f32>(1., 1., 1.);
const AMBIANT_COLOR =  vec3<f32>(0.1);

const SPOT_LIGHT_POSITION = vec3<f32>(0., 5., -15.);
const SPOT_LIGHT_COLOR = vec3<f32>(1., 0., 1.);


@group(0) @binding(0)
var<uniform> camera_mat: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> camera_pos: vec3<f32>;
@group(1) @binding(0)
var<storage, read> transforms: array<mat4x4<f32>>;
@group(1) @binding(1)
var<storage, read> normal_mat: array<mat3x3<f32>>;

@vertex
fn vtx_main(vtx_in: VertexInput, inst_in: InstanceInput) -> FragmentInput {
    var out: FragmentInput;
    var space_position = transforms[inst_in.index] * vec4<f32>(vtx_in.position, 1.0);
    out.normal = normal_mat[inst_in.index] * vtx_in.normal;
    out.space_position = space_position.xyz;
    out.clip_position = camera_mat * space_position;
    return out;
}

@fragment
fn frg_main(frg_in: FragmentInput) -> @location(0) vec4<f32> {
    //let light_coeff = clamp(0.0, 1.0, dot(normalize(frg_in.normal), -normalize(SUN_LIGHT_DIRECTION)));
    //let light_value = AMBIANT_COLOR + light_coeff * SUN_LIGHT_COLOR;

    let spot_light_dir = normalize(SPOT_LIGHT_POSITION - frg_in.space_position);
    let diffuse_strength = max(dot(frg_in.normal, spot_light_dir), 0.0);
    let diffuse_color = SPOT_LIGHT_COLOR * diffuse_strength;

    let view_dir = normalize(camera_pos.xyz - frg_in.space_position);
    let reflect_dir = reflect(-spot_light_dir, frg_in.normal);
    //let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let half_dir = normalize(view_dir + spot_light_dir);
    let specular_strength = pow(max(dot(frg_in.normal, half_dir), 0.0), 32.0);


    let specular_color = specular_strength * SPOT_LIGHT_COLOR;

    let light_value = AMBIANT_COLOR + diffuse_color + specular_color;
    //let light_value = AMBIANT_COLOR + specular_color;


    return vec4<f32>(light_value, 1.0);
}