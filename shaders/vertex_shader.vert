#version 140

in vec3 position;
in vec2 tex_coords;

out vec3 v_position;
out vec2 v_tex_coords;

uniform mat4 scale_matrix;
uniform mat4 pitch_matrix;
uniform mat4 roll_matrix;
uniform mat4 yaw_matrix;

void main() {
    v_tex_coords = tex_coords;
    mat4 matrix = pitch_matrix * roll_matrix * yaw_matrix * scale_matrix;
    gl_Position = matrix * vec4(position, 1.0);
}