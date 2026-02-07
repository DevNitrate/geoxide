use std::{fs::File, io::BufReader};

use bevy::{DefaultPlugins, app::{App, Startup}, asset::{Asset, Assets, Handle, RenderAssetUsages}, camera::Camera3d, camera_controller::free_camera::{FreeCamera, FreeCameraPlugin}, dev_tools::fps_overlay::FpsOverlayPlugin, ecs::system::{Commands, ResMut, Single}, image::Image, math::{Vec2, Vec3, primitives::Rectangle}, mesh::{Mesh, Mesh3d}, pbr::{Material, MaterialPlugin, MeshMaterial3d}, reflect::TypePath, render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat}, transform::components::Transform, window::{PresentMode, Window}};
use bytemuck::cast_slice;
use tiff::{ColorType, decoder::{Decoder, DecodingResult}};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<ScreenMaterial>::default(), FreeCameraPlugin))
        .add_plugins(FpsOverlayPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ScreenMaterial>>, mut win: Single<&mut Window>, mut images: ResMut<Assets<Image>>) {
    win.present_mode = PresentMode::AutoNoVsync;
    commands.spawn((Camera3d::default(), FreeCamera::default(), Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y)));

    let quad = meshes.add(Rectangle::from_size(Vec2 { x: win.width(), y: win.height() }));
    let img: Handle<Image> = images.add(load_tiff("assets/gebco.tif"));
    let material = materials.add(ScreenMaterial {
        width: 100,
        height: 100,
        image: img
    });

    commands.spawn((
        Mesh3d(quad),
        MeshMaterial3d(material)
    ));
}

fn load_tiff(path: &str) -> Image {
    let file: File = File::open(path).unwrap();
    let mut decoder = Decoder::new(BufReader::new(file)).unwrap();

    let (width, height): (u32, u32) = decoder.dimensions().unwrap();
    let mut buf = match decoder.colortype().unwrap() {
        ColorType::Gray(_) =>  {
            println!("gray");
            DecodingResult::I16(vec![0; (width * height) as usize])
        },
        ColorType::RGB(_) => {
            println!("rgba");
            DecodingResult::I16(vec![0; (width * height * 3) as usize])
        }
        ColorType::RGBA(_) => {
            println!("rgba");
            DecodingResult::I16(vec![0; (width * height * 4) as usize])
        }
        _ => panic!("unsupported tiff type")
    };

    let _ = decoder.read_image_to_buffer(&mut buf).unwrap();
    let data_i16: Vec<i16> = match buf {
        DecodingResult::I16(data) => data,
        _ => panic!("Unexpected buffer type; expected I16"),
    };

    let data_u32: Vec<u32> = data_i16.iter().map(|&v| v as u32).collect();

    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1
        },
        TextureDimension::D2,
        cast_slice(&data_u32).to_vec(),
        TextureFormat::R32Sint,
        RenderAssetUsages::all()
    );

    return image;
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