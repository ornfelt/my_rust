use std::{
    ffi::{CStr, CString},
    mem, ptr, str,
    time::{SystemTime, UNIX_EPOCH},
};

use gl::types::*;
use glfw::{Action, Context, Key};

const WIN_WIDTH:  u32 = 800;
const WIN_HEIGHT: u32 = 600;

// ────────────────────── shader sources ──────────────────────
const VERT_SRC: &str = r#"
#version 330 core
layout(location = 0) in vec2 position;
uniform vec2 offset;
void main() { gl_Position = vec4(position + offset, 0.0, 1.0); }
"#;

const FRAG_SRC: &str = r#"
#version 330 core
out vec4 color;
uniform vec4 rectColor;
void main() { color = rectColor; }
"#;

const OBST_VERT_SRC: &str = VERT_SRC;
const OBST_FRAG_SRC: &str = r#"
#version 330 core
out vec4 color;
void main() { color = vec4(1.0,0.0,0.0,1.0); }
"#;

// ────────────────────── helpers ──────────────────────
unsafe fn compile_shader(src: &str, kind: GLenum) -> GLuint {
    let shader = gl::CreateShader(kind);
    let cstr   = CString::new(src).unwrap();
    gl::ShaderSource(shader, 1, &cstr.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    let mut ok = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut ok);
    if ok == gl::FALSE as GLint {
        let mut log = vec![0i8; 512];
        gl::GetShaderInfoLog(
            shader,
            512,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut _,
        );
        panic!(
            "shader error:\n{}",
            str::from_utf8(CStr::from_ptr(log.as_ptr()).to_bytes()).unwrap()
        );
    }
    shader
}

unsafe fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    let prog = gl::CreateProgram();
    gl::AttachShader(prog, vs);
    gl::AttachShader(prog, fs);
    gl::LinkProgram(prog);

    let mut ok = gl::FALSE as GLint;
    gl::GetProgramiv(prog, gl::LINK_STATUS, &mut ok);
    if ok == gl::FALSE as GLint {
        let mut log = vec![0i8; 512];
        gl::GetProgramInfoLog(
            prog,
            512,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut _,
        );
        panic!(
            "link error:\n{}",
            str::from_utf8(CStr::from_ptr(log.as_ptr()).to_bytes()).unwrap()
        );
    }
    prog
}

fn collide(rx: f32, ry: f32, tx: f32, ty: f32, tri_size: f32) -> bool {
    const HALF: f32 = 0.1;
    rx + HALF > tx - tri_size
        && rx - HALF < tx + tri_size
        && ry + HALF > ty - tri_size
        && ry - HALF < ty + tri_size
}

