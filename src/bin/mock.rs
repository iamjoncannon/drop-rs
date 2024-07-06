use httpmock::prelude::*;
use isahc::get;

fn main() {
    let server = MockServer::start();

    let hello_mock = server.mock(|when, then| {
        when.path("/hello/standalone")
            .header("auth", "valid_dev_auth");
        then.status(200);
    });

    println!("server url: {:?}", server.url("/hello/standalone"));

    let response = get(server.url("/hello/standalone")).unwrap();

    assert_eq!(response.status(), 200);
}
