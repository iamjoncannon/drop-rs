use std::thread;

use httpmock::prelude::*;

fn main() {
    let server = MockServer::start();

    let _signup = server.mock(|when, then| {
        when.path("/signup")
            .method("POST")
            .header("cloud_key", "my cloud key header");
        then.status(200);
    });

    let _login = server.mock(|when, then| {
        when.path("/login").method("POST");
        then.status(200).body("{ \"token\": \"secret_auth_token\"}");
    });

    let _get_bananas = server.mock(|when, then| {
        when.path("/banana")
            .method("GET")
            .header("auth", "secret_auth_token")
            .header("cloud_key", "my cloud key header");
        then.status(200);
    });

    println!("server url port: {:?}", server.port());

    // let response = get(server.url("/hello/standalone")).unwrap();
    // assert_eq!(response.status(), 200);

    thread::park();
}
