use std::{cell::Cell, ffi::c_void, mem, rc::Rc};

use gl::{
    ARRAY_BUFFER, COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, DEPTH_TEST, STATIC_DRAW,
    types::{GLuint, GLvoid},
};
use glfw::{Context, Key, OpenGlProfileHint, WindowHint, WindowMode};
use nalgebra_glm::{Mat4, Vec2, Vec4, vec3};

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
    unsafe {
        gl::Enable(DEPTH_TEST);
    }

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_framebuffer_size_callback(|_window, width, height| {
        unsafe { gl::Viewport(0, 0, width, height) };
    });

    let mut triangle_vao: GLuint = 0;
    let mut triangle_vbo: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut triangle_vao);
        gl::GenBuffers(1, &mut triangle_vbo);

        gl::BindVertexArray(triangle_vao);

        #[rustfmt::skip]
        const TRIANGLE_VERTICES: [f32; 3 * 3] = [
            -0.5, -0.5,  0.0,
             0.0,  0.5,  0.0,
             0.5, -0.5,  0.0,
        ];

        gl::BindBuffer(ARRAY_BUFFER, triangle_vbo);
        gl::BufferData(
            ARRAY_BUFFER,
            mem::size_of_val(&TRIANGLE_VERTICES) as _,
            TRIANGLE_VERTICES.as_ptr() as *const GLvoid,
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

    let mut line_vao: GLuint = 0;
    let mut line_vbo: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut line_vao);
        gl::GenBuffers(1, &mut line_vbo);

        gl::BindVertexArray(line_vao);

        #[rustfmt::skip]
        const LINE_VERTICES: [f32; 6] = [
            0.0,  1.0, 0.1,
            0.0, -1.0, 0.1
        ];

        gl::BindBuffer(ARRAY_BUFFER, line_vbo);
        gl::BufferData(
            ARRAY_BUFFER,
            mem::size_of_val(&LINE_VERTICES) as _,
            LINE_VERTICES.as_ptr() as _,
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

    let transform = Rc::new(Cell::new(Vec2::zeros()));
    let last_down = Rc::new(Cell::new(None as Option<Vec2>));
    let last_pos = Rc::new(Cell::new(Vec2::zeros()));

    let last_down_clone = Rc::clone(&last_down);
    let last_pos_clone = Rc::clone(&last_pos);
    window.set_mouse_button_polling(true);
    window.set_mouse_button_callback(
        move |_window, _mouse_button, action, _modifiers| match action {
            glfw::Action::Release => last_down_clone.set(None),
            glfw::Action::Press => last_down_clone.set(Some(last_pos_clone.get())),
            glfw::Action::Repeat => (),
        },
    );

    let transform_clone = Rc::clone(&transform);
    let last_pos_clone = Rc::clone(&last_pos);
    let last_down_clone = Rc::clone(&last_down);
    window.set_cursor_pos_polling(true);
    window.set_cursor_pos_callback(move |_window, xpos, ypos| {
        last_pos_clone.set(Vec2::from_vec(vec![xpos as f32, ypos as f32]));

        if let Some(v) = last_down_clone.get() {
            let prev_transform = transform_clone.get();
            transform_clone.set(
                prev_transform + ((last_pos_clone.get() - v).component_mul(&Vec2::new(1., -1.))),
            );
            last_down_clone.set(Some(last_pos_clone.get()));

            println!("{}", transform_clone.get());
        }
    });

    window.set_key_polling(true);
    window.set_key_callback(|window, key, _scancode, _action, _modifiers| match key {
        Key::Escape => {
            window.set_should_close(true);
        }
        _ => (),
    });

    while !window.should_close() {
        unsafe {
            let (width, height) = window.get_size();
            let (width, height) = (width as f32, height as f32);
            let cam_transform = &nalgebra_glm::scale(
                &Mat4::identity(),
                &vec3(40. / (width as f32), 40. / (height as f32), 1.),
            );

            gl::ClearColor(0.3, 0.3, 0.3, 1.);
            gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

            triangle_shader.use_shader();
            gl::BindVertexArray(triangle_vao);
            triangle_shader.set_vec4("rgba", &Vec4::new(0.3, 0.9, 05., 1.));
            triangle_shader.set_mat4("cam_transform", &cam_transform);
            triangle_shader.set_mat4(
                "transform",
                &nalgebra_glm::translate(
                    &Mat4::identity(),
                    &vec3(
                        transform.get().x / width * 2.,
                        transform.get().y / height * 2.,
                        0.0,
                    ),
                ),
            );
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            gl::BindVertexArray(line_vao);
            triangle_shader.set_vec4("rgba", &Vec4::new(0.3, 0.3, 0.7, 1.0));
            triangle_shader.set_mat4("cam_transform", &Mat4::identity());
            triangle_shader.set_mat4("transform", &Mat4::identity());
            gl::LineWidth(2.0);
            gl::DrawArrays(gl::LINES, 0, 2);
        };

        window.swap_buffers();
        glfw.poll_events();
    }
}
