use std::collections::{BTreeMap, btree_map::Entry::Occupied};

use schemars::{JsonSchema, Schema, SchemaGenerator, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use mosaic_openrpsee_derive::openrpc;

pub const OPENRPC_VERSION: &str = "1.3.2";

/// OPEN-RPC documentation following the `OpenRPC` specification <https://spec.open-rpc.org>
/// The implementation is partial, only required fields and subset of optional fields
/// in the specification are implemented.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    openrpc: String,
    info: Info,
    methods: Vec<Method>,
    components: Components,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_docs: Option<ExternalDocs>,
}

#[derive(Debug, Clone)]
pub struct ProjectBuilder {
    inner: Project,
}

impl ProjectBuilder {
    pub fn new(title: String, version: String) -> Self {
        Self {
            inner: Project {
                openrpc: String::from(OPENRPC_VERSION),
                info: Info {
                    title,
                    version,
                    ..Default::default()
                },
                methods: Vec::new(),
                components: Components::default(),
                external_docs: None,
            },
        }
    }

    pub fn description(&mut self, description: String) -> &mut Self {
        self.inner.info.description = Some(description);
        self
    }

    pub fn terms_of_service(&mut self, terms_of_service: String) -> &mut Self {
        self.inner.info.terms_of_service = Some(terms_of_service);
        self
    }

    pub fn contact(&mut self, contact: Contact) -> &mut Self {
        self.inner.info.contact = Some(contact);
        self
    }

    pub fn license(&mut self, license: String) -> &mut Self {
        self.inner.info.license = Some(License {
            name: license,
            url: None,
        });
        self
    }

    pub fn license_with_url(&mut self, license: String, license_url: String) -> &mut Self {
        self.inner.info.license = Some(License {
            name: license,
            url: Some(license_url),
        });
        self
    }

    pub fn external_docs(&mut self, url: String) -> &mut Self {
        self.inner.external_docs = Some(ExternalDocs {
            url,
            ..Default::default()
        });

        self
    }

    pub fn external_docs_with_description(
        &mut self,
        url: String,
        description: String,
    ) -> &mut Self {
        self.inner.external_docs = Some(ExternalDocs {
            url,
            description: Some(description),
        });

        self
    }

    pub fn build(self) -> Project {
        self.inner
    }
}

impl Project {
    pub fn builder(title: String, version: String) -> ProjectBuilder {
        ProjectBuilder::new(title, version)
    }

    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        version: &str,
        title: &str,
        description: &str,
        contact_name: &str,
        url: &str,
        email: &str,
        license: &str,
        license_url: &str,
    ) -> Self {
        let openrpc = "1.3.2".to_string();
        Self {
            openrpc,
            info: Info {
                title: title.to_string(),
                description: Some(description.to_string()),
                contact: Some(Contact {
                    name: contact_name.to_string(),
                    url: Some(url.to_string()),
                    email: Some(email.to_string()),
                }),
                license: Some(License {
                    name: license.to_string(),
                    url: Some(license_url.to_string()),
                }),
                version: version.to_owned(),
                ..Default::default()
            },
            methods: vec![],
            components: Components {
                content_descriptors: Default::default(),
                schemas: Default::default(),
                errors: Default::default(),
            },
            // TODO: set it
            external_docs: None,
        }
    }

    pub fn add_module(&mut self, module: Module) {
        self.methods.extend(module.methods);

        self.methods.sort_by(|m, n| m.name.cmp(&n.name));

        self.components.schemas.extend(module.components.schemas);
        self.components
            .content_descriptors
            .extend(module.components.content_descriptors);
    }

    pub fn add_examples(&mut self, mut example_provider: BTreeMap<String, Vec<ExamplePairing>>) {
        for method in &mut self.methods {
            if let Occupied(entry) = example_provider.entry(method.name.clone()) {
                let examples = entry.remove();
                let param_names = method
                    .params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>();

                // Make sure example's parameters are correct.
                for example in &examples {
                    let example_param_names = example
                        .params
                        .iter()
                        .map(|param| param.name.clone())
                        .collect::<Vec<_>>();
                    assert_eq!(
                        param_names, example_param_names,
                        "Provided example parameters doesn't match the function parameters."
                    );
                }

                method.examples = examples;
            } else {
                println!("No example found for method: {}", method.name);
            }
        }
    }
}

#[derive(Debug)]
pub struct Module {
    methods: Vec<Method>,
    components: Components,
}

