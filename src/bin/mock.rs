use std::thread;

use httpmock::prelude::*;

fn main() {
    let server = MockServer::start();

    let _example_mock = server.mock(|when, then| {
        when.path("/some_path");
        then.status(200).body("some body");
    });

    let _hello_mock = server.mock(|when, then| {
        when.path("/hello/standalone")
            .header("auth", "valid_dev_auth");
        then.status(200);
    });

    println!("server url port: {:?}", server.port());

    // let response = get(server.url("/hello/standalone")).unwrap();
    // assert_eq!(response.status(), 200);

    thread::park();
}
