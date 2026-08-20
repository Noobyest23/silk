use std::collections::HashMap;
use crate::environment::vm::VirtualMachine;
use super::super::value::SilkValue;
use image::{DynamicImage, GenericImageView, GenericImage, RgbImage, Rgba, imageops::FilterType};

fn extract_image<'a>(vm: &'a mut VirtualMachine, self_val: &'a SilkValue) -> Option<&'a DynamicImage> {
    let obj_val = match self_val {
        SilkValue::Pointer(ptr) => vm.heap.get(ptr)?,
        val => val,
    };

    if let SilkValue::Object(map) = obj_val {
        if let Some(data_val) = map.get("data") {
            return data_val.downcast_ref::<DynamicImage>();
        }
    }
    None
}

fn extract_image_mut<'a>(vm: &'a mut VirtualMachine, self_val: &'a SilkValue) -> Option<&'a mut DynamicImage> {
    let obj_val = match self_val {
        SilkValue::Pointer(ptr) => vm.heap.get_mut(ptr)?,
        _ => return None,
    };

    if let SilkValue::Object(map) = obj_val {
        if let Some(data_val) = map.get_mut("data") {
            return data_val.downcast_mut::<DynamicImage>();
        }
    }
    None
}

fn construct_image_object(vm: &mut VirtualMachine, img: DynamicImage) -> SilkValue {
    let mut obj = HashMap::new();
    obj.insert("data".to_string(), SilkValue::new_native(img));
    
    obj.insert("width".to_string(), SilkValue::NativeFn(silk_image_width, "Image.width() -> Int".to_string()));
    obj.insert("height".to_string(), SilkValue::NativeFn(silk_image_height, "Image.height() -> Int".to_string()));
    obj.insert("resize".to_string(), SilkValue::NativeFn(silk_image_resize, "Image.resize(w: Int, h: Int) -> Image".to_string()));
    obj.insert("ascii".to_string(), SilkValue::NativeFn(silk_image_ascii, "Image.ascii(gradient: String = ' .:-=+*#%@', max_w: Int = null, max_h: Int = null) -> String".to_string()));
    obj.insert("save".to_string(), SilkValue::NativeFn(silk_image_save, "Image.save(path: String) -> Bool".to_string()));
    obj.insert("get_pixel".to_string(), SilkValue::NativeFn(silk_image_get_pixel, "Image.get_pixel(x: Int, y: Int) -> Array[4]".to_string()));
    obj.insert("set_pixel".to_string(), SilkValue::NativeFn(silk_image_set_pixel, "Image.set_pixel(x: Int, y: Int, r: Int, g: Int, b: Int, a: Int = 255)".to_string()));
    obj.insert("crop".to_string(), SilkValue::NativeFn(silk_image_crop, "Image.crop(x: Int, y: Int, w: Int, h: Int) -> Image".to_string()));
    obj.insert("grayscale".to_string(), SilkValue::NativeFn(silk_image_grayscale, "Image.grayscale() -> Image".to_string()));
    obj.insert("invert".to_string(), SilkValue::NativeFn(silk_image_invert, "Image.invert() -> Image".to_string()));
    obj.insert("flip_h".to_string(), SilkValue::NativeFn(silk_image_flip_h, "Image.flip_h() -> Image".to_string()));
    obj.insert("flip_v".to_string(), SilkValue::NativeFn(silk_image_flip_v, "Image.flip_v() -> Image".to_string()));

    let ptr = vm.next_heap_ptr;
    vm.heap_allocate(SilkValue::Object(obj));
    SilkValue::Pointer(ptr)
}

