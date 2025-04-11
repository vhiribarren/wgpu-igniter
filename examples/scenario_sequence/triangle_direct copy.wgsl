struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct FragmentInput {
    @location(0) color: vec3<f32>,
    @builtin(position) position: vec4<f32>,
};


// [[group(0), binding(0)]]
// var<uniform> transform: mat4x4<f32>;

struct TransformUniform {
    m: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> transform: TransformUniform;

@vertex
fn vtx_main(vtx_in: VertexInput) -> FragmentInput {
    var out: FragmentInput;
    out.color = vtx_in.color;
    out.position = transform.m * vec4<f32>(vtx_in.position, 1.0);
    return out;
}

@fragment
fn frg_main(frg_in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(frg_in.color, 1.0);
}