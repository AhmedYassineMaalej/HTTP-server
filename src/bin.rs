use http_server::{
    headers::Headers,
    request::Request,
    response::{ResponseWriter, StatusCode, StatusLine},
    server::{HandlerError, Server},
};

fn main() {
    let Ok(server) = Server::new(8080) else {
        println!("couldnt not create server at port 8080");
        return;
    };

    server.serve(handler).unwrap();
}

fn handler(writer: &mut ResponseWriter, _request: Request) -> Result<(), HandlerError> {
    writer
        .write_status_line(&StatusLine::from(StatusCode::PermanentRedirect))
        .map_err(|_| HandlerError::IntervalServerError)?;

    let mut headers = Headers::new();
    headers.insert("Content-Type", String::from("application/text"));

    writer
        .write_headers(&headers)
        .map_err(|_| HandlerError::IntervalServerError)?;

    let date = chrono::Local::now();

    if let Err(_e) = writer.write_body(
        date.format("it is %Y-%m-%d %H:%M:%S")
            .to_string()
            .as_bytes(),
    ) {
        return Err(HandlerError::IntervalServerError);
    }

    Ok(())
}
