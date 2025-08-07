use crate::js::JsRuntime;
use crate::js::msg::JsRuntimeToEventLoopMessage;
use compact_str::CompactString;
use tracing::trace;

pub fn echo(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  assert!(args.length() == 1);
  let message = args.get(0).to_rust_string_lossy(scope).to_string();
  trace!("echo: {:?}", message);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let jsrt_to_mstr = state.jsrt_to_mstr.clone();
  let current_handle = tokio::runtime::Handle::current();
  current_handle.spawn_blocking(move || {
    let message = CompactString::from(message);
    jsrt_to_mstr
      .blocking_send(JsRuntimeToEventLoopMessage::EchoReq(
        crate::js::msg::EchoReq::new(message),
      ))
      .unwrap();
  });
}
