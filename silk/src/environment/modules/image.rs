use std::collections::HashMap;
use crate::environment::vm::VirtualMachine;
use super::super::value::SilkValue;
use image::{DynamicImage, GenericImageView, GenericImage, RgbImage, Rgba, imageops::FilterType};

// @export Modules/Image
/*
    The Image module provides robust capabilities for loading, manipulating, and saving image files. 
    It leverages an internal object-oriented structure where image instances expose various transformation and data extraction methods.
*/

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

// @export Modules/Image#Image
/*
    <b>Signature</b>
    <code>Image(path: String = "") -> Image</code>

    <p>Image Constructor. Opens an existing image from a file path or creates a 1x1 blank image if no path is provided.</p>

    <b>Parameters:</b>
    - <code>path</code>: (Optional) The string file path to the image you want to load.

    <b>Returns:</b>
    - <code>Image</code>: A new instance of an Image object containing methods for manipulation.

    <b>Usage:</b>
    <pre><code>var img = Image("photo.jpg")
var blank_img = Image()</code></pre>
*/
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

// @export Modules/Image#Image.width
/*
    <b>Signature</b>
    <code>Image.width() -> Int</code>

    <p>Retrieves the width of the image in pixels.</p>

    <b>Returns:</b>
    - <code>Int</code>: The width of the image.
*/
pub fn silk_image_width(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    extract_image(vm, &args[0])
        .map(|img| SilkValue::Int(img.width() as i32))
        .unwrap_or(SilkValue::Null)
}

// @export Modules/Image#Image.height
/*
    <b>Signature</b>
    <code>Image.height() -> Int</code>

    <p>Retrieves the height of the image in pixels.</p>

    <b>Returns:</b>
    - <code>Int</code>: The height of the image.
*/
pub fn silk_image_height(vm: &mut VirtualMachine, args: &Vec<SilkValue>) -> SilkValue {
    if args.is_empty() { return SilkValue::Null; }
    extract_image(vm, &args[0])
        .map(|img| SilkValue::Int(img.height() as i32))
        .unwrap_or(SilkValue::Null)
}

// @export Modules/Image#Image.save
/*
    <b>Signature</b>
    <code>Image.save(path: String) -> Bool</code>

    <p>Saves the current state of the image to the specified file path on disk.</p>

    <b>Parameters:</b>
    - <code>path</code>: The destination file path (e.g., "output.png").

    <b>Returns:</b>
    - <code>Bool</code>: <code>true</code> if the save was successful, <code>false</code> otherwise.

    <b>Usage:</b>
    <pre><code>var img = Image("photo.jpg")
img.save("copy.png")</code></pre>
*/
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

// @export Modules/Image#Image.get_pixel
/*
    <b>Signature</b>
    <code>Image.get_pixel(x: Int, y: Int) -> List</code>

    <p>Gets the RGBA color values of a specific pixel at the given coordinates.</p>

    <b>Parameters:</b>
    - <code>x</code>: The horizontal coordinate (0-indexed).
    - <code>y</code>: The vertical coordinate (0-indexed).

    <b>Returns:</b>
    - <code>List</code>: A list containing 4 integers representing the Red, Green, Blue, and Alpha channels <code>[r, g, b, a]</code>. Returns null if coordinates are out of bounds.

    <b>Usage:</b>
    <pre><code>var color = img.get_pixel(10, 10)
print(color) # => [255, 0, 0, 255]</code></pre>
*/
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

// @export Modules/Image#Image.set_pixel
/*
    <b>Signature</b>
    <code>Image.set_pixel(x: Int, y: Int, r: Int, g: Int, b: Int, a: Int = 255) -> Null</code>

    <p>Modifies the image in-place by setting a specific pixel to a designated RGBA color.</p>

    <b>Parameters:</b>
    - <code>x</code>: The horizontal coordinate.
    - <code>y</code>: The vertical coordinate.
    - <code>r</code>: Red channel value (0-255).
    - <code>g</code>: Green channel value (0-255).
    - <code>b</code>: Blue channel value (0-255).
    - <code>a</code>: (Optional) Alpha channel value (0-255). Defaults to 255.
*/
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

// @export Modules/Image#Image.crop
/*
    <b>Signature</b>
    <code>Image.crop(x: Int, y: Int, w: Int, h: Int) -> Image</code>

    <p>Extracts a rectangular portion of the image and returns it as a new Image object.</p>

    <b>Parameters:</b>
    - <code>x</code>: The starting horizontal coordinate.
    - <code>y</code>: The starting vertical coordinate.
    - <code>w</code>: The width of the cropped area.
    - <code>h</code>: The height of the cropped area.

    <b>Returns:</b>
    - <code>Image</code>: A new cropped Image instance.
*/
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

// @export Modules/Image#Image.resize
/*
    <b>Signature</b>
    <code>Image.resize(w: Int, h: Int) -> Image</code>

    <p>Resizes the image exactly to the specified dimensions using a triangle filter.</p>

    <b>Parameters:</b>
    - <code>w</code>: The new target width.
    - <code>h</code>: The new target height.

    <b>Returns:</b>
    - <code>Image</code>: A new resized Image instance.
*/
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

// @export Modules/Image#Image.grayscale
/*
    <b>Signature</b>
    <code>Image.grayscale() -> Image</code>

    <p>Converts the image to grayscale.</p>

    <b>Returns:</b>
    - <code>Image</code>: A new grayscale Image instance.
*/
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

// @export Modules/Image#Image.invert
/*
    <b>Signature</b>
    <code>Image.invert() -> Image</code>

    <p>Inverts all colors in the image (e.g., creating a negative effect).</p>

    <b>Returns:</b>
    - <code>Image</code>: A new inverted Image instance.
*/
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

// @export Modules/Image#Image.flip_h
/*
    <b>Signature</b>
    <code>Image.flip_h() -> Image</code>

    <p>Flips the image horizontally (mirrors left to right).</p>

    <b>Returns:</b>
    - <code>Image</code>: A new horizontally flipped Image instance.
*/
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

// @export Modules/Image#Image.flip_v
/*
    <b>Signature</b>
    <code>Image.flip_v() -> Image</code>

    <p>Flips the image vertically (upside down).</p>

    <b>Returns:</b>
    - <code>Image</code>: A new vertically flipped Image instance.
*/
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

// @export Modules/Image#Image.ascii
/*
    <b>Signature</b>
    <code>Image.ascii(gradient: String = " .:-=+*#%@", max_w: Int = null, max_h: Int = null) -> String</code>

    <p>Converts the image into an ASCII art string based on pixel luminance.</p>

    <b>Parameters:</b>
    - <code>gradient</code>: (Optional) A string of characters ordered from darkest to lightest. Defaults to <code>" .:-=+*#%@"</code>.
    - <code>max_w</code>: (Optional) Maximum width in characters for the output. Aspect ratio is preserved if height is omitted.
    - <code>max_h</code>: (Optional) Maximum height in characters for the output.

    <b>Returns:</b>
    - <code>String</code>: The generated ASCII art text.

    <b>Usage:</b>
    <pre><code>var my_art = img.ascii("@%#*+=-:. ", 80)
print(my_art)</code></pre>
*/
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