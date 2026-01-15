use std::{ffi::c_void, mem};

use gl::{
    ARRAY_BUFFER, COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, STATIC_DRAW, types::{GLuint, GLvoid}
};
use glfw::{Context, OpenGlProfileHint, WindowHint, WindowMode};
use nalgebra_glm::{Mat4, Vec4};

use crate::shader::Shader;

mod shader;

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    let (mut window, _events) = glfw
        .create_window(800, 600, "2D Linear Transformation", WindowMode::Windowed)
        .unwrap();

    gl::load_with(|s| window.get_proc_address(s).unwrap() as *const c_void);

    window.make_current();
    window.set_framebuffer_size_polling(true);

    #[rustfmt::skip]
    let triangle_vertices: [f32; 3 * 3] = [
        -0.5, -0.5,  0.0,
         0.0,  0.5,  0.0,
         0.5, -0.5,  0.0,
    ];

    let mut triangle_vao: GLuint = 0;
    let mut triangle_vbo: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut triangle_vao);
        gl::GenBuffers(1, &mut triangle_vbo);

        gl::BindVertexArray(triangle_vao);

        gl::BindBuffer(ARRAY_BUFFER, triangle_vbo);
        gl::BufferData(
            ARRAY_BUFFER,
            mem::size_of_val(&triangle_vertices) as _,
            triangle_vertices.as_ptr() as *const GLvoid,
            STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * size_of::<f32>() as i32,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
    }

    let triangle_shader = Shader::new(
        "shaders/triangle_solid.v.glsl",
        "shaders/triangle_solid.f.glsl",
    )
    .unwrap();

    while !window.should_close() {
        triangle_shader.use_shader();
        triangle_shader.set_vec4("rgba", &Vec4::new(0.3, 0.4, 05., 1.));
        triangle_shader.set_mat4("transformation", &Mat4::identity());
        unsafe {
            gl::BindVertexArray(triangle_vao);
            gl::ClearColor(0.3, 0.3, 0.3, 1.);
            gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        };

        window.swap_buffers();
        glfw.poll_events();
    }
}
