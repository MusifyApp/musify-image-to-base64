use url::Url;
use worker::*;
use base64;

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

    // convert image to base64

    let base64_image = base64::encode(&image_data);

    let mut headers = Headers::new();
    // OK reponse with base64 image
    Ok(Response::ok(base64_image)?)
}
