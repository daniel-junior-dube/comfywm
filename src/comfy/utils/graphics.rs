pub mod texture_helper {
	use image::{DynamicImage, GenericImageView};
	use std::path::Path;
	use wlroots::{GenericRenderer, Texture, TextureFormat};

	pub fn load_texture(renderer: &mut GenericRenderer, image_path: &Path) -> Result<Texture<'static>, String> {
		// ? Load image from disk
		// * IDEA: Make an executable to convert the image into a supported format, would lighten comfy's binary
		let wallpaper_image_result = image::open(image_path);
		if wallpaper_image_result.is_err() {
			return Err("Failed to open the wallpaper!".to_string());
		}
		let wallpaper_image = wallpaper_image_result.unwrap();
		// ? Detect texture format from image format (ex: ARGB888)
		let texture_format = match wallpaper_image {
			DynamicImage::ImageRgb8(_) => TextureFormat::RGB888,
			DynamicImage::ImageRgba8(_) => TextureFormat::RGBA8888,
			DynamicImage::ImageBgr8(_) => TextureFormat::BGR888,
			DynamicImage::ImageBgra8(_) => TextureFormat::BGRA8888,
			_ => TextureFormat::BGRA8888,
		};
		// ? Get image dimensions
		let (width, height) = wallpaper_image.dimensions();
		// ? Extract raw_pixels from image
		let mut raw_pixels = wallpaper_image.raw_pixels();
		match texture_format {
			TextureFormat::RGBA8888 => {}
			_ => convert_to_rgba(&mut raw_pixels, &texture_format),
		}

		// ? create_texture_from_pixels using appropriate format
		let wallpaper_texture_option =
			renderer.create_texture_from_pixels(TextureFormat::ABGR8888.into(), width * 4, width, height, &raw_pixels);
		match wallpaper_texture_option {
			Some(wallpaper_texture) => Ok(wallpaper_texture),
			None => Err("Failed to create texture from image pixels (wallpaper)".to_string()),
		}
	}

	fn convert_to_rgba(data: &mut Vec<u8>, format: &TextureFormat) {
		let data_size = data.len();
		match format {
			TextureFormat::RGB888 => {
				let stride = 3;
				let mut new_data = Vec::<u8>::with_capacity(data_size + data_size / stride);
				for i in 0..(data_size / stride) {
					new_data.push(data[i * stride]);
					new_data.push(data[i * stride + 1]);
					new_data.push(data[i * stride + 2]);
					new_data.push(std::u8::MAX);
				}
				*data = new_data;
			}
			TextureFormat::RGBA8888 => {
				warn!("Tried to convert pixel buffer to rgba, but it's already rgba.");
			}
			_ => {
				error!("Could not convert wallpaper format to ARGB!");
			}
		}
	}
}
