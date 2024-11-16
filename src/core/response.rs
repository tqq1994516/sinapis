use axum::{
    http::{
        StatusCode,
        header,
        HeaderValue
    },
    response::{
        IntoResponse,
        Response
    },
    body::{
        self,
        Full
    }
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use std::fmt::Debug;

#[derive(Debug, Serialize)]
/// 查 数据返回
pub struct ListData<T> {
    pub list: Vec<T>,
    pub total: u64,
    pub total_pages: u64,
    pub page_num: u64,
}
/// 分页参数
#[derive(Deserialize, Clone, Debug, Serialize, Default)]
pub struct PageParams {
    pub current: Option<u64>,
    pub page_size: Option<u64>,
}

pub enum ErrorShowType {
    Silent = 0,
    WarnMessage = 1,
    ErrorMessage = 2,
    Notification = 3,
    Redirect = 9,
}

pub enum ErrorCodeType {
    Success = 0,
    Fail = -1,
}

/// 数据统一返回格式
#[derive(Debug, Serialize, Default, ToSchema)]
pub struct Res<T> {
    pub data: Option<T>,
    pub success: bool,
    pub error_message: Option<String>,
    pub error_code: Option<i8>,
    pub show_type: u8,
}

/// 填入到extensions中的数据
// #[derive(Debug)]
// pub struct ResJsonString(pub String);
#[allow(unconditional_recursion)]
impl<T> IntoResponse for Res<T>
where
    T: Serialize + Send, 
{
    fn into_response(self) -> Response {
        let res_string = match serde_json::to_string(&self) {
            Ok(v) => v,
            Err(e) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()))
                    .body(body::boxed(Full::from(e.to_string())))
                    .unwrap();
            }
        };
        // let res_json_string = ResJsonString(res_string.clone());
        let response = res_string.into_response();
        // response.extensions_mut().insert(res_json_string);
        response
    }
}

impl<T: Serialize> Res<T> {
    pub fn with_data(data: T) -> Self {
        Self {
            data: Some(data),
            success: true,
            error_message: Some("success".to_string()),
            error_code: Some(0),
            show_type: 3,
        }
    }
    pub fn with_err(err: &str, show_type: ErrorShowType) -> Self {
        let _show_type = Self::show_type_number(show_type);
        Self {
            data: None,
            success: false,
            error_message: Some(err.to_string()),
            error_code: Some(-1),
            show_type: _show_type
        }
    }
    pub fn with_msg(data: Option<T>, msg: &str, success: bool, show_type: Option<ErrorShowType>, error_code: Option<ErrorCodeType>) -> Self {
        let _show_type = match show_type {
            Some(t) => Self::show_type_number(t),
            None => 0,
        };
        match error_code {
            Some(c) => {
                Self {
                    data,
                    success,
                    error_message: Some(msg.to_string()),
                    error_code: Some(Self::error_code_number(c)),
                    show_type: _show_type
                }
            },
            None => {
                Self {
                    data,
                    success,
                    error_message: Some(msg.to_string()),
                    error_code: None,
                    show_type: _show_type
                }
            },
        }
    }
    // #[allow(dead_code)]
    // pub fn with_data_msg(data: T, msg: &str) -> Self {
    //     Self {
    //         code: 0,
    //         data: Some(data),
    //         message: msg.to_string(),
    //     }
    // }
    pub fn show_type_number(show_type: ErrorShowType) -> u8 {
        match show_type {
            ErrorShowType::Silent => 0,
            ErrorShowType::WarnMessage => 1,
            ErrorShowType::ErrorMessage => 2,
            ErrorShowType::Notification => 3,
            ErrorShowType::Redirect => 9,
        }
    }
    pub fn error_code_number(error_code: ErrorCodeType) -> i8 {
        match error_code {
            ErrorCodeType::Success => 0,
            ErrorCodeType::Fail => -1,
        }
    }
}
