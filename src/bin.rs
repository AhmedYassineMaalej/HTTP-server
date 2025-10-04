use http_server::{
    headers::Headers,
    request::Request,
    response::{ResponseWriter, StatusCode, StatusLine},
    server::{HandlerError, Server},
};

fn main() {
    let Ok(server) = Server::new(8080) else {
        println!("couldnt not create server at port");
        return;
    };

    server.serve(handler);
}

fn handler(writer: &mut ResponseWriter, request: Request) -> Result<(), HandlerError> {
    writer.write_status_line(&StatusLine::from(StatusCode::Ok));
    writer.write_headers(&Headers::get_default(0));
    writer.write_chunked_body(b"Hello world");
    Ok(())
}
