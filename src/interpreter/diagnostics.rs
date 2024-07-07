use colored::Colorize;
use derive_getters::Getters;
use hcl::eval::Errors;
use log::trace;

#[derive(Debug, Getters)]
pub struct EvalDiagnostics {
    errors: Option<Vec<hcl::eval::Error>>,
    file_name: String,
    has_input_error: bool,
    has_secret_error: bool,
}

impl EvalDiagnostics {
    pub fn new(file_name: &str) -> EvalDiagnostics {
        EvalDiagnostics {
            errors: None,
            file_name: file_name.to_string(),
            has_input_error: false,
            has_secret_error: false,
        }
    }

    pub fn is_err(&self) -> bool {
        self.errors.is_some()
    }

    #[log_attributes::log(trace, "exit {fn}")]
    pub fn evaluate_errors(&mut self, errors: &Errors) {
        log::trace!("EvalDiagnostics evaluate_errors errors {errors:?}");

        let mut errors_surfaced_to_caller_for_handling = Vec::<hcl::eval::Error>::new();
        let mut errors_to_panic_now = Vec::<hcl::eval::Error>::new();

        let mut total_errs = errors.len();

        for error in errors {
            log::trace!(
                "EvalDiagnostics evaluate_errors error {}",
                error.to_string()
            );

            let message = error.to_string();

            // todo- handle variable error for
            // hit drop id in run

            if message.contains("assert.") {
                total_errs -= 1;
                continue;
            }

            if message.contains("response.") {
                total_errs -= 1;
                continue;
            }

            if message.contains("secrets.") {
                self.has_secret_error = true;
                errors_surfaced_to_caller_for_handling.push(error.to_owned());
                continue;
            }

            if message.contains("inputs.") {
                self.has_input_error = true;
                errors_surfaced_to_caller_for_handling.push(error.to_owned());
                continue;
            }

            if message.contains("undefined variable `secret`") {
                self.print_input_secret_helpers();
            }

            if message.contains("undefined variable `input`") {
                self.print_input_secret_helpers();
            }

            trace!("evaluate_errors error message {message}");
            errors_to_panic_now.push(error.to_owned());
        }

        let should_panic_now = total_errs > errors_surfaced_to_caller_for_handling.len();

        if should_panic_now {
            self.errors = Some(errors_to_panic_now);

            self.panic();
        } else {
            if !errors_surfaced_to_caller_for_handling.is_empty() {
                self.errors = Some(errors_surfaced_to_caller_for_handling);
            }
        }
    }

    pub fn panic(&self) {
        match &self.errors {
            None => {}
            Some(errors) => {
                self.print_input_calltime_warnings();
                self.print_secret_calltime_warnings();

                log::trace!("EvalDiagnostics evaluate_errors panic {errors:?}");

                errors.iter().for_each(|error| {
                    let message = error.to_string();

                    log::trace!("");
                    log::error!(
                        "Error evaluating environment variables in file {}:\n{message}\n",
                        self.file_name.yellow()
                    );
                });

                log::trace!("panic");
                std::process::exit(1);
            }
        }
    }

    fn print_input_secret_helpers(&self) {
        match &self.errors {
            None => {}
            Some(errors) => {
                for error in errors {
                    let message = error.to_string();

                    if message.contains("undefined variable `input`") {
                        println!("\nYou probably meant 'inputs'?");
                    }

                    if message.contains("undefined variable `secret`") {
                        println!("\nYou probably meant 'secrets'?");
                    }
                }
            }
        }
    }

    pub fn print_input_calltime_warnings(&self) {
        match &self.errors {
            None => {}
            Some(errors) => {
                for error in errors {
                    let message = error.to_string();

                    if message.contains("inputs") {
                        if let hcl::eval::ErrorKind::NoSuchKey(var) = error.kind() {
                            let var = var.to_string();

                            println!(
                                "\t{} in {}\n\tinputs.{} will have to be defined at calltime.\n",
                                "warning--".purple(),
                                self.file_name.yellow(),
                                var.yellow(),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn print_secret_calltime_warnings(&self) {
        match &self.errors {
            None => {}
            Some(errors) => {
                for error in errors {
                    let message = error.to_string();

                    if message.contains("secrets") {
                        if let hcl::eval::ErrorKind::NoSuchKey(var) = error.kind() {
                            let var = var.to_string();

                            println!(
                                "\t{} in {}\n\tsecrets.{} will have to be defined at calltime.\n",
                                "warning--".purple(),
                                self.file_name.yellow(),
                                var.yellow(),
                            );
                        }
                    }
                }
            }
        }
    }
}
