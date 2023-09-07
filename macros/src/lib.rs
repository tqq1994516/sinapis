extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_attribute]
pub fn system_info(_: TokenStream, input: TokenStream) -> TokenStream {
    // 解析输入的结构体
    let input = parse_macro_input!(input as DeriveInput);
    let table_name = &input.ident.to_string();

    // 获取结构体字段信息
    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // 生成添加默认字段的新结构体定义
    let expanded = quote! {
        #[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = #table_name)]
        pub struct Model {
            #fields
            #[sea_orm(default = OffsetDateTime::now_local(), indexed)]
            pub create_time: OffsetDateTime,
            #[sea_orm(default = OffsetDateTime::now_local(), indexed)]
            pub update_time: OffsetDateTime,
            pub comment: String,
            pub owner: i64,
        }
    };

    // 返回生成的代码作为 TokenStream
    TokenStream::from(expanded)
}
