use napi_derive::napi;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_circular_dependency::{CircularDependencyPluginOptions, OnDetectedFn};

#[napi(object, object_to_js = false)]
pub struct RawCircularDependencyPluginOptions {
  pub exclude: Option<JsRegExp>,
  pub include: Option<JsRegExp>,
  pub fail_on_error: Option<bool>,
  pub allow_async_cycles: Option<bool>,
  pub cwd: Option<String>,
  #[napi(ts_type = "(paths: string[]) => void")]
  pub on_detected: Option<ThreadsafeFunction<Vec<String>, ()>>,
}

fn normalize_on_detected_fn(raw: ThreadsafeFunction<Vec<String>, ()>) -> OnDetectedFn {
  Box::new(move |cyclical_paths_list| {
    let raw = raw.clone();
    let value = cyclical_paths_list.clone();
    Box::pin(async move { raw.call(value).await })
  })
}
impl From<RawCircularDependencyPluginOptions> for CircularDependencyPluginOptions {
  fn from(value: RawCircularDependencyPluginOptions) -> Self {
    let mut exclude = None;
    if let Some(ex) = value.exclude {
      exclude = Some(ex.to_rspack_regex());
    }
    let mut include = None;
    if let Some(inc) = value.include {
      include = Some(inc.to_rspack_regex());
    }
    let on_detected = value.on_detected.map(normalize_on_detected_fn);
    Self {
      exclude,
      include,
      fail_on_error: value.fail_on_error,
      allow_async_cycles: value.allow_async_cycles,
      cwd: value.cwd,
      on_detected,
    }
  }
}
