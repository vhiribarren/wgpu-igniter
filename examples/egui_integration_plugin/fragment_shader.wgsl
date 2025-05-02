/*
MIT License

Copyright (c) 2025 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

const canvas: array<vec2<f32>, 3> = array(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0)
);

@group(0) @binding(0)
var<uniform> elapsed_time: f32;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct FragmentInput {
    @builtin(position) screen_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    let vtx_coords = canvas[input.vertex_index];
    var output: VertexOutput;
    output.clip_position = vec4<f32>(vtx_coords, 1.0, 1.0);
    output.uv = (vtx_coords + vec2<f32>(1.0))/2.0;
    output.uv.y = 1. - output.uv.y;
    return output;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var col = 0.5 + 0.5*cos(elapsed_time + in.uv.xyx + vec3(0.0,2.0,4.0));
    return vec4<f32>(col, 1.0);
}