#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use rodio::{OutputStream, Source};
use std::io::Cursor;
use glium::{uniform, Surface};

mod obj;

const VERTEX_SHADER_SRC: &str = include_str!("../shaders/vertex_shader.vert");
const FRAGMENT_SHADER_SRC: &str = include_str!("../shaders/fragment_shader.frag");
const MAXWELL_OBJ: &str = include_str!("../models/maxwell.obj");


fn main() {
    let event_loop = glium::winit::event_loop::EventLoop::builder().build().expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
    .with_title("Maxwell")
    .build(&event_loop);


    // Load maxwell model
    let model: obj::ObjData = obj::parse_obj(MAXWELL_OBJ, Some("dingus"));

    let positions = glium::VertexBuffer::new(&display, &model.vertices).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &model.indices).unwrap();

    let model: obj::ObjData = obj::parse_obj(MAXWELL_OBJ, Some("whiskers"));

    let whiskers_positions = glium::VertexBuffer::new(&display, &model.vertices).unwrap();
    let whiskers_indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &model.indices).unwrap();

    // Load maxwell texture
    let image = image::load(std::io::Cursor::new(&include_bytes!("../textures/maxwell.jpg")),
                        image::ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let image = image::load(std::io::Cursor::new(&include_bytes!("../textures/whiskers.png")),
    image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let whiskers_texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    // Set up audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let source = rodio::Decoder::new(Cursor::new(include_bytes!("../stockmarket.mp3"))).unwrap();

    let _ = stream_handle.play_raw(source.repeat_infinite().convert_samples());

    // Set up variables
    let mut pitch: f32 = 0.5;
    let mut roll: f32 = 0.0;
    let mut yaw: f32 = 0.0;

    let start = std::time::Instant::now();
    
    let _ = event_loop.run(move |event, window_target| {
        match event {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
                glium::winit::event::WindowEvent::RedrawRequested => {
                    let elasped = std::time::Instant::now() - start;
                    let elasped_ms = elasped.as_millis();

                    let mut target = display.draw();
                    // Clear screen
                    target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

                    if elasped_ms % 3585 == elasped_ms % 7170 {
                        // Dancy
                        let temp = elasped_ms as f32 / 150.0;
                        pitch = temp.sin() / 2.5 + 0.25;
                        roll = -temp.sin() / 10.0;
                        yaw = 2.7;
                    } else {
                        // Spinny
                        roll = 0.0;
                        pitch = 0.5;
                        yaw = elasped_ms as f32 / 500.0 * -1.0;
                    }

                    // Precompute values, this should be faster than calling the functions repeatidly
                    // but this is done each frame to satisfy changes
                    let pitch_sin = pitch.sin();
                    let pitch_cos = pitch.cos();

                    let roll_sin = roll.sin();
                    let roll_cos = roll.cos();
                    
                    let yaw_sin = yaw.sin();
                    let yaw_cos = yaw.cos();

                    let pitch_matrix = [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, pitch_cos, -pitch_sin, 0.0],
                        [0.0, pitch_sin, pitch_cos, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32],
                    ];

                    let roll_matrix = [
                        [roll_cos, -roll_sin, 0.0, 0.0],
                        [roll_sin, roll_cos, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32],
                    ];
                    
                    let yaw_matrix = [
                        [yaw_cos, 0.0, yaw_sin, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [-yaw_sin, 0.0, yaw_cos, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32]
                    ];

                    // Scale Maxwell to 5% of his size
                    // Also invert the x axis because he gets inverted for some reason...?
                    let scale_matrix = [
                        [-0.05, 0.0, 0.0, 0.0],  // x
                        [0.0, 0.05, 0.0, 0.0],  // y
                        [0.0, 0.0, 0.05, 0.0],  // z 
                        [0.0, -0.5, 0.0, 1.0f32] // w
                    ];
                    

                    let params = glium::DrawParameters {
                        // Depth buffer stuff
                        depth: glium::Depth {
                            test: glium::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        blend: glium::Blend::alpha_blending(), // Allow whiskers to draw ontop of other stuff
                        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise, // Enable backface culling (free perfromance!)
                        .. Default::default()
                    };

                    // Draw maxwell
                    target.draw(&positions, &indices, &program, &uniform! {
                        pitch_matrix: pitch_matrix,
                        roll_matrix: roll_matrix,
                        yaw_matrix: yaw_matrix,
                        scale_matrix: scale_matrix,
                        tex: &texture,
                    },
                        &params).unwrap();

                    // And don't forget about his whiskers!
                    target.draw(&whiskers_positions, &whiskers_indices, &program, &uniform! {
                        pitch_matrix: pitch_matrix,
                        roll_matrix: roll_matrix,
                        yaw_matrix: yaw_matrix,
                        scale_matrix: scale_matrix,
                        tex: &whiskers_texture,
                    },
                        &params).unwrap();
                
                    // Draw to screen
                    target.finish().unwrap();
                },
                _ => (),
            },    
            glium::winit::event::Event::AboutToWait => {
                window.request_redraw(); // Constant framerate, with vsync handled by winit
            },        
            _ => (),
        };
    });

}