pub fn silk_image(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() > 1 {
        vm.error("'Image' constructor expects 1 or no arguments".to_string());
        return SilkValue::Null;
    }

    let img = if args.len() == 1 {
        if let Some(filepath) = vm.heap_get_string(args[0].clone()) {
            match image::open(&filepath) {
                Ok(dy_img) => dy_img,
                Err(err) => {
                    vm.error(format!("'Image' constructor could not open image: {}", err));
                    return SilkValue::Null;
                }
            }
        } else {
            vm.error("'Image' constructor expects first argument to be a string path".to_string());
            return SilkValue::Null;
        }
    } else {
        DynamicImage::ImageRgb8(RgbImage::new(1, 1))
    };

    construct_image_object(vm, img)
}

pub fn silk_image_width(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    extract_image(vm, &args[0])
        .map(|img| SilkValue::Int(img.width() as i32))
        .unwrap_or(SilkValue::Null)
}

pub fn silk_image_height(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    extract_image(vm, &args[0])
        .map(|img| SilkValue::Int(img.height() as i32))
        .unwrap_or(SilkValue::Null)
}

pub fn silk_image_save(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() < 2 {
        vm.error("Image.save(path) requires target file path".to_string());
        return SilkValue::Bool(false);
    }

    let filepath = match vm.heap_get_string(args[1].clone()) {
        Some(p) => p,
        None => {
            vm.error("Image.save() argument must be a string".to_string());
            return SilkValue::Bool(false);
        }
    };

    if let Some(img) = extract_image(vm, &args[0]) {
        match img.save(&filepath) {
            Ok(_) => SilkValue::Bool(true),
            Err(err) => {
                vm.error(format!("Failed to save image: {}", err));
                SilkValue::Bool(false)
            }
        }
    } else {
        SilkValue::Bool(false)
    }
}

pub fn silk_image_get_pixel(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() < 3 {
        vm.error("Image.get_pixel(x, y) expects x and y coordinates".to_string());
        return SilkValue::Null;
    }

    let (x, y) = match (args[1].as_int(), args[2].as_int()) {
        (Some(x), Some(y)) if x >= 0 && y >= 0 => (x as u32, y as u32),
        _ => return SilkValue::Null,
    };

    if let Some(img) = extract_image(vm, &args[0]) {
        if x < img.width() && y < img.height() {
            let pixel = img.get_pixel(x, y);
            let color_array = vec![
                SilkValue::Int(pixel[0] as i32),
                SilkValue::Int(pixel[1] as i32),
                SilkValue::Int(pixel[2] as i32),
                SilkValue::Int(pixel[3] as i32),
            ];

            let ptr = vm.next_heap_ptr;
            vm.heap_allocate(SilkValue::List(color_array));
            return SilkValue::Pointer(ptr);
        }
    }

    SilkValue::Null
}

pub fn silk_image_set_pixel(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() < 6 {
        vm.error("Image.set_pixel(x, y, r, g, b, a) expects coordinates and RGBA channels".to_string());
        return SilkValue::Null;
    }

    let x = args[1].as_int().unwrap_or(-1);
    let y = args[2].as_int().unwrap_or(-1);
    let r = args[3].as_int().unwrap_or(0) as u8;
    let g = args[4].as_int().unwrap_or(0) as u8;
    let b = args[5].as_int().unwrap_or(0) as u8;
    let a = if args.len() > 6 { args[6].as_int().unwrap_or(255) as u8 } else { 255 };

    if x >= 0 && y >= 0 {
        if let Some(img) = extract_image_mut(vm, &args[0]) {
            let (u_x, u_y) = (x as u32, y as u32);
            if u_x < img.width() && u_y < img.height() {
                img.put_pixel(u_x, u_y, Rgba([r, g, b, a]));
            }
        }
    }

    SilkValue::Null
}

pub fn silk_image_crop(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() < 5 {
        vm.error("Image.crop(x, y, w, h) expects 4 positioning parameters".to_string());
        return SilkValue::Null;
    }

    let x = args[1].as_int().unwrap_or(0) as u32;
    let y = args[2].as_int().unwrap_or(0) as u32;
    let w = args[3].as_int().unwrap_or(0) as u32;
    let h = args[4].as_int().unwrap_or(0) as u32;

    let cropped = {
        if let Some(img) = extract_image(vm, &args[0]) {
            img.crop_imm(x, y, w, h)
        } else {
            return SilkValue::Null;
        }
    };

    construct_image_object(vm, cropped)
}

