use std::{fs::File, io::{BufReader, BufWriter}, time::Instant};

use bevy::{asset::RenderAssetUsages, image::Image, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};
use bytemuck::cast_slice;
use tiff::{ColorType, decoder::{Decoder, DecodingResult}, encoder::{TiffEncoder, colortype}};

pub fn load_tiff(path: &str, compute: bool, save: bool, output_path: Option<&str>) -> Image {
    let (mut data_i16, width, height): (Vec<i16>, u32, u32) = rgba16_from_tiff(path);

    let now = Instant::now();

    if compute {
        compute_tiff(&mut data_i16, width, height, 16);
    }

    if save {
        write_tiff(output_path.unwrap(), &data_i16, width, height);
    }

    println!("took {} seconds to create texture", now.elapsed().as_secs_f64());

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

pub fn rgba16_from_tiff(path: &str) -> (Vec<i16>, u32, u32) {
    let file: File = File::open(format!("assets/{}", path)).unwrap();
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

        DecodingResult::U16(data) => {
            data.into_iter()
                .map(|v| (v as i32 - i16::MIN as i32) as i16)
                .collect()
        }

        other => panic!("Unsupported TIFF buffer type: {:?}", other),
    };


    let mut buf: Vec<i16> = vec![0; (width * height * 4) as usize];

    match colortype {
        ColorType::RGBA(_) => {
            buf = data_i16
        },
        ColorType::Gray(_) => {
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let idx: usize = y * width as usize + x;
                    let pixel_val = data_i16[idx];
                    let dst_idx = idx * 4;
                    buf[dst_idx + 0] = pixel_val;
                    buf[dst_idx + 1] = pixel_val;
                    buf[dst_idx + 2] = pixel_val;
                    buf[dst_idx + 3] = 32767; // max alpha
                }
            }
        },
        ColorType::RGB(_) => {
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let idx: usize = (y * width as usize + x) * 3;
                    let dst_idx: usize = (y * width as usize + x) * 4;
                    buf[dst_idx + 0] = data_i16[idx + 0];
                    buf[dst_idx + 1] = data_i16[idx + 1];
                    buf[dst_idx + 2] = data_i16[idx + 2];
                    buf[dst_idx + 3] = 32767; // max alpha
                }
            }
        },
        _ => unreachable!()
    }

    return (buf, width, height);
}

pub fn compute_tiff(data: &mut Vec<i16>, width: u32, height: u32, radius: i16) {
    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx: usize = (y * width as usize + x) * 4;
            
            // let r_idx: usize = idx;
            let g_idx: usize = idx + 1;
            let b_idx: usize = idx + 2;
            let a_idx: usize = idx + 3;

            let mut max_height_in_radius: i16 = i16::MIN;
            let mut max_diff_in_radius: i16 = 0;

            let rad: isize = radius as isize;
            let rad_sqr: isize = rad*rad;

            for j in -rad..=rad {
                for i in -rad..=rad {
                    if (i*i + j*j) > rad_sqr {
                        continue;
                    }

                    let x_rad: isize = x as isize + i;
                    let y_rad: isize = y as isize + j;

                    if x_rad < 0 || y_rad < 0 || x_rad >= width as isize || y_rad >= height as isize {
                        continue;
                    }

                    let radius_idx: usize = ((y_rad as usize) * width as usize + (x_rad as usize)) * 4;
                    let rad_height: i16 = data[radius_idx];

                    let neighbors = [
                        (-1, -1), (0, -1), (1, -1),
                        (-1,  0),          (1,  0),
                        (-1,  1), (0,  1), (1,  1),
                    ];

                    for (dx, dy) in neighbors {
                        let nx = x_rad + dx;
                        let ny = y_rad + dy;

                        if nx < 0 || ny < 0 ||
                        nx >= width as isize ||
                        ny >= height as isize {
                            continue;
                        }

                        let n_idx =
                            ((ny as usize) * width as usize + nx as usize) * 4;

                        let diff = data[n_idx] - rad_height;
                        max_diff_in_radius = max_diff_in_radius.max(diff);
                    }

                    if rad_height > max_height_in_radius {
                        max_height_in_radius = rad_height;
                    }
                }
            }

            data[g_idx] = max_height_in_radius;
            data[b_idx] = max_diff_in_radius;
            data[a_idx] = radius;
        }
    }
}

fn write_tiff(path: &str, data: &[i16], width: u32, height: u32) {
    let file = File::create(format!("assets/{}", path)).unwrap();
    let writer = BufWriter::new(file);

    let mut encoder = TiffEncoder::new(writer).unwrap();
    let image = encoder.new_image::<colortype::RGBA16>(width, height).unwrap();

    let data_u16: Vec<u16> = data.iter().map(|&v| ((v as i32) - (i16::MIN as i32)) as u16).collect();

    image.write_data(&data_u16).unwrap();
}