use std::path::Path;
use std::sync::Arc;

use either::Either;
use rspack_core::CompilerOptions;
use swc_core::atoms::Atom;
use swc_core::common::collections::AHashMap;
use swc_core::common::BytePos;
use swc_core::common::{comments::Comments, Mark, SourceMap};
use swc_core::ecma::ast::Ident;
use swc_core::ecma::visit::{noop_visit_type, Visit};
use swc_core::ecma::{transforms::base::pass::noop, visit::Fold};

use crate::options::RspackExperiments;

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      #[allow(clippy::redundant_closure_call)]
      Either::Left($f(config))
    } else {
      Either::Right(noop())
    }
  };
  ($config:expr, $f:expr, $enabled:expr) => {
    if $enabled {
      either!($config, $f)
    } else {
      Either::Right(noop())
    }
  };
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn transform<'a>(
  _resource_path: &'a Path,
  _rspack_options: &'a CompilerOptions,
  _comments: Option<&'a dyn Comments>,
  _top_level_mark: Mark,
  _unresolved_mark: Mark,
  _cm: Arc<SourceMap>,
  _content: &'a str,
  rspack_experiments: &'a RspackExperiments,
) -> impl Fold + 'a {
  either!(rspack_experiments.import, |options| {
    rspack_swc_visitors::import(options)
  })
}

pub struct IdentCollector {
  pub names: AHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