// ────────────────────── main ──────────────────────
fn main() {
    // initialise glfw
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(WIN_WIDTH, WIN_HEIGHT, "glfw + OpenGL (Rust)", glfw::WindowMode::Windowed)
        .expect("Failed to open GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    // load GL symbols
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    // ─── compile + link shaders ───
    let (vs,  fs)  = unsafe { (compile_shader(VERT_SRC,  gl::VERTEX_SHADER),
                               compile_shader(FRAG_SRC,  gl::FRAGMENT_SHADER)) };
    let (ovs, ofs) = unsafe { (compile_shader(OBST_VERT_SRC, gl::VERTEX_SHADER),
                               compile_shader(OBST_FRAG_SRC, gl::FRAGMENT_SHADER)) };

    let prog  = unsafe { link_program(vs,  fs)  };
    let oprog = unsafe { link_program(ovs, ofs) };

    unsafe { gl::DeleteShader(vs); gl::DeleteShader(fs);
             gl::DeleteShader(ovs); gl::DeleteShader(ofs); }

    // ─── rectangle data ───
    let verts:   [f32; 8] = [-0.1,-0.1,  0.1,-0.1,  0.1,0.1,  -0.1,0.1];
    let indices: [u32; 6] = [0,1,2, 2,3,0];

    let mut vao = 0; let mut vbo = 0; let mut ebo = 0;
    unsafe {
        gl::GenVertexArrays(1,&mut vao);
        gl::GenBuffers(1,&mut vbo);
        gl::GenBuffers(1,&mut ebo);
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
            (verts.len()*mem::size_of::<GLfloat>()) as isize,
            verts.as_ptr() as *const _, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
            (indices.len()*mem::size_of::<GLuint>()) as isize,
            indices.as_ptr() as *const _, gl::STATIC_DRAW);

        gl::VertexAttribPointer(0,2,gl::FLOAT,gl::FALSE,0,ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindVertexArray(0);
    }

    // ─── triangle data ───
    let tri_verts: [f32; 6] = [0.0,0.1,  -0.1,-0.1,  0.1,-0.1];
    let (mut tvao, mut tvbo) = (0,0);
    unsafe {
        gl::GenVertexArrays(1,&mut tvao);
        gl::GenBuffers(1,&mut tvbo);
        gl::BindVertexArray(tvao);
        gl::BindBuffer(gl::ARRAY_BUFFER, tvbo);
        gl::BufferData(gl::ARRAY_BUFFER,
            (tri_verts.len()*mem::size_of::<GLfloat>()) as isize,
            tri_verts.as_ptr() as *const _, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0,2,gl::FLOAT,gl::FALSE,0,ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindVertexArray(0);
    }

    // ─── gameplay state ───
    let mut x_off = 0.0f32; let mut y_off = 0.0f32;
    const MOVE: f32 = 0.01;
    let mut tri_x = rand::random::<f32>()*2.0 - 1.0;
    let mut tri_y = rand::random::<f32>()*2.0 - 1.0;
    const TRI_MOVE: f32 = 0.005;

    // ─── main loop ───
    while !window.should_close() {
        // --- input by polling keys (simplest GLFW path)
        if window.get_key(Key::Escape) == Action::Press { window.set_should_close(true); }
        if window.get_key(Key::W) == Action::Press { y_off += MOVE; }
        if window.get_key(Key::S) == Action::Press { y_off -= MOVE; }
        if window.get_key(Key::A) == Action::Press { x_off -= MOVE; }
        if window.get_key(Key::D) == Action::Press { x_off += MOVE; }

        // collision check
        let colliding = collide(x_off,y_off,tri_x,tri_y,0.1);
        let rect_col: [f32;4] = if colliding { [1.0,0.0,0.0,1.0] } else { [0.0,1.0,0.0,1.0] };

        // simple push-back when colliding
        if colliding {
            if window.get_key(Key::W) == Action::Press { y_off -= MOVE; }
            if window.get_key(Key::S) == Action::Press { y_off += MOVE; }
            if window.get_key(Key::A) == Action::Press { x_off += MOVE; }
            if window.get_key(Key::D) == Action::Press { x_off -= MOVE; }
        }

        // triangle wander
        tri_x += (rand::random::<f32>()*2.0 - 1.0)*TRI_MOVE;
        tri_y += (rand::random::<f32>()*2.0 - 1.0)*TRI_MOVE;
        if tri_x.abs() > 1.0 { tri_x = 0.0; }
        if tri_y.abs() > 1.0 { tri_y = 0.0; }

        // --- draw
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(prog);
            gl::Uniform2f(gl::GetUniformLocation(prog, b"offset\0".as_ptr() as _), x_off, y_off);
            gl::Uniform4fv(gl::GetUniformLocation(prog, b"rectColor\0".as_ptr() as _), 1, rect_col.as_ptr());
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

            gl::UseProgram(oprog);
            gl::Uniform2f(gl::GetUniformLocation(oprog, b"offset\0".as_ptr() as _), tri_x, tri_y);
            gl::BindVertexArray(tvao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers();
        glfw.poll_events();

        // handle resize events (optional)
        for (_,e) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(w,h) = e {
                unsafe { gl::Viewport(0,0,w,h); }
            }
        }
    }

    // ─── cleanup ───
    unsafe {
        gl::DeleteVertexArrays(1,&vao);
        gl::DeleteBuffers(1,&vbo);
        gl::DeleteBuffers(1,&ebo);
        gl::DeleteVertexArrays(1,&tvao);
        gl::DeleteBuffers(1,&tvbo);
        gl::DeleteProgram(prog);
        gl::DeleteProgram(oprog);
    }
}
