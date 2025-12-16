use jsonrpsee::{
    core::{RpcResult, SubscriptionResult},
    proc_macros::rpc,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use openrpsee::{Contact, Project, openrpc};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Bar {
    baz: u32,
    quox: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct FooRes {
    success: bool,
    num: Option<u32>,
    bar: Bar,
    any: serde_json::Value,
}

#[openrpc(namespace = "myrpc", tag = "public")]
#[rpc(server, namespace = "myrpc")]
pub trait MyRpc {
    /// # This method does the foo!
    ///
    /// And it does it really well!
    #[method(name = "foo", aliases = ["nonamespacefoo", "oldnamespace_foo"])]
    async fn foo(&self, limit: Option<u32>, any: serde_json::Value) -> RpcResult<FooRes>;

    /// This is a subscription
    #[subscription(name = "subscribe_bar", item = Bar)]
    async fn subscribe_bar(&self, idx: u32) -> SubscriptionResult;
}

fn main() {
    let mut project = Project::builder("My JSON-RPC API".into(), "0.1.0".into());
    project
        .description("API for interaction with my beautiful backend".into())
        .license("Apache-2.0".into())
        .contact(Contact {
            name: "FooMaker".into(),
            email: Some("foomaker@example.com".into()),
            ..Default::default()
        });

    let mut project = project.build();

    project.add_module(MyRpcOpenRpc::module_doc());

    println!("{}", serde_json::to_string_pretty(&project).unwrap());
}
