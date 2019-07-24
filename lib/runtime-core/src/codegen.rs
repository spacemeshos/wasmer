use crate::{
    backend::RunnableModule,
    backend::{Backend, CacheGen, Compiler, CompilerConfig, Token},
    cache::{Artifact, Error as CacheError},
    error::{CompileError, CompileResult},
    module::{ModuleInfo, ModuleInner},
    structures::Map,
    types::{FuncIndex, FuncSig, SigIndex},
};
use smallvec::SmallVec;
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use wasmparser::{self, WasmDecoder};
use wasmparser::{Operator, Type as WpType};

pub type BreakpointHandler =
    Box<Fn(BreakpointInfo) -> Result<(), Box<dyn Any>> + Send + Sync + 'static>;
pub type BreakpointMap = Arc<HashMap<usize, BreakpointHandler>>;

#[derive(Debug)]
pub enum Event<'a, 'b> {
    Internal(InternalEvent),
    Wasm(&'b Operator<'a>),
    WasmOwned(Operator<'a>),
}

pub enum InternalEvent {
    FunctionBegin(u32),
    FunctionEnd,
    Breakpoint(BreakpointHandler),
    SetInternal(u32),
    GetInternal(u32),
}

impl fmt::Debug for InternalEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InternalEvent::FunctionBegin(_) => write!(f, "FunctionBegin"),
            InternalEvent::FunctionEnd => write!(f, "FunctionEnd"),
            InternalEvent::Breakpoint(_) => write!(f, "Breakpoint"),
            InternalEvent::SetInternal(_) => write!(f, "SetInternal"),
            InternalEvent::GetInternal(_) => write!(f, "GetInternal"),
        }
    }
}

pub struct BreakpointInfo<'a> {
    pub fault: Option<&'a dyn Any>,
}

pub trait ModuleCodeGenerator<FCG: FunctionCodeGenerator<E>, RM: RunnableModule, E: Debug> {
    /// Creates a new module code generator.
    fn new() -> Self;

    /// Returns the backend id associated with this MCG.
    fn backend_id() -> Backend;

    /// Feeds the compiler config.
    fn feed_compiler_config(&mut self, _config: &CompilerConfig) -> Result<(), E> {
        Ok(())
    }
    /// Adds an import function.
    fn feed_import_function(&mut self) -> Result<(), E>;

    fn feed_signatures(&mut self, signatures: Map<SigIndex, FuncSig>) -> Result<(), E>;
    /// Sets function signatures.
    fn feed_function_signatures(&mut self, assoc: Map<FuncIndex, SigIndex>) -> Result<(), E>;
    /// Checks the precondition for a module.
    fn check_precondition(&mut self, module_info: &ModuleInfo) -> Result<(), E>;
    /// Creates a new function and returns the function-scope code generator for it.
    fn next_function(&mut self, module_info: Arc<RwLock<ModuleInfo>>) -> Result<&mut FCG, E>;
    /// Finalizes this module.
    fn finalize(self, module_info: &ModuleInfo) -> Result<(RM, Box<dyn CacheGen>), E>;

    /// Creates a module from cache.
    unsafe fn from_cache(cache: Artifact, _: Token) -> Result<ModuleInner, CacheError>;
}

pub struct StreamingCompiler<
    MCG: ModuleCodeGenerator<FCG, RM, E>,
    FCG: FunctionCodeGenerator<E>,
    RM: RunnableModule + 'static,
    E: Debug,
    CGEN: Fn() -> MiddlewareChain,
> {
    middleware_chain_generator: CGEN,
    _phantom_mcg: PhantomData<MCG>,
    _phantom_fcg: PhantomData<FCG>,
    _phantom_rm: PhantomData<RM>,
    _phantom_e: PhantomData<E>,
}

pub struct SimpleStreamingCompilerGen<
    MCG: ModuleCodeGenerator<FCG, RM, E>,
    FCG: FunctionCodeGenerator<E>,
    RM: RunnableModule + 'static,
    E: Debug,
> {
    _phantom_mcg: PhantomData<MCG>,
    _phantom_fcg: PhantomData<FCG>,
    _phantom_rm: PhantomData<RM>,
    _phantom_e: PhantomData<E>,
}

impl<
        MCG: ModuleCodeGenerator<FCG, RM, E>,
        FCG: FunctionCodeGenerator<E>,
        RM: RunnableModule + 'static,
        E: Debug,
    > SimpleStreamingCompilerGen<MCG, FCG, RM, E>
{
    pub fn new() -> StreamingCompiler<MCG, FCG, RM, E, impl Fn() -> MiddlewareChain> {
        StreamingCompiler::new(|| MiddlewareChain::new())
    }
}

impl<
        MCG: ModuleCodeGenerator<FCG, RM, E>,
        FCG: FunctionCodeGenerator<E>,
        RM: RunnableModule + 'static,
        E: Debug,
        CGEN: Fn() -> MiddlewareChain,
    > StreamingCompiler<MCG, FCG, RM, E, CGEN>
{
    pub fn new(chain_gen: CGEN) -> Self {
        Self {
            middleware_chain_generator: chain_gen,
            _phantom_mcg: PhantomData,
            _phantom_fcg: PhantomData,
            _phantom_rm: PhantomData,
            _phantom_e: PhantomData,
        }
    }
}

pub fn default_validating_parser_config() -> wasmparser::ValidatingParserConfig {
    wasmparser::ValidatingParserConfig {
        operator_config: wasmparser::OperatorValidatorConfig {
            enable_threads: false,
            enable_reference_types: false,
            enable_simd: true,
            enable_bulk_memory: false,
            enable_multi_value: false,
        },
        mutable_global_imports: false,
    }
}

