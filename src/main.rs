// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::imageops::resize;
use image::imageops::FilterType::Gaussian;
use image::io::Reader as ImageReader;
use image::{EncodableLayout, ImageBuffer, ImageOutputFormat, Rgba};
use tauri::{command, Window};

use base64::Engine;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            build_icons,
            browse_png,
            get_web_img_png_base64,
            save_file_diag,
            browse_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[command]
fn browse_png(window: Window) -> Option<String> {
    let diag = native_dialog::FileDialog::new();
    let path: Option<PathBuf> = diag
        .set_location("~/Desktop")
        .add_filter(
            "Image File",
            &[
                "png", "jpg", "jpeg", "bmp", "ico", "gif", "tga", "tif", "tiff", "webp",
            ],
        )
        .set_owner(&window)
        .show_open_single_file()
        .unwrap();
    if path.is_none() {
        None
    } else {
        let path = path.unwrap();
        let path = path.to_str().unwrap();
        Some(path.to_string())
    }
}

#[command]
fn browse_dir(window: tauri::Window) -> Option<String> {
    let diag = native_dialog::FileDialog::new();
    let path = diag
        .set_owner(&window)
        .set_location("~/Desktop")
        .show_open_single_dir()
        .unwrap();
    if let Some(path) = path {
        Some(path.to_str().unwrap().to_string())
    } else {
        None
    }
}

#[command]
fn save_file_diag(window: tauri::Window, filters: HashMap<String, Vec<String>>) -> Option<String> {
    let mut diag = native_dialog::FileDialog::new().set_owner(&window);

    let mut filter_exts_list = Vec::new();
    let mut filter_ext_refs_list = Vec::<Vec<&str>>::new();
    let mut keys = Vec::new();
    for (k, v) in filters {
        keys.insert(0, k.clone());
        filter_exts_list.insert(0, v.clone());
    }
    for i in 0..filter_exts_list.len() {
        let mut ext_refs_list = Vec::<&str>::new();
        for ext in &filter_exts_list[i] {
            ext_refs_list.push(ext.as_str());
        }
        filter_ext_refs_list.push(ext_refs_list);
    }

    for i in 0..keys.len() {
        diag = diag.add_filter(&keys[i], &filter_ext_refs_list[i]);
    }

    let path = diag
        .set_location("~/Desktop")
        .show_save_single_file()
        .unwrap();
    if let Some(path) = path {
        Some(path.to_str().unwrap().to_string())
    } else {
        None
    }
}

#[command]
fn get_web_img_png_base64(image_path: &str) -> Option<String> {
    let reader = ImageReader::open(image_path);
    if let Err(err) = reader {
        return Some(format!("{:?}", err));
    }
    let reader = reader.unwrap();

    let img = reader.decode();
    if let Err(err) = img {
        return Some(format!("{:?}", err));
    }
    let img = img.unwrap();

    let img = img.into_rgba8();

    let mut c = Cursor::new(Vec::new());
    let ret = img.write_to(&mut c, ImageOutputFormat::Png);

    c.seek(std::io::SeekFrom::Start(0)).unwrap();

    if ret.is_err() {
        None
    } else {
        let mut buf = Vec::new();
        c.read_to_end(&mut buf).unwrap();
        let base64 = base64::engine::general_purpose::STANDARD.encode(&buf);
        Some(base64)
    }
}

#[command]
async fn build_icons(image_path: &str, target_dir: &str) -> Result<(), String> {
    let image_path = image_path.to_string();
    let target_dir = target_dir.to_string();
    let ret = tokio::task::spawn_blocking(||{
        build_icons_sync(image_path, target_dir)
    }).await.unwrap();
    ret
}

