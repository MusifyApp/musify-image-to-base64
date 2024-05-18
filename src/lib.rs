use url::Url;
use worker::*;
use base64;
use image::{imageops::resize, DynamicImage, GenericImageView, ImageOutputFormat};


use console_error_panic_hook::set_once as set_panic_hook;

#[event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    console_log!("{} - [{}]", Date::now().to_string(), req.path());
    let image_path = req.path()[1..].to_string();

    set_panic_hook();
    
    match handle_render(image_path).await {
        Err(err) => {
            println!("error: {:?}", err);
            Response::error(format!("an unexpected error occurred: {}", err), 500)
        }
        Ok(res) => Ok(res),
    }
}

async fn handle_render(image_url: String) -> Result<Response> {
    console_log!("image: {}", image_url);
    let url = Url::parse(&image_url)
        .map_err(|err| format!("failed to parse URL: {}", err))?;

    let mut res = Fetch::Url(url)
        .send()
        .await
        .map_err(|err| format!("failed to request remote image: {}", err))?;
    if res.status_code() != 200 {
        let body = res.text().await?;
        return Response::error(
            format!("upstream image returned: {}: {}", res.status_code(), body),
            500,
        );
    }
    let image_data = res.bytes().await?;

    // Load the image from the bytes
    let img = match image::load_from_memory(&image_data) {
        Ok(img) => img,
        Err(err) => return Response::error(format!("Failed to load image: {}", err), 500),
    };
    
    // Calculate the new dimensions while maintaining aspect ratio
    let (width, height) = img.dimensions();
    let aspect_ratio = width as f32 / height as f32;
    let new_height = ((256 * 1024) as f32 / aspect_ratio).sqrt() as u32;
    let new_width = (new_height as f32 * aspect_ratio) as u32;
    
    // Resize the image
    let resized_img = resize(&img, new_width, new_height, image::imageops::FilterType::Nearest);
    
    // Write the resized image to a vector in memory
    let dyn_img = DynamicImage::ImageRgba8(resized_img);
    let mut buffer = Vec::new();
    match dyn_img.write_to(&mut buffer, ImageOutputFormat::Jpeg(75)) {
        Ok(_) => (),
        Err(err) => return Response::error(format!("Failed to write image: {}", err), 500),
    };
    
    // Convert the resized image to base64
    let base64_image = base64::encode(&buffer);
    
    let mut headers = Headers::new();
    // OK response with base64 image
    Ok(Response::ok(base64_image)?)
}
