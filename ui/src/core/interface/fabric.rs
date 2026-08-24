use anyhow::{Result, anyhow};
use std::{any::Any, collections::HashMap};

trait FabricFunc: Send + Sync {
    fn build(&mut self) -> Box<dyn Any>;
}

struct FuncWrapper<T> {
    func: T,
}

impl<I: 'static + Any + Send + Sync, T: 'static + Send + Sync + Fn() -> I> FuncWrapper<T> {
    fn new(func: T) -> Box<dyn FabricFunc> {
        Box::new(Self { func })
    }
}

impl<I: 'static + Any + Send + Sync, T: 'static + Send + Sync + Fn() -> I> FabricFunc
    for FuncWrapper<T>
{
    fn build(&mut self) -> Box<dyn Any> {
        Box::new((self.func)())
    }
}

pub struct IfFabric {
    fabrics: HashMap<String, Box<dyn FabricFunc>>,
}

impl IfFabric {
    pub fn new() -> Self {
        IfFabric {
            fabrics: HashMap::new(),
        }
    }

    pub fn reg<I: 'static + Any + Send + Sync, F: 'static + Send + Sync + Fn() -> I>(
        &mut self,
        key: String,
        func: F,
    ) -> Result<()> {
        if self.fabrics.contains_key(&key) {
            return Err(anyhow!("Already exist"));
        }
        self.fabrics.insert(key, FuncWrapper::new(func));
        Ok(())
    }

    pub fn get<I: 'static + Any>(&mut self, key: String) -> Result<I> {
        let f = self
            .fabrics
            .get_mut(&key)
            .ok_or(anyhow!("Fabric for \"{key}\" not found"))?;
        Ok(*f
            .build()
            .downcast::<I>()
            .map_err(|_| anyhow!("Unexpected interface type"))?)
    }
}
