// //! project constants

// // blocks
// /// Identifier for the "get" block.
pub const GET_BLOCK_IDENTIFIER: &str = "get";

// /// Identifier for the "post" block.
pub const POST_BLOCK_IDENTIFIER: &str = "post";

// /// Identifier for the "patch" block.
pub const PATCH_BLOCK_IDENTIFIER: &str = "patch";

// /// Identifier for the "environment" block.
pub const ENVIRONMENT_BLOCK_IDENTIFIER: &str = "environment";

// /// Title for the base environment.
pub const BASE_ENVIRONMENT_TITLE: &str = "base";

// /// call block properties \\\
// /// Property key for the base URL in a call block.
pub const CALL_BASE_URL: &str = "base_url";
// /// Property key for the body in a call block.
pub const CALL_BODY: &str = "body";
// /// Property key for the "after" action in a call block.
pub const CALL_AFTER: &str = "after";
// /// Property key for the path in a call block.
pub const CALL_PATH: &str = "path";
// /// Property key for the headers in a call block.
pub const CALL_HEADERS: &str = "headers";
// /// Property key for the outputs in a call block.
pub const CALL_OUTPUT: &str = "outputs";
// /// Property key for asserts in a call block.
pub const CALL_ASSERT: &str = "assert";

// /// internal variables \\\
// /// Property key for the response in internal variables.
pub const RESPONSE_PROP: &str = "response";

// /// Property key for the body in internal variables.
pub const BODY_PROP: &str = "body";

// /// Prefix for environment object variables.
pub const ENV_OBJECT_VAR_PREFIX: &str = "env";

// /// Prefix for module object variables.
pub const MOD_OBJECT_VAR_PREFIX: &str = "mod";

// /// Prefix for secret object variables.
pub const SECRET_OBJECT_VAR_PREFIX: &str = "secrets";

// /// Prefix for input object variables.
pub const INPUT_OBJECT_VAR_PREFIX: &str = "inputs";

// // auth
// /// Bearer key for authentication.
pub const AUTH_BEARER_KEY: &str = "Bearer";

// /// Property key for the token key in authentication.
pub const AUTH_TOKEN_KEY: &str = "token_key";

// /// Property key for the authorization in authentication.
pub const AUTH_AUTHORIZATION_KEY: &str = "authorization";

// // post action - set secret
// /// Type value for the "`set_secret`" action in post actions.
pub const AFTER_SET_SECRET_TYPE_VALUE: &str = "set_secret";

// /// Property key for the target in the "`set_secret`" action.
pub const SET_ENV_TARGET_PROP: &str = "target";

// /// Property key for the variable in the "`set_secret`" action.
pub const SET_ENV_VARIABLE_PROP: &str = "variable";

// /// Property key for the output in the "`set_secret`" action.
pub const SET_ENV_OUTPUT_PROP: &str = "output";

// // other
// /// Property key for the type in post actions.
pub const TYPE_PROP: &str = "type";

// // post actions
// /// Label for the "`set_secret`" post action.
pub const SET_POST_ACTION_LABEL: &str = "set_secret";

// /// Property key for the type in post actions.
pub const POST_ACTION_TYPE: &str = "type";
