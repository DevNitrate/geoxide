mod tiff_utils;

use bevy::{DefaultPlugins, app::{App, Startup, Update}, asset::{Asset, Assets, Handle}, camera::{Camera3d, OrthographicProjection, Projection, ScalingMode}, camera_controller::free_camera::{FreeCamera, FreeCameraPlugin}, dev_tools::fps_overlay::FpsOverlayPlugin, ecs::{message::MessageReader, system::{Commands, Query, ResMut, Single}}, image::Image, math::{Vec2, Vec3, primitives::Rectangle}, mesh::{Mesh, Mesh3d}, pbr::{Material, MaterialPlugin, MeshMaterial3d}, reflect::TypePath, render::render_resource::AsBindGroup, transform::components::Transform, window::{PresentMode, Window, WindowResized}};
use crate::tiff_utils::load_tiff;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<ScreenMaterial>::default(), FreeCameraPlugin))
        .add_plugins(FpsOverlayPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (resize_quad_system, update_material))
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ScreenMaterial>>, mut win: Single<&mut Window>, mut images: ResMut<Assets<Image>>) {
    win.present_mode = PresentMode::AutoNoVsync;
    let quad = meshes.add(Rectangle::from_size(Vec2 { x: win.width(), y: win.height() }));
    let (tiff_img, max_height, min_height) = load_tiff("cali_final.tif", false, false, Some("cali_final.tif"));
    let img: Handle<Image> = images.add(tiff_img);
    let material = materials.add(ScreenMaterial {
        width: win.width(),
        height: win.height(),
        image: img,
        forward_vec: Vec3::NEG_Z,
        up_vec: Vec3::Y,
        camera_pos: Vec3::ZERO,
        max_height: max_height,
        min_height: min_height,
        scale_factor: 0.05
    });

    commands.spawn((
        Camera3d::default(),
        FreeCamera {
            walk_speed: 6000.0,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 10000.0).looking_at(Vec3::ZERO, Vec3::Y),
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
    mut materials: ResMut<Assets<ScreenMaterial>>
) {
    for event in resize_events.read() {
        let new_size = Vec2::new(event.width, event.height);
        for mesh in meshes.iter_mut() {
            *(mesh.1) = Rectangle::from_size(new_size).into();
        }

        for material in materials.iter_mut() {
            (*(material.1)).width = event.width;
            (*(material.1)).height = event.height;
        }
    }
}

fn update_material(query: Query<(&FreeCamera, &Transform)>, mut materials: ResMut<Assets<ScreenMaterial>>) {
    for material in materials.iter_mut() {
        for camera in query {
            (*(material.1)).up_vec = (camera.1).up().as_vec3();
            (*(material.1)).forward_vec = (camera.1).forward().as_vec3();
            (*(material.1)).camera_pos = (camera.1).translation;
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ScreenMaterial {
    #[uniform(100)]
    width: f32,
    #[uniform(101)]
    height: f32,
    #[uniform(102)]
    forward_vec: Vec3,
    #[uniform(103)]
    up_vec: Vec3,
    #[uniform(104)]
    camera_pos: Vec3,
    #[uniform(105)]
    max_height: f32,
    #[uniform(106)]
    min_height: f32,
    #[uniform(107)]
    scale_factor: f32,
    #[texture(1)]
    #[sampler(2)]
    image: Handle<Image>
}

impl Material for ScreenMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shader/frag.wgsl".into()
    }
}