# drop-rs

> This is a development version of the utility. Please post questions in the Github issues.

Declarative cli-based api management utility with strong IaC support. 

Main features:

- api resources and access patterns declared in HCL, the config langauge of Terraform, supporting strong standard CI/CD and devops patterns without compromise or requiring external vendor tooling 

- more complex and powerful environments and modules- leverage many of the features and IaC patterns of terraform- set global scope variables, access module variables in the environment, differentiate environment and secret variables.

- access patterns to endpoints can be defined and socialized- define not just the call, but also different parameters, request bodies, authorization tokens. 

- automatically access and store data from api responses- socialize and reduce friction managing api tokens, crsf_tokens, headers during development ("set it and forget it").

- list, search, evaluate, invoke endpoints via cli dropdown

- persist output of calls to external data store, with the ability to customzie the sink 

- run assertions against api outputs for e2e testing locally and in CI/CD 

## build

1. install cargo, the Rust building toolchain (see https://doc.rust-lang.org/cargo/getting-started/installation.html)

2. build the utility on your machine

```
cargo build --release --bin drop-rs
```

## demo

See "examples" folder for working version of the main features, including defining environments, modules, api calls, and 'runs' (parameterized api calls).

After installing-

#### 1. Run the mock server:

```
cargo run --bin mock 

server url port: 57888
```

#### 2. set port in environment file

```hcl
environment "base" {
    base_url = "http://localhost:57888"
}
```

#### 3. set initial secret defaults

Secrets for an environment must have a default. 

```
./target/release/drop-rs secret set csrf_token null
./target/release/drop-rs secret set secret_auth_token null
```

#### 4. evaluate a mock server endpoint with a call

The following command will list all the api calls ("hits") defined in the 'example' module and give you a dropdown selector to evaluate one in the default environment.

```
./target/release/drop-rs give example      
giving example in environment base

calls in module: example

? Select drop  
> example.post.signup
  example.post.login
  example.get.user
  example.get.user_with_input
  example.run.login_user_2
[↑↓ to move, enter to select, type to filter]
```

select `example.post.signup`

```
giving example in environment base

calls in module: example

> Select drop example.post.signup
post "signup" {
  base_url = "http://localhost:57888"
  path = "/signup"
  body = {
    "user_name" = "example_user_name_1"
    "password" = "example_password"
  }
  after = [
    {
      "type" = "set_secret"
      "input" = response.headers.csrf_token
      "key" = "csrf_token"
      "env" = "base"
      "overwrite" = true
    }
  ]
  inputs = {
    "user_name" = "example_user_name_1"
  }
  assert = {
    response.headers.csrf_token = assert.exist
  }
  outputs = [
    response.headers.csrf_token
  ]
}
```

#### 5. invoke the signup endpoint

```
./target/release/drop-rs hit example

hitting example in environment base

calls in module: example

? Select drop  
> example.post.signup
  example.post.login
  example.get.user
  example.get.user_with_input
  example.run.login_user_2
[↑↓ to move, enter to select, type to filter]
```

```
> Select drop example.post.signup

calling http://localhost:57888/signup

example.post.signup assertions
+-----------------------------+-------+---------+
| response.headers.csrf_token | exist | Success |
+-----------------------------+-------+---------+

"example.post.signup" result: 200


output "response.headers.csrf_token" 
"secret_csrf_token"

secret csrf_token in environment base set successfully.
```

The mock signup call will:
- assert the response from the mock server 
- store the csrf_token response header in your local secret store as "csrf_token."

#### 6. view the secret that the call automatically stored in your local machine 

```
./target/release/drop-rs secret get  

Secrets for env: base
Secret { key: "secret_auth_token", value: "null", _env: "base" }
Secret { key: "csrf_token", value: "secret_csrf_token", _env: "base" }
```

See the example files for a more detailed walkthrough and documentation of the current features.

# discussion

### issues with postman

As your project scales, and you start to manage dozens of microservices and hundreds of endpoints, many issues arise with Postman:

1. difficulty and friction versioning and socializing api definitions on teams 
2. difficulty searching modules for specific calls
3. inability to define and socialize access patterns
4. lack of variable scope between environments and modules
5. fiction managing data interfaces between calls

### vs bruno

This project was inspired by Bruno, a popular new api management tool that is config based and open source.

What we love about Bruno:

1. True IaC/devop workflow to reduce friction in socializing endpoints during development process  
2. open source
3. strong dx via VsCode extensions
4. strong cli support alongside ui 

Some differences:

1. Rust vs. JS/npm ecosystem- drop is compiled for portability
2. Rather than a custom DSL, we use hcl, the fully featured, mature configuration language of terraform with very strong Rust support. Users familiar with terraform can start coding immediately. 
3. Variable scope- hcl allows us to define and access complex variable contexts between modules and environments. 
4. hcl functions- hcl supports ergonomic string manipulation and formatting directly in the code
5. 'run' blocks- drop supports encapsulating access patterns
6. drop does not and will never have a ui 

### why rust?

1. portability- compiled, requires no external dependencies to use locally or in CI/CD 
2. reliability- modern type and null safety  
3. memory safe concurrency
4. strong embedded Python support
