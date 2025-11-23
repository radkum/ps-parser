use super::{RuntimeTypeTrait, ValType};

#[derive(Debug)]
pub(crate) struct TypeInfo {
    pub is_public: bool,
    pub is_serial: bool,
    pub name: String,
    pub base_type: Box<dyn RuntimeTypeTrait>,
}

impl std::fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "IsPublic\tIsSerial\tName\tBaseType")?;
        writeln!(f, "--------\t--------\t----\t--------")?;
        write!(
            f,
            "{:>8}\t{:>8}\t{:>4}\t{:>8}",
            self.is_public,
            self.is_serial,
            self.name,
            self.base_type.name()
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeType;
impl RuntimeTypeTrait for RuntimeType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(TypeInfoType {})
    }
    fn name(&self) -> String {
        "RuntimeType".to_string()
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TypeInfoType;
impl RuntimeTypeTrait for TypeInfoType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(TypeType {})
    }
    fn name(&self) -> String {
        "TypeInfo".to_string()
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TypeType;
impl RuntimeTypeTrait for TypeType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(MemberInfoType {})
    }
    fn name(&self) -> String {
        "Type".to_string()
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MemberInfoType;
impl RuntimeTypeTrait for MemberInfoType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(ObjectType {})
    }
    fn name(&self) -> String {
        "MemberInfo".to_string()
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ValueType;
impl RuntimeTypeTrait for ValueType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(ObjectType {})
    }
    fn name(&self) -> String {
        "ValueType".to_string()
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ArrayType;
impl RuntimeTypeTrait for ArrayType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(ObjectType {})
    }
    fn name(&self) -> String {
        "Array".to_string()
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectType;
impl RuntimeTypeTrait for ObjectType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(ValType::Null)
    }
    fn name(&self) -> String {
        "System.Object".to_string()
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}
