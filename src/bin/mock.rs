use std::thread;

use httpmock::prelude::*;

fn main() {
    let server = MockServer::start();

    let _signup = server.mock(|when, then| {
        when.path("/signup").method("POST");
        then.status(200).header("csrf_token", "secret_csrf_token");
    });

    let _login_1 = server.mock(|when, then| {

        when.path("/login")
            .method("POST")
            .header("user_name", "example_user_name_1")
            .header("user_password", "example_password")
            .header("csrf_token", "secret_csrf_token");

        then.status(200).body("{ \"token\": \"secret_auth_token\", \"user_id\": 42 }");
    });

    let _login_2 = server.mock(|when, then| {

        when.path("/login")
            .method("POST")
            .header("user_name", "example_user_name_2")
            .header("user_password", "example_password")
            .header("csrf_token", "secret_csrf_token");

        then.status(200).body("{ \"token\": \"secret_auth_token\", \"user_id\": 88 }");
    });

    let _get_user = server.mock(|when, then| {

        when.path("/user")
            .method("GET")
            .header("auth", "secret_auth_token")
            .header("csrf_token", "secret_csrf_token");
        
        then.status(200).body("{ \"user_id\": 42, \"user_name\": \"example_user_name\", \"user_password\": \"example_password\" }");
    });

    println!("server url port: {:?}", server.port());

    // let response = get(server.url("/hello/standalone")).unwrap();
    // assert_eq!(response.status(), 200);

    thread::park();
}
