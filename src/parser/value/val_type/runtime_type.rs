use super::{
    super::{MethodError, StaticFnCallType},
    RuntimeResult, Val, ValType,
    type_info::TypeInfo,
};
use crate::parser::MethodName;

pub(crate) trait RuntimeTypeTrait: std::fmt::Debug + Sync + Send {
    fn describe(&self) -> String {
        format!("{}", self.type_info())
    }

    fn static_method(&self, method_name: MethodName) -> RuntimeResult<StaticFnCallType> {
        Err(MethodError::NotImplemented(method_name.name().into()).into())
    }
    fn readonly_static_member(&self, name: &str) -> RuntimeResult<Val> {
        Err(MethodError::NotImplemented(name.to_string()).into())
    }
    fn readonly_member(&self, name: &str) -> RuntimeResult<Val> {
        match name.to_ascii_lowercase().as_str() {
            "name" => Ok(Val::String(self.name().into())),
            "basetype" => Ok(Val::RuntimeType(self.base_type())),
            "ispublic" => Ok(Val::Bool(true)),
            "isserial" => Ok(Val::Bool(true)),
            _ => Err(MethodError::NotImplemented(name.to_string()).into()),
        }
    }

    fn base_type(&self) -> Box<dyn RuntimeTypeTrait>;

    fn name(&self) -> String;

    fn full_name(&self) -> String {
        format!("System.{}", self.name())
    }

    fn type_definition(&self) -> ValType {
        ValType::RuntimeObject(self.full_name())
    }

    fn type_info(&self) -> TypeInfo {
        TypeInfo {
            is_public: true,
            is_serial: true,
            name: self.name(),
            base_type: self.base_type(),
        }
    }

    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait>;
}

#[cfg(test)]
mod tests {
    use super::{super::ValError, *};

    #[test]
    fn runtime_type() {
        assert_eq!(
            ValType::runtime_type_from_str("null").unwrap_err(),
            ValError::UnknownType("null".into())
        );

        let runtime_object = ValType::runtime_type_from_str("Bool".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        assert_eq!(
            Val::RuntimeType(Box::new(ValType::Bool)).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), ValType::Bool);
        }

        let runtime_object = ValType::runtime_type_from_str("char".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        assert_eq!(
            Val::RuntimeType(Box::new(ValType::Char)).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), ValType::Char);
        }

        let runtime_object = ValType::runtime_type_from_str("double".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::Float;
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        let runtime_object = ValType::runtime_type_from_str("StRing".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::String;
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        let runtime_object = ValType::runtime_type_from_str("long".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::Int;
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        let runtime_object = ValType::runtime_type_from_str("array".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::Array(None);
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        //it's failing because right now Array is not a structure and ValType::Array
        // and Val::Array are inconsistent assert_eq!(rt.ttype(),
        // ValType::RuntimeType("Array".into()));
        //assert_eq!(rt.type_definition().unwrap(), ValType::Array(None));

        let runtime_object = ValType::runtime_type_from_str("hashtable".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::HashTable;
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        let runtime_object = ValType::runtime_type_from_str("scriptblock".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::ScriptBlock;
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        let runtime_object = ValType::runtime_type_from_str("system.convert".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::RuntimeObject("System.Convert".into());
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        let runtime_object = ValType::runtime_type_from_str("system.text.encoding".into()).unwrap();
        let runtime_type = runtime_object.ttype();
        let val_type = ValType::RuntimeObject("System.Text.Encoding".into());
        assert_eq!(
            Val::RuntimeType(Box::new(val_type.clone())).ttype(),
            runtime_type
        );
        if let Val::RuntimeType(rt_type) = runtime_object {
            assert_eq!(rt_type.type_definition(), val_type);
        }

        // let runtime_object =
        // ValType::runtime_type_from_str("system.text.encoding::unicode".into()).
        // unwrap(); let runtime_type = runtime_object.ttype();
        // let val_type  = ValType::RuntimeObject("System.Text.UnicodeEncoding".into());
        // assert_eq!(Val::RuntimeType(Box::new(val_type.clone())).ttype(),
        // runtime_type); if let Val::RuntimeType(rt_type) = runtime_object {
        //     assert_eq!(rt_type.type_definition(), val_type);
        // }

        assert_eq!(
            ValType::cast("a").unwrap_err(),
            ValError::UnknownType("a".into())
        );
    }
}