fn validate(bytes: &[u8]) -> CompileResult<()> {
    let mut parser =
        wasmparser::ValidatingParser::new(bytes, Some(default_validating_parser_config()));
    loop {
        let state = parser.read();
        match *state {
            wasmparser::ParserState::EndWasm => break Ok(()),
            wasmparser::ParserState::Error(err) => Err(CompileError::ValidationError {
                msg: err.message.to_string(),
            })?,
            _ => {}
        }
    }
}

impl<
        MCG: ModuleCodeGenerator<FCG, RM, E>,
        FCG: FunctionCodeGenerator<E>,
        RM: RunnableModule + 'static,
        E: Debug,
        CGEN: Fn() -> MiddlewareChain,
    > Compiler for StreamingCompiler<MCG, FCG, RM, E, CGEN>
{
    fn compile(
        &self,
        wasm: &[u8],
        compiler_config: CompilerConfig,
        _: Token,
    ) -> CompileResult<ModuleInner> {
        if requires_pre_validation(MCG::backend_id()) {
            validate(wasm)?;
        }

        let mut mcg = MCG::new();
        let mut chain = (self.middleware_chain_generator)();
        let info = crate::parse::read_module(
            wasm,
            MCG::backend_id(),
            &mut mcg,
            &mut chain,
            &compiler_config,
        )?;
        let (exec_context, cache_gen) =
            mcg.finalize(&info.read().unwrap())
                .map_err(|x| CompileError::InternalError {
                    msg: format!("{:?}", x),
                })?;
        Ok(ModuleInner {
            cache_gen,
            runnable_module: Box::new(exec_context),
            info: Arc::try_unwrap(info).unwrap().into_inner().unwrap(),
        })
    }

    unsafe fn from_cache(
        &self,
        artifact: Artifact,
        token: Token,
    ) -> Result<ModuleInner, CacheError> {
        MCG::from_cache(artifact, token)
    }
}

fn requires_pre_validation(backend: Backend) -> bool {
    match backend {
        Backend::Cranelift => true,
        Backend::LLVM => false,
        Backend::Singlepass => false,
    }
}

pub struct EventSink<'a, 'b> {
    buffer: SmallVec<[Event<'a, 'b>; 2]>,
}

impl<'a, 'b> EventSink<'a, 'b> {
    pub fn push(&mut self, ev: Event<'a, 'b>) {
        self.buffer.push(ev);
    }
}

pub struct MiddlewareChain {
    chain: Vec<Box<GenericFunctionMiddleware>>,
}

impl MiddlewareChain {
    pub fn new() -> MiddlewareChain {
        MiddlewareChain { chain: vec![] }
    }

    pub fn push<M: FunctionMiddleware + 'static>(&mut self, m: M) {
        self.chain.push(Box::new(m));
    }

    pub(crate) fn run<E: Debug, FCG: FunctionCodeGenerator<E>>(
        &mut self,
        fcg: Option<&mut FCG>,
        ev: Event,
        module_info: &ModuleInfo,
    ) -> Result<(), String> {
        let mut sink = EventSink {
            buffer: SmallVec::new(),
        };
        sink.push(ev);
        for m in &mut self.chain {
            let prev: SmallVec<[Event; 2]> = sink.buffer.drain().collect();
            for ev in prev {
                m.feed_event(ev, module_info, &mut sink)?;
            }
        }
        if let Some(fcg) = fcg {
            for ev in sink.buffer {
                fcg.feed_event(ev, module_info)
                    .map_err(|x| format!("{:?}", x))?;
            }
        }

        Ok(())
    }
}

pub trait FunctionMiddleware {
    type Error: Debug;
    fn feed_event<'a, 'b: 'a>(
        &mut self,
        op: Event<'a, 'b>,
        module_info: &ModuleInfo,
        sink: &mut EventSink<'a, 'b>,
    ) -> Result<(), Self::Error>;
}

pub(crate) trait GenericFunctionMiddleware {
    fn feed_event<'a, 'b: 'a>(
        &mut self,
        op: Event<'a, 'b>,
        module_info: &ModuleInfo,
        sink: &mut EventSink<'a, 'b>,
    ) -> Result<(), String>;
}

impl<E: Debug, T: FunctionMiddleware<Error = E>> GenericFunctionMiddleware for T {
    fn feed_event<'a, 'b: 'a>(
        &mut self,
        op: Event<'a, 'b>,
        module_info: &ModuleInfo,
        sink: &mut EventSink<'a, 'b>,
    ) -> Result<(), String> {
        <Self as FunctionMiddleware>::feed_event(self, op, module_info, sink)
            .map_err(|x| format!("{:?}", x))
    }
}

/// The function-scope code generator trait.
pub trait FunctionCodeGenerator<E: Debug> {
    /// Sets the return type.
    fn feed_return(&mut self, ty: WpType) -> Result<(), E>;

    /// Adds a parameter to the function.
    fn feed_param(&mut self, ty: WpType) -> Result<(), E>;

    /// Adds `n` locals to the function.
    fn feed_local(&mut self, ty: WpType, n: usize) -> Result<(), E>;

    /// Called before the first call to `feed_opcode`.
    fn begin_body(&mut self, module_info: &ModuleInfo) -> Result<(), E>;

    /// Called for each operator.
    fn feed_event(&mut self, op: Event, module_info: &ModuleInfo) -> Result<(), E>;

    /// Finalizes the function.
    fn finalize(&mut self) -> Result<(), E>;
}
