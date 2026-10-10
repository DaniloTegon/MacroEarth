use std::{env, fs, path::PathBuf};

use resvg::usvg::TreeParsing;

fn main() {
    println!("cargo:rerun-if-changed=assets/super_earth_emblem.svg");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let parsed_tree = resvg::usvg::Tree::from_data(
        include_bytes!("assets/super_earth_emblem.svg"),
        &resvg::usvg::Options::default(),
    )
    .expect("MacroEarth icon SVG must be valid");
    let tree = resvg::Tree::from_usvg(&parsed_tree);
    let icon_data = create_ico(&tree);
    let icon_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("macroearth.ico");
    fs::write(&icon_path, icon_data).expect("failed to write MacroEarth icon");

    let mut resources = winres::WindowsResource::new();
    resources.set_icon(icon_path.to_str().expect("icon path must be valid UTF-8"));
    resources.set("FileDescription", "MacroEarth");
    resources.set("ProductName", "MacroEarth");
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        println!("cargo:rustc-link-arg-bins=-mwindows");
        resources.set_toolkit_path(&env::var("CARGO_MANIFEST_DIR").unwrap());
        if let Ok(windres) = env::var("WINDRES") {
            resources.set_windres_path(&windres);
        }
        if let Ok(archiver) = env::var("AR") {
            resources.set_ar_path(&archiver);
        }
    }
    resources
        .compile()
        .expect("failed to embed MacroEarth icon");
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        let resource_object = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("resource.o");
        println!("cargo:rustc-link-arg-bins={}", resource_object.display());
    }
}

fn create_ico(tree: &resvg::Tree) -> Vec<u8> {
    let sizes = [16_u32, 24, 32, 48, 64, 128, 256];
    let mut images = Vec::new();

    for size in sizes {
        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(size, size).expect("icon dimensions must be valid");
        let scale_x = size as f32 / tree.size.width();
        let scale_y = size as f32 / tree.size.height();
        tree.render(
            resvg::tiny_skia::Transform::from_scale(scale_x, scale_y),
            &mut pixmap.as_mut(),
        );

        let mask_row_size = size.div_ceil(32) * 4;
        let xor_size = size * size * 4;
        let mut image = Vec::with_capacity((40 + xor_size + mask_row_size * size) as usize);
        image.extend_from_slice(&40_u32.to_le_bytes());
        image.extend_from_slice(&(size as i32).to_le_bytes());
        image.extend_from_slice(&((size * 2) as i32).to_le_bytes());
        image.extend_from_slice(&1_u16.to_le_bytes());
        image.extend_from_slice(&32_u16.to_le_bytes());
        image.extend_from_slice(&0_u32.to_le_bytes());
        image.extend_from_slice(&xor_size.to_le_bytes());
        image.extend_from_slice(&0_i32.to_le_bytes());
        image.extend_from_slice(&0_i32.to_le_bytes());
        image.extend_from_slice(&0_u32.to_le_bytes());
        image.extend_from_slice(&0_u32.to_le_bytes());

        for y in (0..size).rev() {
            for x in 0..size {
                let offset = ((y * size + x) * 4) as usize;
                let rgba = &pixmap.data()[offset..offset + 4];
                image.extend_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]);
            }
        }
        image.resize(image.len() + (mask_row_size * size) as usize, 0);
        images.push(image);
    }

    let directory_size = 6 + sizes.len() * 16;
    let mut icon = Vec::with_capacity(directory_size + images.iter().map(Vec::len).sum::<usize>());
    icon.extend_from_slice(&0_u16.to_le_bytes());
    icon.extend_from_slice(&1_u16.to_le_bytes());
    icon.extend_from_slice(&(sizes.len() as u16).to_le_bytes());

    let mut offset = directory_size as u32;
    for (size, image) in sizes.into_iter().zip(&images) {
        let dimension = if size == 256 { 0 } else { size as u8 };
        icon.extend_from_slice(&[dimension, dimension, 0, 0]);
        icon.extend_from_slice(&1_u16.to_le_bytes());
        icon.extend_from_slice(&32_u16.to_le_bytes());
        icon.extend_from_slice(&(image.len() as u32).to_le_bytes());
        icon.extend_from_slice(&offset.to_le_bytes());
        offset += image.len() as u32;
    }
    for image in images {
        icon.extend_from_slice(&image);
    }
    icon
}
