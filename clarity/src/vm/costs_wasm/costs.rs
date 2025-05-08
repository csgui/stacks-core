use std::fmt;

use wasmtime::{AsContextMut, Extern, Global, Mutability, Val, ValType};

/// Globals used for cost tracking
#[derive(Debug, Clone, Copy)]
pub struct CostGlobals {
    pub runtime: Global,
    pub read_count: Global,
    pub read_length: Global,
    pub write_count: Global,
    pub write_length: Global,
}

/// Trait for a `Linker` that can be used to retrieve the cost globals.
pub trait CostLinker<T> {
    /// Get the cost globals.
    fn get_cost_globals(&self, store: impl AsContextMut<Data = T>)
        -> wasmtime::Result<CostGlobals>;
    /// Define the cost globals.
    fn define_cost_globals(&mut self, store: impl AsContextMut<Data = T>) -> wasmtime::Result<()>;
}

/// Convenience to use the same error string in multiple places
#[derive(Debug)]
enum GetCostGlobalsError {
    Runtime,
    ReadCount,
    ReadLength,
    WriteCount,
    WriteLength,
}

impl fmt::Display for GetCostGlobalsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GetCostGlobalsError::*;

        match self {
            Runtime => write!(f, "missing `cost-runtime` global"),
            ReadCount => write!(f, "missing `cost-read-count` global"),
            ReadLength => write!(f, "missing `cost-read-length` global"),
            WriteCount => write!(f, "missing `cost-write-count` global"),
            WriteLength => write!(f, "missing `cost-write-length` global"),
        }
    }
}

impl std::error::Error for GetCostGlobalsError {}

impl<T> CostLinker<T> for wasmtime::Linker<T> {
    fn get_cost_globals(
        &self,
        mut store: impl AsContextMut<Data = T>,
    ) -> wasmtime::Result<CostGlobals> {
        let mut store = store.as_context_mut();

        let runtime = self.get(&mut store, "clarity", "cost-runtime");
        let read_count = self.get(&mut store, "clarity", "cost-read-count");
        let read_length = self.get(&mut store, "clarity", "cost-read-length");
        let write_count = self.get(&mut store, "clarity", "cost-write-count");
        let write_length = self.get(&mut store, "clarity", "cost-write-length");

        use GetCostGlobalsError::*;

        fn unwrap_global_or(
            ext: Option<Extern>,
            err: GetCostGlobalsError,
        ) -> Result<Global, GetCostGlobalsError> {
            match ext {
                Some(Extern::Global(global)) => Ok(global),
                _ => Err(err),
            }
        }

        Ok(CostGlobals {
            runtime: unwrap_global_or(runtime, Runtime)?,
            read_count: unwrap_global_or(read_count, ReadCount)?,
            read_length: unwrap_global_or(read_length, ReadLength)?,
            write_count: unwrap_global_or(write_count, WriteCount)?,
            write_length: unwrap_global_or(write_length, WriteLength)?,
        })
    }

    fn define_cost_globals(
        &mut self,
        mut store: impl AsContextMut<Data = T>,
    ) -> wasmtime::Result<()> {
        let mut store = store.as_context_mut();

        define_cost_global_import(self, &mut store, "cost-runtime", 0)?;
        define_cost_global_import(self, &mut store, "cost-read-count", 0)?;
        define_cost_global_import(self, &mut store, "cost-read-length", 0)?;
        define_cost_global_import(self, &mut store, "cost-write-count", 0)?;
        define_cost_global_import(self, &mut store, "cost-write-length", 0)?;

        Ok(())
    }
}

fn define_cost_global_import<T>(
    linker: &mut wasmtime::Linker<T>,
    mut store: impl AsContextMut<Data = T>,
    name: &str,
    value: u64,
) -> wasmtime::Result<()> {
    use wasmtime::{Global, GlobalType};

    let mut store = store.as_context_mut();

    let global = Global::new(
        &mut store,
        GlobalType::new(ValType::I64, Mutability::Var),
        Val::I64(value as _),
    )?;

    linker.define(&mut store, "clarity", name, global)?;

    Ok(())
}
