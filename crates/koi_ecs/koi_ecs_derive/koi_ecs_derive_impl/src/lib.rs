use kreflect_common::*;

pub fn kecs_component_impl(value: &Value) -> String {
    let (name, generic_parameters) = match value {
        Value::Struct(s) => (&s.name, &s.generic_parameters),
        Value::Enum(e) => (&e.name, &e.generic_parameters),
    };

    format!(
        r#"
        impl{} WorldClonableTrait for {}{} {{
            fn clone_with_context(&self, entity_migrator: &EntityMigrator) -> Option<Vec<Self>> {{
                self.clone()
            }}
        }}
    "#,
        &generic_parameters.as_impl_args(),
        &name,
        &generic_parameters.as_args(),
    )
}