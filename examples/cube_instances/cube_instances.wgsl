struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @builtin(instance_index) index: u32,
    @location(2) offset: vec3<f32>,
};

struct FragmentInput {
    @location(0) normal: vec3<f32>,
    @builtin(position) position: vec4<f32>,
};

const LIGHT_DIRECTION = vec3<f32>(0., -1., 1.);
const LIGHT_COLOR = vec3<f32>(1., 1., 1.);
const AMBIANT_COLOR =  vec3<f32>(0.2);

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;
// @group(1) @binding(0)
// var<uniform> transform: mat4x4<f32>;
@group(1) @binding(1)
var<storage, read> normal_mat: array<mat3x3<f32>>;

@vertex
fn vtx_main(vtx_in: VertexInput, inst_in: InstanceInput) -> FragmentInput {
    var out: FragmentInput;
    out.normal = normal_mat[inst_in.index] * vtx_in.normal;
    out.position = camera * vec4<f32>(vtx_in.position + inst_in.offset, 1.0);
    return out;
}

@fragment
fn frg_main(frg_in: FragmentInput) -> @location(0) vec4<f32> {
    let light_coeff = clamp(0.0, 1.0, dot(normalize(frg_in.normal), -normalize(LIGHT_DIRECTION)));
    let light_value = AMBIANT_COLOR + light_coeff * LIGHT_COLOR;
    return vec4<f32>(light_value, 1.0);
}