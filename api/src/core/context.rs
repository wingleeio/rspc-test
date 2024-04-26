use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug, Default)]
pub struct Context {
    pub data: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

macro_rules! query {
    ($ctx:expr, $($t:ty),+) => {{
        (
            $(
                $ctx.get::<$t>(&std::any::type_name::<$t>().to_string())
                    .ok_or_else(|| {
                        rspc::Error::new(
                            rspc::ErrorCode::InternalServerError,
                            format!(
                                "Failed to find value of type {} in the context.",
                                std::any::type_name::<$t>()
                            ),
                        )
                    })?
            ),+
        )
    }};
}

pub(crate) use query;

macro_rules! add {
    ($ctx:expr, $($value:expr),+) => {
        $(
            let value = $value;
            let type_name = std::any::type_name_of_val(&value);
            $ctx.insert(type_name, value);
        )+
    };
}

pub(crate) use add;

macro_rules! middleware {
    () => {
        impl rspc::internal::middleware::ConstrainedMiddleware<self::context::Context> + rspc::internal::middleware::SealedMiddleware<self::context::Context, NewCtx = self::context::Context>
    }
}

pub(crate) use middleware;

impl Context {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Arc::new(value));
    }

    pub fn get<T: Send + Sync + 'static>(&self, key: &str) -> Option<Arc<T>> {
        self.data.get(key).and_then(|v| v.clone().downcast().ok())
    }
}