fn build_icons_sync(image_path: String, target_dir: String) -> Result<(), String> {
    let image_path = image_path.as_str();
    let target_dir = target_dir.as_str();
    if !Path::new(image_path).exists() {
        return Err(String::from_str("file not found").unwrap());
    }
    let reader = ImageReader::open(image_path);
    if let Err(err) = reader {
        return Err(format!("{:?}", err));
    }
    let reader = reader.unwrap();

    let img = reader.decode();
    if let Err(err) = img {
        return Err(format!("{:?}", err));
    }
    let img = img.unwrap();

    let org_width = img.width();
    let org_height = img.height();
    let edge = min(org_width, org_height);
    let mut img = img.into_rgba8();

    let square_img = image::imageops::crop(
        &mut img,
        (org_width - edge) / 2,
        (org_height - edge) / 2,
        edge,
        edge,
    );

    let square_img = square_img.to_image();
    let edges = vec![
        512, 256, 128, 96, 64, 48, 32, 24, 16, 310, 284, 150, 142, 107, 89, 71, 44, 30, 50,
    ];
    let mut size_images = HashMap::new();
    for ed in edges {
        size_images.insert(ed, resize(&square_img, ed, ed, Gaussian));
    }

    std::fs::create_dir_all(target_dir).unwrap();

    fill_icns(&size_images, "icon.icns", target_dir);

    fill_ico(&size_images, "icon.ico", target_dir);

    let mut pngs = HashMap::new();
    pngs.insert(32u32, "32x32");
    pngs.insert(128, "128x128");
    pngs.insert(256, "128x128@2x");
    pngs.insert(30, "Square30x30Logo");
    pngs.insert(44, "Square44x44Logo");
    pngs.insert(71, "Square71x71Logo");
    pngs.insert(89, "Square89x89Logo");
    pngs.insert(107, "Square107x107Logo");
    pngs.insert(142, "Square142x142Logo");
    pngs.insert(150, "Square150x150Logo");
    pngs.insert(284, "Square284x284Logo");
    pngs.insert(310, "Square310x310Logo");
    pngs.insert(512, "icon.png");
    pngs.insert(50, "StoreLogo");

    fill_pngs(&size_images, &pngs, target_dir);
    Ok(())
}


fn fill_pngs(
    size_images: &HashMap<u32, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    target_pngs: &HashMap<u32, &str>,
    dir_name: &str,
) {
    for (size, target_name) in target_pngs {
        let img = size_images.get(&size);
        if img.is_none() {
            println!("{} not found", target_name);
            continue;
        }
        let img = img.unwrap();

        let mut target_name: String = target_name.to_string();
        if !target_name.contains('.') {
            target_name.push_str(".png");
        }

        let full_path = Path::new(dir_name).join(target_name);
        let mut file = File::create(full_path).unwrap();
        img.write_to(&mut file, image::ImageOutputFormat::Png)
            .unwrap();
    }
}

fn fill_ico(
    size_images: &HashMap<u32, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    target_name: &str,
    dir_name: &str,
) {
    let mut imgs = Vec::new();

    let sizes = [256, 64, 32, 16];
    for size in sizes {
        let img = size_images.get(&size).unwrap();
        let frm = image::Frame::new(img.clone());
        let ifrm = IcoFrame::as_png(&frm.buffer(), size, size, image::ColorType::Rgba8).unwrap();
        imgs.push(ifrm);
    }

    let mut target_name = target_name.to_string();
    if !target_name.contains('.') {
        target_name.push_str(".ico");
    }
    let full_path = Path::new(dir_name).join(target_name);

    let ico_encoder = IcoEncoder::new(File::create(full_path).unwrap());
    ico_encoder.encode_images(imgs.as_slice()).unwrap();
}

fn fill_icns(
    size_images: &HashMap<u32, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    target_name: &str,
    dir_name: &str,
) {
    let mut icns_img = icns::IconFamily::new();

    let sizes = [512, 256, 128, 32, 16];
    for size in sizes {
        let mut icn_img = icns::Image::new(icns::PixelFormat::RGBA, size, size);
        let buf = icn_img.data_mut();
        let img = size_images.get(&size).unwrap();
        let bytes = img.as_bytes();
        for i in 0..(size * size * 4) as usize {
            buf[i] = bytes[i];
        }
        icns_img.add_icon(&icn_img).unwrap();
    }

    let mut target_name = target_name.to_string();
    if !target_name.contains('.') {
        target_name.push_str(".icns");
    }
    let full_path = Path::new(dir_name).join(target_name);

    icns_img.write(File::create(full_path).unwrap()).unwrap();
}
