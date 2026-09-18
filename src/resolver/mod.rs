pub mod decode;
pub mod detect;
pub mod extract;
pub mod fetch;
pub mod pipeline;
pub mod stage;

pub use detect::ContentKind;
pub use pipeline::{AnalysisReport, Resolver, ResolverConfig};
