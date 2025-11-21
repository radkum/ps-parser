use crate::Variables;
use std::collections::HashMap;
use std::vec;
use super::{
    RuntimeResult, Val, ValType,
};
use crate::parser::RuntimeObjectTrait;
use crate::parser::variables::FunctionMap;
use crate::parser::value::{RuntimeTypeTrait};
use crate::parser::value::val_type::ObjectType;
pub(crate) type ClassProperties = HashMap<String, (Option<ValType>, Option<Val>)>;
use crate::parser::value::MethodResult;
use crate::parser::value::MethodError;
use crate::parser::value::StaticFnCallType;

#[derive(Debug, Clone)]
pub(crate) struct ClassType {
    name: String,
    properties: ClassProperties,
    static_functions: FunctionMap,
    methods: FunctionMap,
}

unsafe impl Sync for ClassType {}
unsafe impl Send for ClassType {}

impl ClassType {
    pub fn new(
        name: String,
        properties: ClassProperties,
        static_functions: FunctionMap,
        methods: FunctionMap,
    ) -> Self {
        Self { name, properties, static_functions, methods }
    }
}

impl RuntimeTypeTrait for ClassType {
    fn static_method(&self, name: &str) -> RuntimeResult<StaticFnCallType> {
        match name.to_ascii_lowercase().as_str() {
            "new" => Ok(Box::new(new_instance)),
            _ => {
                let Some(fn_body) = self.static_functions
                    .get(&name.to_ascii_lowercase()).cloned() else {
                    return Err(MethodError::MethodNotFound(name.to_string()).into());
                    };
                let Some(fun) = fn_body.get_method() else {
                    return Err(MethodError::MethodNotFound(name.to_string()).into());
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
fn new_instance(args: Vec<Val>) -> MethodResult<Val> {
    // Implementation of the 'new' method for class instantiation
    Ok(Val::RuntimeObject(Box::new(ClassObject {
        properties: HashMap::new(),
        methods: HashMap::new(),
    })))
}

#[derive(Debug, Clone)]
pub(crate) struct ClassObject {
    properties: ClassProperties,
    methods: FunctionMap,
}

impl ClassObject {
    pub fn new(
        properties: ClassProperties,
        methods: FunctionMap,
    ) -> Self {
        Self { properties, methods }
    }
}

impl RuntimeObjectTrait for ClassObject {
    // fn method(&self, name: &str) -> RuntimeResult<StaticFnCallType> {
    //     let Some(fn_body) = self.methods
    //         .get(&name.to_ascii_lowercase()).cloned() else {
    //         return Err(MethodError::MethodNotFound(name.to_string()).into());
    //     };
    //     let Some(fun) = fn_body.get_method() else {
    //         return Err(MethodError::MethodNotFound(name.to_string()).into());
    //     };
    //     Ok(fun)
    // }

    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait> {
        Box::new(self.clone())
    }
}

impl std::fmt::Display for ClassObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys = vec![];
        let mut spaces = vec![];
        let mut values = vec![];
        let mut len_vec = vec![];
        for property in self.properties.iter() {
            let key = property.0.clone();
            let val = if let Some(val_type) = &property.1 .1 {
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

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn dfeault_construction() {
        let mut p = PowerShellSession::new();
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
$person = [Person]::new()
$person.FirstName
$person.LastName
$person.GetFullName()"#;
        p.parse_input(input).unwrap();
        assert_eq!(p.parse_input("$person.FirstName").unwrap().result(), PsValue::String("John".to_string()));
        assert_eq!(p.parse_input("$person.LastName").unwrap().result(), PsValue::String("Doe".to_string()));
        assert_eq!(p.parse_input("$person.GetFullName()").unwrap().result(), PsValue::String("John Doe".to_string()));
    }

    #[test]
    fn constructor() {
        let mut p = PowerShellSession::new();
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
$person = [Person]::new('John', 'Doe')
$person.FirstName
$person.LastName
$person.GetFullName()"#;
        p.parse_input(input).unwrap();
        assert_eq!(p.parse_input("$person.FirstName").unwrap().result(), PsValue::String("John".to_string()));
        assert_eq!(p.parse_input("$person.LastName").unwrap().result(), PsValue::String("Doe".to_string()));
        assert_eq!(p.parse_input("$person.GetFullName()").unwrap().result(), PsValue::String("John Doe".to_string()));
    }
}