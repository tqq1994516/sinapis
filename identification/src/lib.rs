wit_bindgen::generate!({ generate_all });

use wasmcloud::postgres::{
    query::query,
    types::{PgValue, ResultRow, ResultRowEntry},
};
use wasmcloud_component::wasi::keyvalue::*;
use exports::sinapis::identification::invoke::Guest;


struct Identification;

impl Guest for Identification {
    #[doc = " Account identification"]
    fn identification(
        account: String,
        password: String,
    ) -> String {
        let sql = r#"
            SELECT verify_password($1, $2);
        "#;
        match query(&sql, &[PgValue::Text(account.into()), PgValue::Text(password.into())]) {
            Ok(record) => {
                let Some(ResultRowEntry { value: accessed, .. }) = record
                    .first()
                    .and_then(|r| r.first())
                else {
                    return "ERROR: returned row is missing an id column".into();
                };
                match accessed {
                    PgValue::Bool(access) => {
                        let bucket = store::open("default")
                            .map_err(|e| format!("error:{e}")).expect("error");
                        bucket.set("11", b"11");
                        format!("accessed, {access:#?}")
                    },
                    PgValue::Boolean(access) => format!("accessed, {access:#?}"),
                    _ => format!("failed, {:#?}", accessed)
                }
            },
            Err(e) => format!("login error, {}", e),
        }
    }
}
