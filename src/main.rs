use std::{cell::Cell, f32::consts::PI, ffi::c_void, mem, rc::Rc};

use gl::{
    ARRAY_BUFFER, COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, DEPTH_TEST, MULTISAMPLE, STATIC_DRAW,
    types::{GLuint, GLvoid},
};
use glfw::{Context, Key, OpenGlProfileHint, WindowHint, WindowMode};
use nalgebra_glm::{Mat4, Vec2, Vec4, vec2, vec3, vec4};

use crate::shader::Shader;

mod shader;

#[derive(Clone, Copy, PartialEq)]
enum CursorState {
    None,
    XDrag(Vec2),
    YDrag(Vec2),
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::Samples(Some(4)));

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
            -1.0, -1.0,  0.0,
             0.0,  1.0,  0.0,
             1.0, -1.0,  0.0,
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
            0.0,  0.0, 0.1
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
    let last_pos = Rc::new(Cell::new(Vec2::zeros()));
    let cursor_state = Rc::new(Cell::new(CursorState::None));

    window.set_mouse_button_polling(true);
    window.set_mouse_button_callback({
        let last_pos = Rc::clone(&last_pos);
        let cursor_state = Rc::clone(&cursor_state);
        let transform = Rc::clone(&transform);

        move |window, _mouse_button, action, _modifiers| match action {
            glfw::Action::Release => cursor_state.set(CursorState::None),
            glfw::Action::Press => {
                let last_pos = last_pos.get();
                let t = transform.get();
                let (width, height) = window.get_size();

                let click_pos = (last_pos - vec2(width as f32 / 2., height as f32 / 2.))
                    .component_mul(&vec2(1., -1.));

                if click_pos.x > t.x - 20.
                    && click_pos.x < t.x + 20.
                    && click_pos.y > t.y - 20.
                    && click_pos.y < t.y + 20.
                {
                    cursor_state.set(CursorState::XDrag(vec2(last_pos.x, last_pos.y)));
                }
            }
            glfw::Action::Repeat => (),
        }
    });

    window.set_cursor_pos_polling(true);
    window.set_cursor_pos_callback({
        let transform = Rc::clone(&transform);
        let last_pos = Rc::clone(&last_pos);
        let cursor_state = Rc::clone(&cursor_state);

        move |_window, xpos, ypos| {
            last_pos.set(Vec2::from_vec(vec![xpos as f32, ypos as f32]));

            match cursor_state.get() {
                CursorState::None => (),
                CursorState::XDrag(matrix) => {
                    let prev_transform = transform.get();
                    transform.set(
                        prev_transform + ((last_pos.get() - matrix).component_mul(&vec2(1., -1.))),
                    );
                    cursor_state.set(CursorState::XDrag(last_pos.get()));
                }
                CursorState::YDrag(matrix) => (),
            }
        }
    });

    window.set_key_polling(true);
    window.set_key_callback(|window, key, _scancode, _action, _modifiers| match key {
        Key::Escape => {
            window.set_should_close(true);
        }
        _ => (),
    });

    unsafe {
        gl::Enable(MULTISAMPLE);
    }

    while !window.should_close() {
        unsafe {
            let (width, height) = window.get_size();
            let (width, height) = (width as f32, height as f32);
            let cam_transform = &nalgebra_glm::scale(
                &Mat4::identity(),
                &vec3(1. / (width as f32), 1. / (height as f32), 1.),
            );

            gl::ClearColor(0.3, 0.3, 0.3, 1.);
            gl::Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

            // Vector head
            triangle_shader.use_shader();
            gl::BindVertexArray(triangle_vao);
            triangle_shader.set_vec4("rgba", &Vec4::new(0.1, 0.8, 0.1, 1.));
            let vector_head_translate = nalgebra_glm::translate(
                &Mat4::identity(),
                &vec3(
                    transform.get().x / width * 2.,
                    transform.get().y / height * 2.,
                    0.0,
                ),
            );
            let t = vector_head_translate.column(3);
            let mut vector_head_rot_angle = ((t.x * width) / (t.y * height)).atan();
            if t.y < 0. {
                vector_head_rot_angle += PI;
            }
            if vector_head_rot_angle.is_nan() {
                vector_head_rot_angle = 0.;
            }
            let vector_head_rot =
                nalgebra_glm::rotate(&Mat4::identity(), vector_head_rot_angle, &vec3(0., 0., -1.));
            let vector_head_scale = nalgebra_glm::scale(&Mat4::identity(), &vec3(20., 20., 1.));
            let vector_head_transform =
                &vector_head_translate * cam_transform * &vector_head_rot * &vector_head_scale;

            triangle_shader.set_mat4("transform", &vector_head_transform);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            // Vector line
            gl::BindVertexArray(line_vao);
            triangle_shader.set_vec4("rgba", &vec4(0.1, 0.8, 0.1, 1.0));
            #[rustfmt::skip]
            let vector_line_scale = Mat4::new(
                1., t.x * width,  0., 0.,
                0., t.y * height, 0., 0.,
                0., 0.,           1., 0.,
                0., 0.,           0., 1.
            );
            let vector_line_transform = cam_transform * &vector_line_scale;
            triangle_shader.set_mat4("transform", &vector_line_transform);
            gl::LineWidth(4.0);
            gl::DrawArrays(gl::LINES, 0, 2);

            // y Axis
            gl::BindVertexArray(line_vao);
            triangle_shader.set_vec4("rgba", &Vec4::new(0.3, 0.3, 0.7, 1.0));
            triangle_shader.set_mat4("transform", &Mat4::identity());
            gl::LineWidth(2.0);
            gl::DrawArrays(gl::LINES, 0, 2);

            // x Axis
            triangle_shader.set_vec4("rgba", &Vec4::new(0.3, 0.3, 0.7, 1.0));
            triangle_shader.set_mat4(
                "transform",
                &nalgebra_glm::rotate_z(&Mat4::identity(), PI / 2.),
            );
            gl::LineWidth(2.0);
            gl::DrawArrays(gl::LINES, 0, 2);
        };

        window.swap_buffers();
        glfw.poll_events();
    }
}
