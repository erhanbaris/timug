use serde::{Deserialize, Serialize};
use worker::*;

static KV_NAME: &str = "timug-page-infos";

#[derive(Deserialize, Serialize, Debug)]
struct JsonResponse {
    status: bool,
    value: Option<PageInfo>,
}

fn build_headers(json: JsonResponse) -> Result<Response> {
    let mut response = Response::from_json(&json)?;
    let headers = response.headers_mut();
    for header in CORS_HEADERS.iter() {
        headers.set(header[0], header[1])?;
    }
    Ok(response)
}

impl JsonResponse {
    pub fn success(info: PageInfo) -> Result<Response> {
        build_headers(Self {
            status: true,
            value: Some(info),
        })
    }

    pub fn bad_request() -> Result<Response> {
        build_headers(Self {
            status: false,
            value: None,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
struct PageInfo {
    likes: u32,
    views: u32,
}

const SEARCH_HEADERS: [&str; 3] = [
    "access-control-request-method",
    "access-control-request-headers",
    "origin",
];

const CORS_HEADERS: [[&str; 2]; 3] = [
    ["Access-Control-Allow-Origin", "*"],
    ["Access-Control-Allow-Methods", "GET,HEAD,POST,OPTIONS"],
    ["Access-Control-Max-Age", "86400"],
];

async fn handle_options(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let headers: Vec<_> = req.headers().keys().collect();
    if SEARCH_HEADERS
        .iter()
        .all(|i| headers.contains(&i.to_string()))
    {
        let mut headers = Headers::new();
        for header in CORS_HEADERS.iter() {
            headers.set(header[0], header[1])?;
        }
        return Ok(Response::empty()?.with_headers(headers));
    }
    Response::empty()
}

async fn handle_get(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(page) = ctx.param("page") {
        let mut info = ctx
            .kv(KV_NAME)?
            .get(page)
            .json::<PageInfo>()
            .await?
            .unwrap_or_default();
        info.views += 1;
        let _ = ctx.kv(KV_NAME)?.put(page, info.clone())?.execute().await;
        return JsonResponse::success(info);
    }

    JsonResponse::bad_request()
}

async fn handle_post(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(page) = ctx.param("page") {
        let mut info = (ctx.kv(KV_NAME)?.get(page).json::<PageInfo>().await?).unwrap_or_default();

        info.likes += 1;
        let _ = ctx.kv(KV_NAME)?.put(page, info.clone())?.execute().await;
        JsonResponse::success(info)
    } else {
        JsonResponse::bad_request()
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/:page", handle_post)
        .options_async("/:page", handle_options)
        .get_async("/:page", handle_get)
        .run(req, env)
        .await
}
