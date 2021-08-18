use std::borrow::Cow;

use async_graphql::{registry, InputType, InputValueError, InputValueResult, Type, Value};

use crate::ActiveValue;

impl<T> InputType for ActiveValue<T>
where
    T: InputType,
    crate::Value: From<T>,
{
    default fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value {
            None | Some(Value::Null) => Ok(ActiveValue::unset()),
            Some(value) => Ok(ActiveValue::set(
                T::parse(Some(value)).map_err(InputValueError::propagate)?,
            )),
        }
    }

    default fn to_value(&self) -> Value {
        if self.is_set() {
            self.to_value()
        } else {
            Value::Null
        }
    }
}

impl<T> InputType for ActiveValue<Option<T>>
where
    T: InputType,
    crate::Value: From<Option<T>>,
{
    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value {
            None => Ok(ActiveValue::unset()),
            Some(Value::Null) => Ok(ActiveValue::set(None)),
            Some(value) => Ok(ActiveValue::set(Some(
                T::parse(Some(value)).map_err(InputValueError::propagate)?,
            ))),
        }
    }

    fn to_value(&self) -> Value {
        if self.is_set() {
            self.to_value()
        } else {
            Value::Null
        }
    }
}

impl<T> Type for ActiveValue<T>
where
    T: Type,
    crate::Value: From<T>,
{
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::type_name().to_string()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        T::type_name().to_string()
    }
}
