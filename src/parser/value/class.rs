use std::{collections::HashMap, vec};

use super::{RuntimeResult, Val, ValType};
use crate::parser::{
    RuntimeObjectTrait, ScriptBlock,
    value::{
        MethodError, MethodResult, RuntimeError, RuntimeTypeTrait, StaticFnCallType,
        val_type::ObjectType,
    },
};
pub(crate) type MethodMap = HashMap<String, ScriptBlock>;
use crate::parser::Param;

#[derive(Debug, Clone, Default)]
pub(crate) struct ClassProperties(HashMap<String, (Option<ValType>, Option<Val>)>);
impl ClassProperties {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_property(
        &mut self,
        name: String,
        val_type: Option<ValType>,
        default_value: Option<Val>,
    ) {
        self.0.insert(name, (val_type, default_value));
    }
}
#[derive(Debug, Clone)]
pub(crate) struct ClassType {
    name: String,
    properties: ClassProperties,
    constructors: MethodMap,
    static_methods: MethodMap,
    methods: MethodMap,
}

unsafe impl Sync for ClassType {}
unsafe impl Send for ClassType {}

fn strip_case_insensitive_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.to_ascii_lowercase()
        .starts_with(&prefix.to_ascii_lowercase())
    {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}
impl ClassType {
    pub fn new(
        name: String,
        properties: ClassProperties,
        static_methods: MethodMap,
        mut methods: MethodMap,
    ) -> Self {
        let mut constructors = HashMap::new();
        let mut key_to_remove = vec![];
        for (key, fn_body) in methods.iter() {
            if let Some(stripped) = strip_case_insensitive_prefix(key, &name)
                && (stripped.is_empty() || stripped.starts_with('('))
            {
                constructors.insert(format!("new{}", stripped), fn_body.clone());
                key_to_remove.push(key.clone());
                break;
            }
        }

        for key in key_to_remove {
            methods.remove(&key);
        }
        if constructors.is_empty() {
            constructors.insert("new".to_string(), ScriptBlock::default());
        }
        Self {
            name,
            properties,
            constructors,
            static_methods,
            methods,
        }
    }
}

impl RuntimeTypeTrait for ClassType {
    fn static_method(&self, name: MethodName) -> RuntimeResult<StaticFnCallType> {
        match name.name() {
            "new" => {
                self.constructors
                    .get(name.full_name())
                    .map(|sb| self.constructor(sb.clone()))
                    .ok_or_else(|| MethodError::MethodNotFound(name.full_name().into()).into())
                //Ok(Box::new(self.default_constructor()))
            }
            _ => {
                let Some(fn_body) = self.static_methods.get(name.full_name()).cloned() else {
                    return Err(MethodError::MethodNotFound(name.full_name().into()).into());
                };
                let Some(fun) = fn_body.get_static_method() else {
                    return Err(MethodError::MethodNotFound(name.full_name().into()).into());
                };
                Ok(fun)
            }
        }
    }
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(ObjectType {})
    }
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn full_name(&self) -> String {
        format!("System.{}", self.name())
    }

    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

impl ClassType {
    fn constructor(&self, constructor_body: ScriptBlock) -> StaticFnCallType {
        let class = self.clone();
        Box::new(move |args: Vec<Val>| new_instance(class.clone(), args, constructor_body.clone()))
    }
}
fn new_instance(
    mut class_type: ClassType,
    args: Vec<Val>,
    constructor_body: ScriptBlock,
) -> MethodResult<Val> {
    // Implementation of the 'new' method for class instantiation
    let properties = std::mem::take(&mut class_type.properties);
    let mut this = Val::RuntimeObject(Box::new(ClassObject::new(class_type, properties)));
    if let Some(mut constructor_fn) = constructor_body.get_method() {
        constructor_fn(&mut this, args)?;
    }

    Ok(this)
}
#[derive(Debug, Clone)]
pub(crate) struct ClassObject {
    class_type: ClassType,
    properties: ClassProperties,
}

impl ClassObject {
    pub fn new(class_type: ClassType, properties: ClassProperties) -> Self {
        Self {
            class_type,
            properties,
        }
    }
}

impl RuntimeObjectTrait for ClassObject {
    fn member(&mut self, name: &str) -> RuntimeResult<&mut Val> {
        match self.properties.0.get_mut(&name.to_ascii_lowercase()) {
            Some(prop) => {
                if prop.1.is_none() {
                    prop.1 = Some(Val::Null);
                }
                Ok(prop.1.as_mut().unwrap())
            }
            None => Err(RuntimeError::MemberNotFound(name.to_string())),
        }
    }

    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait> {
        Box::new(self.clone())
    }

    fn name(&self) -> String {
        self.class_type.name.to_string()
    }

