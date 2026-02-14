use bevy::{ecs::{resource::Resource, system::ResMut, world::World}, reflect::TypePath};
use bevy_app_compute::prelude::{AppComputeWorker, AppComputeWorkerBuilder, ComputeShader, ComputeWorker};

#[derive(TypePath)]
struct TiffShader;

impl ComputeShader for TiffShader {
    fn shader() -> bevy_app_compute::prelude::ShaderRef {
        "shader/compute_tiff.wgsl".into()
    }
}

#[derive(Resource)]
pub struct SimpleComputeWorker;

impl ComputeWorker for SimpleComputeWorker {
    fn build(world: &mut World) -> bevy_app_compute::prelude::AppComputeWorker<Self> {
        let worker = AppComputeWorkerBuilder::new(world)
            .add_staging("values", &[1, 2, 3, 4])
            .add_pass::<TiffShader>([4, 1, 1], &["values"])
            .one_shot()
            .build();

        worker
    }
}

pub fn compute_tiff(data: &mut Vec<i32>, width: u32, height: u32, radius: u32, mut worker: ResMut<AppComputeWorker<SimpleComputeWorker>>) {
    worker.execute();

    println!("output from compute: {:?}", worker.read_vec::<i32>("values"));
}
