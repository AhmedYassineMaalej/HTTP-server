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

    server.serve(handler).unwrap();
}

fn handler(writer: &mut ResponseWriter, request: Request) -> Result<(), HandlerError> {
    dbg!(request);

    writer
        .write_status_line(&StatusLine::from(StatusCode::Ok))
        .map_err(|_| HandlerError::IntervalServerError)?;

    let headers = Headers::new();
    writer
        .write_headers(&headers)
        .map_err(|_| HandlerError::IntervalServerError)?;

    if let Err(_e) = writer.write_chunked_body(b"Hello world") {
        return Err(HandlerError::IntervalServerError);
    }

    Ok(())
}