pub fn silk_image_resize(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.len() < 3 { return SilkValue::Null; }
    let (new_w, new_h) = (args[1].as_int().unwrap_or(1) as u32, args[2].as_int().unwrap_or(1) as u32);

    let resized = {
        if let Some(img) = extract_image(vm, &args[0]) {
            img.resize_exact(new_w, new_h, FilterType::Triangle)
        } else {
            return SilkValue::Null;
        }
    };

    construct_image_object(vm, resized)
}

pub fn silk_image_grayscale(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    let gray = {
        if let Some(img) = extract_image(vm, &args[0]) {
            img.grayscale()
        } else {
            return SilkValue::Null;
        }
    };

    construct_image_object(vm, gray)
}

pub fn silk_image_invert(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    let inverted = {
        if let Some(img) = extract_image(vm, &args[0]) {
            let mut inv = img.clone();
            inv.invert();
            inv
        } else {
            return SilkValue::Null;
        }
    };

    construct_image_object(vm, inverted)
}

pub fn silk_image_flip_h(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    let flipped = {
        if let Some(img) = extract_image(vm, &args[0]) {
            img.fliph()
        } else {
            return SilkValue::Null;
        }
    };

    construct_image_object(vm, flipped)
}

pub fn silk_image_flip_v(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    let flipped = {
        if let Some(img) = extract_image(vm, &args[0]) {
            img.flipv()
        } else {
            return SilkValue::Null;
        }
    };

    construct_image_object(vm, flipped)
}

pub fn silk_image_ascii(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }

    let default_gradient = " .:-=+*#%@".to_string();
    let gradient_str = if args.len() > 1 && !matches!(args[1], SilkValue::Null) {
        vm.heap_get_string(args[1].clone()).unwrap_or(default_gradient)
    } else {
        default_gradient
    };

    let chars: Vec<char> = gradient_str.chars().collect();
    let max_w = if args.len() > 2 { args[2].as_int().filter(|&w| w > 0) } else { None };
    let max_h = if args.len() > 3 { args[3].as_int().filter(|&h| h > 0) } else { None };

    if let Some(img) = extract_image(vm, &args[0]) {
        let (orig_w, orig_h) = (img.width() as f32, img.height() as f32);
        let font_aspect_ratio = 0.5f32;

        let (target_w, target_h) = match (max_w, max_h) {
            (Some(mw), Some(mh)) => (mw as u32, mh as u32),
            (Some(mw), None) => (mw as u32, ((mw as f32) * (orig_h / orig_w) * font_aspect_ratio).round() as u32),
            (None, Some(mh)) => (((mh as f32) * (orig_w / orig_h) / font_aspect_ratio).round() as u32, mh as u32),
            (None, None) => (img.width(), img.height()),
        };

        let processed_img = img.resize_exact(target_w.max(1), target_h.max(1), FilterType::Triangle);
        let gray_img = processed_img.to_luma8();
        let (w, h) = gray_img.dimensions();
        let mut ascii_art = String::new();

        for y in 0..h {
            for x in 0..w {
                let pixel_val = gray_img.get_pixel(x, y)[0] as usize;
                let idx = (pixel_val * (chars.len() - 1)) / 255;
                ascii_art.push(chars[idx]);
            }
            ascii_art.push('\n');
        }

        SilkValue::String(ascii_art)
    } else {
        SilkValue::Null
    }
}

pub fn build_image_map() -> HashMap<String, SilkValue> {
    let mut map = HashMap::new();
    map.insert(
        "Image".to_string(),
        SilkValue::NativeFn(
            silk_image,
            String::from("Image(path: String = '') -> Image; Opens or instantiates an image."),
        ),
    );
    map
}