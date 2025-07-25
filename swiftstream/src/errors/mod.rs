// From lcsm-server project
#[macro_export]
macro_rules! something_with_error_log {
    ($status_code:expr) => {
        |e| {
            use log::error;

            error!("{}", e);
            $status_code
        }
    };
}

#[macro_export]
macro_rules! internal_error_with_log {
    () => {{
        use axum::http::StatusCode;
        $crate::something_with_error_log!(StatusCode::INTERNAL_SERVER_ERROR)
    }};
}

#[macro_export]
macro_rules! bad_request_with_log {
    () => {{
        use axum::http::StatusCode;
        $crate::something_with_error_log!(StatusCode::BAD_REQUEST)
    }};
}

pub use bad_request_with_log;
pub use internal_error_with_log;
pub use something_with_error_log;