    fn type_definition(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.class_type.clone())
    }

    fn method(
        &self,
        method_name: MethodName,
    ) -> RuntimeResult<super::runtime_object::MethodCallType> {
        match self
            .class_type
            .methods
            .get(method_name.full_name())
            .cloned()
        {
            Some(fn_body) => {
                let Some(fun) = fn_body.get_method() else {
                    return Err(MethodError::MethodNotFound(method_name.full_name().into()).into());
                };
                Ok(fun)
            }
            None => Err(MethodError::MethodNotFound(method_name.full_name().into()).into()),
        }
    }
}

impl std::fmt::Display for ClassObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys = vec![];
        let mut spaces = vec![];
        let mut values = vec![];
        let mut len_vec = vec![];
        for property in self.properties.0.iter() {
            let key = property.0.clone();
            let val = if let Some(val_type) = &property.1.1 {
                val_type.to_string()
            } else {
                "".to_string()
            };

            let max = std::cmp::max(key.len(), val.len());
            let max = std::cmp::max(max, 8);

            keys.push(key);
            spaces.push("-".repeat(max));
            values.push(val);
            len_vec.push(max);
        }

        writeln!(f, "{}", keys.join("\t"))?;
        writeln!(f, "{}", spaces.join("\t"))?;
        for (i, value) in values.iter().enumerate() {
            write!(f, "{:>width$}", value, width = len_vec[i])?;
            if i < values.len() - 1 {
                write!(f, "\t")?;
            }
        }
        Ok(())
    }
}

pub(crate) struct MethodName(String, Option<String>);
impl MethodName {
    pub fn new(name: &str, parameters: &[Param]) -> Self {
        let mangled = if parameters.is_empty() {
            None
        } else {
            let param_types: Vec<String> = parameters
                .iter()
                .filter_map(|p| p.ttype().map(|t| t.to_string()))
                .collect();
            Some(Self::mangle(name, param_types))
        };

        Self(name.to_ascii_lowercase(), mangled)
    }

    pub fn from_args(name: &str, parameters: &[Val]) -> Self {
        let mangled = if parameters.is_empty() {
            None
        } else {
            let param_types: Vec<String> =
                parameters.iter().map(|t| t.ttype().to_string()).collect();
            Some(Self::mangle(name, param_types))
        };
        Self(name.to_ascii_lowercase(), mangled)
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn full_name(&self) -> &str {
        match &self.1 {
            Some(mangled) => mangled.as_str(),
            None => &self.0,
        }
    }

    fn mangle(name: &str, args: Vec<String>) -> String {
        if args.is_empty() {
            return name.to_ascii_lowercase();
        }
        let mut mangled_name = name.to_ascii_lowercase();
        mangled_name.push('(');
        mangled_name.push_str(&args.join(","));
        mangled_name.push(')');
        mangled_name.push_str(&args.len().to_string());
        mangled_name
    }
}

impl From<MethodName> for String {
    fn from(value: MethodName) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue, Variables};

    #[test]
    fn default_constructor() {
        let mut p = PowerShellSession::new().with_variables(Variables::new().values_persist());
        let input = r#"
class Person {
    [string]$FirstName
    [string]$LastName

    [string] GetFullName() {
        return "$($this.FirstName) and $($this.LastName)"
    }
}
$person = [Person]::new()
"#;
        p.parse_input(input).unwrap();
        assert_eq!(
            p.parse_input("[person].gettype().name").unwrap().result(),
            PsValue::String("RuntimeType".to_string())
        );
        assert_eq!(
            p.parse_input("$person.gettype().name").unwrap().result(),
            PsValue::String("Person".to_string())
        );
        assert_eq!(
            p.parse_input("$person.LastName").unwrap().result(),
            PsValue::Null
        );
        assert_eq!(
            p.parse_input("$person.GetFullName()").unwrap().result(),
            PsValue::String(" and ".to_string())
        );
    }

    #[test]
    fn constructor() {
        let mut p = PowerShellSession::new().with_variables(Variables::new().values_persist());
        let input = r#"
class Person {
    [string]$FirstName
    [string]$LastName

    Person([string]$first, [string]$last) {
        $this.FirstName = $first
        $this.LastName  = $last
    }

    [string] GetFullName() {
        return "$($this.FirstName) $($this.LastName)"
    }
}
$person = [Person]::new('John', 'Doe')"#;
        p.parse_input(input).unwrap();
        assert_eq!(
            p.parse_input("$person.FirstName").unwrap().result(),
            PsValue::String("John".to_string())
        );
        assert_eq!(
            p.parse_input("$person.LastName").unwrap().result(),
            PsValue::String("Doe".to_string())
        );
        assert_eq!(
            p.parse_input("$person.GetFullName()").unwrap().result(),
            PsValue::String("John Doe".to_string())
        );
    }
}