pub struct RpcModuleDocBuilder {
    schema_generator: SchemaGenerator,
    methods: BTreeMap<String, Method>,
    content_descriptors: BTreeMap<String, ContentDescriptor>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ContentDescriptor {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "default")]
    required: bool,
    schema: Schema,
    #[serde(skip_serializing_if = "default")]
    deprecated: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Method {
    name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<Tag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_docs: Option<ExternalDocs>,
    params: Vec<ContentDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<ContentDescriptor>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    deprecated: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<Error>,
    #[serde(skip_serializing_if = "default")]
    param_structure: ParamStructure,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    examples: Vec<ExamplePairing>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParamStructure {
    ByName,
    ByPosition,
    #[default]
    Either,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ExamplePairing {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    params: Vec<Example>,
    result: Example,
}

impl ExamplePairing {
    #[must_use]
    pub fn new(name: &str, params: Vec<(&str, Value)>, result: Value) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            summary: None,
            params: params
                .into_iter()
                .map(|(name, value)| Example {
                    name: name.to_string(),
                    summary: None,
                    description: None,
                    value,
                })
                .collect(),
            result: Example {
                name: "Result".to_string(),
                summary: None,
                description: None,
                value: result,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Example {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    value: Value,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Tag {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_docs: Option<ExternalDocs>,
}

impl Tag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            summary: None,
            description: None,
            // TODO: set it
            external_docs: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Info {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    terms_of_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<License>,
    version: String,
}

fn default<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    value == &T::default()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Contact {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct License {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct ExternalDocs {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Error {
    code: i64,
    message: String,
    data: Value,
}

impl Default for RpcModuleDocBuilder {
    fn default() -> Self {
        let schema_generator = SchemaSettings::default()
            .with(|s| {
                s.definitions_path = "#/components/schemas/".into();
            })
            .into_generator();

        Self {
            schema_generator,
            methods: BTreeMap::new(),
            content_descriptors: BTreeMap::new(),
        }
    }
}

impl RpcModuleDocBuilder {
    #[must_use]
    pub fn build(mut self) -> Module {
        Module {
            methods: self.methods.into_values().collect(),
            components: Components {
                content_descriptors: self.content_descriptors,
                schemas: self
                    .schema_generator
                    .take_definitions(false)
                    .into_iter()
                    .collect(),
                errors: Default::default(),
            },
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn add_method(
        &mut self,
        namespace: Option<&str>,
        name: &str,
        params: Vec<ContentDescriptor>,
        result: Option<ContentDescriptor>,
        doc: &str,
        tag: Option<String>,
        deprecated: bool,
    ) {
        let tags = tag.map(|t| Tag::new(&t)).into_iter().collect::<Vec<_>>();
        self.add_method_internal(namespace, name, params, result, doc, tags, deprecated);
    }

    #[expect(clippy::too_many_arguments)]
    pub fn add_subscription(
        &mut self,
        namespace: Option<&str>,
        name: &str,
        params: Vec<ContentDescriptor>,
        result: Option<ContentDescriptor>,
        doc: &str,
        tag: Option<String>,
        deprecated: bool,
    ) {
        let mut tags = tag.map(|t| Tag::new(&t)).into_iter().collect::<Vec<_>>();
        tags.push(Tag::new("Websocket"));
        tags.push(Tag::new("PubSub"));
        self.add_method_internal(namespace, name, params, result, doc, tags, deprecated);
    }

    #[expect(clippy::too_many_arguments)]
    fn add_method_internal(
        &mut self,
        namespace: Option<&str>,
        name: &str,
        params: Vec<ContentDescriptor>,
        result: Option<ContentDescriptor>,
        doc: &str,
        tags: Vec<Tag>,
        deprecated: bool,
    ) {
        let description = if doc.trim().is_empty() {
            None
        } else {
            Some(doc.trim().to_string())
        };

        let name = namespace.map_or_else(|| name.to_string(), |ns| format!("{ns}_{name}"));

        self.methods.insert(
            name.clone(),
            Method {
                name,
                description,
                params,
                result,
                tags,
                examples: Vec::new(),
                deprecated,
                // TODO: set these
                external_docs: None,
                param_structure: ParamStructure::Either,
                errors: Vec::new(),
                summary: None,
            },
        );
    }

    pub fn create_content_descriptor<T: JsonSchema>(
        &mut self,
        name: &str,
        summary: Option<String>,
        description: Option<String>,
        required: bool,
    ) -> ContentDescriptor {
        ContentDescriptor {
            name: name.replace(' ', ""),
            summary,
            description,
            required,
            schema: self.schema_generator.subschema_for::<T>(),
            deprecated: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Components {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    content_descriptors: BTreeMap<String, ContentDescriptor>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    schemas: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    errors: BTreeMap<String, Error>,
}
