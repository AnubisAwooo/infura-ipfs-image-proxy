use salvo::prelude::*;

async fn get(
    id: String,
    width: Option<u32>,
    height: Option<u32>,
    headers: &mut http::header::HeaderMap,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response =
        reqwest::get("https://xxx.infura-ipfs.io/ipfs/{id}".replace("{id}", &id)).await?;

    // println!("Status: {}", response.status());

    let map = response.headers();
    for (name, value) in map.iter() {
        headers.insert(name, value.clone());
    }

    let body = response.bytes().await?;

    // println!("Body:\n\n{:?}", body);

    // println!("Response: {:?}", body.len());

    let dynamic = image::load_from_memory(&body.to_vec()).unwrap();
    let dynamic = dynamic.thumbnail(
        width.unwrap_or(dynamic.width()),
        height.unwrap_or(dynamic.height()),
    );
    let mut cursor = std::io::Cursor::new(Vec::new());
    dynamic
        .write_to(&mut cursor, image::ImageOutputFormat::Png)
        .unwrap();
    let bytes = cursor.into_inner();

    // println!("bytes: {:?}", bytes.len());

    Ok(bytes)
}

#[handler]
async fn do_proxy(req: &mut Request, resp: &mut Response) {
    let path = req.uri().path();
    // println!("path: {:?}", path);
    let query = req.uri().query().unwrap_or("");
    // println!("query: {:?}", query);

    let mut width: Option<u32> = None;
    if query.contains("width=") {
        let mut splits = query.split("width=");
        splits.next();
        let s = splits.next().unwrap().split("&").next().unwrap();
        width = Some(s.parse::<u32>().unwrap());
    }
    let mut height: Option<u32> = None;
    if query.contains("height=") {
        let mut splits = query.split("height=");
        splits.next();
        let s = splits.next().unwrap().split("&").next().unwrap();
        height = Some(s.parse::<u32>().unwrap());
    }

    let id = path.replace("/ipfs/", "");

    let bytes = get(id, width, height, resp.headers_mut()).await.unwrap();

    resp.add_header("Content-Length", bytes.len(), true)
        .unwrap();

    resp.write_body(bytes).unwrap();
}

#[tokio::main]
async fn main() {
    let router = Router::with_path("ipfs/<id>").get(do_proxy);
    Server::new(TcpListener::bind("0.0.0.0:6025"))
        .serve(router)
        .await;
}
