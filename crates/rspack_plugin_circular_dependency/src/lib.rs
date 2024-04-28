#![feature(let_chains)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use derivative::Derivative;
use futures::future::BoxFuture;
use futures::join;
use rspack_core::{
  Compilation, CompilationOptimizeModules, DependenciesBlock, ModuleIdentifier, NormalModule,
  Plugin, PluginContext,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;

pub type OnDetectedFn = Box<dyn Fn(Vec<String>) -> BoxFuture<'static, Result<()>> + Send + Sync>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CircularDependencyPluginOptions {
  pub exclude: Option<RspackRegex>,
  pub include: Option<RspackRegex>,
  pub fail_on_error: Option<bool>,
  pub allow_async_cycles: Option<bool>,
  pub cwd: Option<String>,
  #[derivative(Debug = "ignore")]
  pub on_detected: Option<OnDetectedFn>,
}

#[plugin]
#[derive(Debug)]
pub struct CircularDependencyPlugin {
  options: CircularDependencyPluginOptions,
}

impl CircularDependencyPlugin {
  pub fn new(options: CircularDependencyPluginOptions) -> Self {
    Self::new_inner(options)
  }

  pub async fn check_circular_dependency(&self, compilation: &mut Compilation) {
    let excude_regex = self
      .options
      .exclude
      .clone()
      .unwrap_or(RspackRegex::new("$^").unwrap());
    let include_regex = self
      .options
      .exclude
      .clone()
      .unwrap_or(RspackRegex::new(".*").unwrap());
    let mut cyclical_paths: HashSet<String> = HashSet::new();
    let fail_on_error = self.options.fail_on_error.unwrap_or(false);
    let mut diagnostics: Vec<Diagnostic> = vec![];
    for module in compilation.get_module_graph().modules().iter() {
      if let Some(normal_module) = module.1.as_normal_module() {
        let resource = &normal_module.resource_resolved_data().resource;
        let should_skip =
          resource.is_empty() || excude_regex.test(resource) || !include_regex.test(resource);

        // skip the module if it matches the exclude pattern
        if should_skip {
          continue;
        }
        let mut seen_modules = HashMap::new();
        let maybe_cyclical_paths_list =
          self.is_cyclic(normal_module, normal_module, &mut seen_modules, compilation);
        if let Some(cyclical_paths_list) = maybe_cyclical_paths_list {
          // check if its already in report list
          let mut copy_paths = cyclical_paths_list.clone();
          copy_paths.remove(0);
          copy_paths.sort();
          let copy_paths = copy_paths.join(",");
          if cyclical_paths.contains(&copy_paths) {
            continue;
          } else {
            cyclical_paths.insert(copy_paths);
          }

          let dep_chain = cyclical_paths_list.join(" -> ");
          if let Some(on_detected) = &self.options.on_detected {
            let feature = on_detected(cyclical_paths_list);
            let _ = join!(feature);
          }

          let error = format!("Circular dependency detected:\r\n {dep_chain}");
          if fail_on_error {
            diagnostics.push(Diagnostic::error(String::from("dep error"), error));
          } else {
            diagnostics.push(Diagnostic::warn(String::from("dep error"), error));
          }
        }
      }
    }

    compilation.push_batch_diagnostic(diagnostics);
  }

  pub fn is_cyclic(
    &self,
    init_module: &NormalModule,
    current_module: &NormalModule,
    seen_modules: &mut HashMap<ModuleIdentifier, bool>,
    compilation: &Compilation,
  ) -> Option<Vec<String>> {
    let cwd = self.options.cwd.clone().unwrap_or("./".to_string());
    let cwd_path = Path::new(&cwd);
    let allow_async_cycles = self.options.allow_async_cycles.unwrap_or(false);
    seen_modules.insert(current_module.id(), true);
    if init_module.resource_resolved_data().resource.is_empty()
      || current_module.resource_resolved_data().resource.is_empty()
    {
      return None;
    }
    for dependency_id in current_module.get_dependencies() {
      if let Some(box_module) = compilation
        .get_module_graph()
        .get_module_by_dependency_id(dependency_id)
        && let Some(dep_module) = box_module.as_normal_module()
        && !allow_async_cycles
        && current_module != dep_module
      {
        if seen_modules.get(&dep_module.id()).is_some() {
          if dep_module.id() == init_module.id() {
            // Initial module has a circular dependency
            return Some(vec![
              cwd_path
                .join(
                  current_module
                    .resource_resolved_data()
                    .resource_path
                    .clone(),
                )
                .to_str()
                .unwrap_or("")
                .to_string(),
              cwd_path
                .join(dep_module.resource_resolved_data().resource_path.clone())
                .to_str()
                .unwrap_or("")
                .to_string(),
            ]);
          }
          // Found a cycle, but not for this module
          continue;
        }
        let maybe_cyclical_paths_list =
          self.is_cyclic(init_module, dep_module, seen_modules, compilation);
        if let Some(mut cyclical_paths_list) = maybe_cyclical_paths_list {
          cyclical_paths_list.insert(
            0,
            cwd_path
              .join(
                current_module
                  .resource_resolved_data()
                  .resource_path
                  .clone(),
              )
              .to_str()
              .unwrap_or("")
              .to_string(),
          );
          return Some(cyclical_paths_list);
        }
      }
    }

    None
  }
}

#[plugin_hook(CompilationOptimizeModules for CircularDependencyPlugin)]
async fn optimize_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  self.check_circular_dependency(compilation).await;
  Ok(None)
}

impl Plugin for CircularDependencyPlugin {
  fn name(&self) -> &'static str {
    "rspack.CircularDependencyPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_modules
      .tap(optimize_modules::new(self));
    Ok(())
  }
}

#[cfg(test)]
mod tests {}
