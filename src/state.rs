//! The global editing state.

use crossterm::event::Event;
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::{Arc, Weak};
use tracing::debug;

use crate::buffer::{Buffer, BufferId};
use crate::state::fsm::{Stateful, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;
use crate::ui::widget::WidgetId;

pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  stateful: StatefulValue,
  last_stateful: StatefulValue,

  // Editing mode.
  mode: Mode,

  // Widgets {

  // [`cursor`](crate::ui::widget::cursor::Cursor) widget ID.
  cursor_widget: Option<WidgetId>,
  // Current [`window container`](crate::ui::widget::window::WindowContainer) widget ID that the
  // cursor widget belongs to.
  current_window_widget: Option<WidgetId>,
  // All [`window`](crate::ui::widget::window::Window) widget IDs.
  window_widgets: BTreeSet<WidgetId>,

  // Widgets }

  // Buffers {

  // Buffers.
  buffers: BTreeMap<BufferId, Buffer>,

  // Current [`buffer`](crate::ui::buffer::Buffer) ID.
  current_buffer: Option<BufferId>,
  // Buffers }
}

#[derive(Debug, Copy, Clone)]
pub struct StateHandleResponse {
  pub stateful: StatefulValue,
  pub next_stateful: StatefulValue,
}

impl StateHandleResponse {
  pub fn new(stateful: StatefulValue, next_stateful: StatefulValue) -> Self {
    StateHandleResponse {
      stateful,
      next_stateful,
    }
  }
}

pub type StateArc = Arc<Mutex<State>>;
pub type StateWk = Weak<Mutex<State>>;

impl State {
  pub fn new() -> Self {
    State {
      stateful: StatefulValue::default(),
      last_stateful: StatefulValue::default(),
      mode: Mode::Normal,
      cursor_widget: None,
      current_window_widget: None,
      window_widgets: BTreeSet::new(),
      buffers: BTreeMap::new(),
      current_buffer: None,
    }
  }

  pub fn to_arc(s: State) -> StateArc {
    Arc::new(Mutex::new(s))
  }

  pub fn handle(&mut self, tree: TreeArc, event: Event) -> StateHandleResponse {
    // Current stateful
    let stateful = self.stateful;

    let data_access = StatefulDataAccessMut::new(self, tree, event);
    let next_stateful = stateful.handle(data_access);
    debug!("Stateful now:{:?}, next:{:?}", stateful, next_stateful);

    // Save current stateful
    self.last_stateful = stateful;
    // Set next stateful
    self.stateful = next_stateful;

    StateHandleResponse::new(stateful, next_stateful)
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }

  pub fn set_mode(&mut self, mode: Mode) -> Mode {
    let last_mod = self.mode;
    self.mode = mode;
    last_mod
  }

  pub fn cursor_widget(&self) -> &Option<WidgetId> {
    &self.cursor_widget
  }

  pub fn set_cursor_widget(&mut self, widget_id: Option<WidgetId>) -> Option<WidgetId> {
    let old_widget = self.cursor_widget;
    self.cursor_widget = widget_id;
    old_widget
  }

  pub fn current_window_widget(&self) -> &Option<WidgetId> {
    &self.current_window_widget
  }

  pub fn set_current_window_widget(&mut self, widget_id: Option<WidgetId>) -> Option<WidgetId> {
    let old_widget = self.cursor_widget;
    self.cursor_widget = widget_id;
    old_widget
  }

  pub fn window_widgets(&self) -> &BTreeSet<WidgetId> {
    &self.window_widgets
  }

  pub fn window_widgets_mut(&mut self) -> &mut BTreeSet<WidgetId> {
    &mut self.window_widgets
  }

  pub fn buffers(&self) -> &BTreeMap<BufferId, Buffer> {
    &self.buffers
  }

  pub fn buffers_mut(&mut self) -> &mut BTreeMap<BufferId, Buffer> {
    &mut self.buffers
  }

  pub fn current_buffer(&self) -> Option<BufferId> {
    self.current_buffer
  }

  pub fn set_current_buffer(&mut self, buffer_id: Option<BufferId>) -> Option<BufferId> {
    let old_buffer = self.current_buffer;
    self.current_buffer = buffer_id;
    old_buffer
  }
}

impl Default for State {
  fn default() -> Self {
    State::new()
  }
}
