use std::{fs::File, io::BufReader};

use bevy::{DefaultPlugins, app::{App, Startup, Update}, asset::{Asset, Assets, Handle, RenderAssetUsages}, camera::{Camera3d, OrthographicProjection, Projection, ScalingMode}, camera_controller::free_camera::{FreeCamera, FreeCameraPlugin}, dev_tools::fps_overlay::FpsOverlayPlugin, ecs::{message::MessageReader, system::{Commands, ResMut, Single}}, image::Image, math::{Vec2, Vec3, primitives::Rectangle}, mesh::{Mesh, Mesh3d}, pbr::{Material, MaterialPlugin, MeshMaterial3d}, reflect::TypePath, render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat}, transform::components::Transform, window::{PresentMode, Window, WindowResized}};
use bytemuck::cast_slice;
use tiff::{ColorType, decoder::{Decoder, DecodingResult}};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<ScreenMaterial>::default(), FreeCameraPlugin))
        .add_plugins(FpsOverlayPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, resize_quad_system)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ScreenMaterial>>, mut win: Single<&mut Window>, mut images: ResMut<Assets<Image>>) {
    win.present_mode = PresentMode::AutoNoVsync;
    let quad = meshes.add(Rectangle::from_size(Vec2 { x: win.width(), y: win.height() }));
    let img: Handle<Image> = images.add(load_tiff("assets/gebco.tif"));
    let material = materials.add(ScreenMaterial {
        width: 100,
        height: 100,
        image: img
    });

    commands.spawn((
        Camera3d::default(),
        FreeCamera {
            walk_speed: 600.0,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::NEG_Z, Vec3::Y),
        Projection::from(
            OrthographicProjection {
                scale: 1.0,
                scaling_mode: ScalingMode::WindowSize,
                viewport_origin: Vec2::new(0.5, 0.5),
                near: 0.0,
                far: 1000.0,
                area: Default::default()
            }
        ),
    )).with_children(|parent| {
        parent.spawn((
            Mesh3d(quad),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, -1.0)
        ));
    });
}

fn resize_quad_system(
    mut resize_events: MessageReader<WindowResized>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in resize_events.read() {
        let new_size = Vec2::new(event.width, event.height);
        for mesh in meshes.iter_mut() {
            *(mesh.1) = Rectangle::from_size(new_size).into();
        }
    }
}

fn load_tiff(path: &str) -> Image {
    let file: File = File::open(path).unwrap();
    let mut decoder = Decoder::new(BufReader::new(file)).unwrap();

    let (width, height): (u32, u32) = decoder.dimensions().unwrap();
    let data_i16 = compute_tiff("assets/gebco.tif");

    let data_f32: Vec<f32> = data_i16.iter().map(|&v| (v as f32) / 10930.0).collect();

    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1
        },
        TextureDimension::D2,
        cast_slice(&data_f32).to_vec(),
        TextureFormat::Rgba32Float,
        RenderAssetUsages::all()
    );

    return image;
}

fn compute_tiff(path: &str) -> Vec<i16> {
    let file: File = File::open(path).unwrap();
    let mut decoder = Decoder::new(BufReader::new(file)).unwrap();

    let (width, height): (u32, u32) = decoder.dimensions().unwrap();
    let colortype: ColorType = decoder.colortype().unwrap();

    let mut src_buf = match colortype {
        ColorType::Gray(_) =>  {
            println!("gray");
            DecodingResult::I16(vec![0; (width * height) as usize])
        },
        ColorType::RGB(_) => {
            println!("rgb");
            DecodingResult::I16(vec![0; (width * height * 3) as usize])
        }
        ColorType::RGBA(_) => {
            println!("rgba");
            DecodingResult::I16(vec![0; (width * height * 4) as usize])
        }
        _ => panic!("unsupported tiff type")
    };

    let _ = decoder.read_image_to_buffer(&mut src_buf).unwrap();
    let data_i16: Vec<i16> = match src_buf {
        DecodingResult::I16(data) => data,
        _ => panic!("Unexpected buffer type; expected I16"),
    };

    let mut buf: Vec<i16> = vec![0; (width * height * 4) as usize];

    match colortype {
        ColorType::RGBA(_) => {
            buf = data_i16
        },
        ColorType::Gray(_) => {
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let idx = y * width as usize + x;
                    let pixel_val = data_i16[idx];
                    let dst_idx = idx * 4;
                    buf[dst_idx + 0] = pixel_val;
                    buf[dst_idx + 1] = pixel_val;
                    buf[dst_idx + 2] = pixel_val;
                    buf[dst_idx + 3] = 32767; // max alpha
                }
            }
        }
        _ => unreachable!()
    }

    return buf;
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ScreenMaterial {
    #[uniform(100)]
    width: u32,
    #[uniform(101)]
    height: u32,
    #[texture(1)]
    #[sampler(2)]
    image: Handle<Image>
}

impl Material for ScreenMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shader/frag.wgsl".into()
    }
}