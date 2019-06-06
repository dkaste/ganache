#![warn(rust_2018_idioms)]

pub mod layout;
mod theme;

pub use self::theme::{SlotStyle, Theme};

use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

#[cfg(not(feature = "scalar_i32"))]
mod scalar {
    pub type Scalar = f64;

    pub const ZERO: Scalar = 0.0;
    pub const TWO: Scalar = 2.0;
}

#[cfg(feature = "scalar_i32")]
mod scalar {
    pub type Scalar = i32;

    pub const ZERO: Scalar = 0;
    pub const TWO: Scalar = 2;
}

pub use self::scalar::Scalar;

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub width: Scalar,
    pub height: Scalar,
}

impl Dimensions {
    pub fn new(width: Scalar, height: Scalar) -> Dimensions {
        Dimensions { width, height }
    }

    pub fn zero() -> Dimensions {
        Dimensions {
            width: scalar::ZERO,
            height: scalar::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: Scalar,
    pub y: Scalar,
    pub size: Dimensions,
}

impl Bounds {
    pub fn new(x: Scalar, y: Scalar, width: Scalar, height: Scalar) -> Bounds {
        Bounds {
            x,
            y,
            size: Dimensions::new(width, height),
        }
    }

    pub fn zero() -> Bounds {
        Bounds {
            x: scalar::ZERO,
            y: scalar::ZERO,
            size: Dimensions::zero(),
        }
    }
}

pub trait InputEvent {
    fn dirty(&self) -> bool;
    
    fn offset_coordinates(&mut self, x: Scalar, y: Scalar);
}

pub trait Context: 'static {
    type ThemeResources;
    type StyleFieldValue;
    type DrawCommand;
    type DrawContext: Copy;
    type InputEvent: InputEvent;
}

pub struct ProcessEventResult {
    pub request_focus: bool,
    pub signals: Vec<Signal>,
}

impl Default for ProcessEventResult {
    fn default() -> ProcessEventResult {
        ProcessEventResult {
            request_focus: false,
            signals: Vec::new(),
        }
    }
}

pub struct Signal {
    name: String,
    fields: HashMap<String, Box<dyn Any>>,
}

impl Signal {
    pub fn new<T: Into<String>>(name: T) -> Signal {
        Signal {
            name: name.into(),
            fields: HashMap::new(),
        }
    }

    pub fn with_fields<T: Into<String>>(name: T, fields: HashMap<String, Box<dyn Any>>) -> Signal {
        Signal {
            name: name.into(),
            fields,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field<T: 'static>(&self, name: &str) -> &T {
        self.fields
            .get(name)
            .and_then(|f| f.downcast_ref())
            .unwrap()
    }
}

pub struct MinimumSizeArgs<'a, C: Context> {
    pub slots: &'a Slots,
    pub slot_id: SlotId,
    pub minimum_size_cache: &'a HashMap<SlotId, Dimensions>,
    pub resources: &'a C::ThemeResources,
    pub style: &'a SlotStyle<'a, C::ThemeResources, C::StyleFieldValue>,
}

pub struct LayoutChildrenArgs<'a, C: Context> {
    pub slots: &'a mut Slots,
    pub slot_id: SlotId,
    pub minimum_size_cache: &'a HashMap<SlotId, Dimensions>,
    // TODO: Keeping the C parameter right now for future proofing, look into removing
    phantom: PhantomData<C>,
}

pub struct ProcessEventArgs<'a, C: Context> {
    pub slots: &'a mut Slots,
    pub slot_id: SlotId,
    pub bounds: Bounds,
    pub event: &'a mut C::InputEvent,
    pub focused: bool,
    pub resources: &'a C::ThemeResources,
    pub style: &'a SlotStyle<'a, C::ThemeResources, C::StyleFieldValue>,
}

pub struct DrawArgs<'a, C: Context> {
    pub bounds: Bounds,
    pub focused: bool,
    pub resources: &'a C::ThemeResources,
    pub style: &'a SlotStyle<'a, C::ThemeResources, C::StyleFieldValue>,
    pub context: &'a mut C::DrawContext,
    pub commands: &'a mut Vec<C::DrawCommand>,
}

pub trait Widget<C: Context>: downcast_rs::Downcast {
    fn kind_id(&self) -> &'static str;

    fn takes_focus(&self) -> bool;

    /// The minimum size that this widget's slot can shrink to.
    fn minimum_size(&self, args: MinimumSizeArgs<'_, C>) -> Dimensions;

    fn layout_children(&self, args: LayoutChildrenArgs<'_, C>);

    fn process_event(&mut self, args: ProcessEventArgs<'_, C>) -> ProcessEventResult;

    fn draw(&self, args: DrawArgs<'_, C>);
}

downcast_rs::impl_downcast!(Widget<C> where C: Context);

pub fn visible_children<'a>(
    slots: &'a Slots,
    slot_id: SlotId,
) -> impl Iterator<Item = SlotId> + 'a {
    let slot = slots.get(slot_id);
    slot.children
        .iter()
        .filter(move |&child_id| !slots.get(*child_id).info.hidden)
        .cloned()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u32);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct WidgetHandle<C: Context, W: Widget<C>>(WidgetId, PhantomData<(C, W)>);

impl<C: Context, W: Widget<C>> Clone for WidgetHandle<C, W> {
    fn clone(&self) -> Self {
        WidgetHandle(self.0, PhantomData)
    }
}

impl<C: Context, W: Widget<C>> Copy for WidgetHandle<C, W> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrowDirection {
    Begin,
    End,
    Both,
}

pub struct SlotInfo {
    pub hidden: bool,

    pub minimum_size: Dimensions,
    pub expand_x: bool,
    pub expand_y: bool,

    pub grow_x: GrowDirection,
    pub grow_y: GrowDirection,

    pub anchor_left: f32,
    pub anchor_right: f32,
    pub anchor_top: f32,
    pub anchor_bottom: f32,

    pub margin_left: Scalar,
    pub margin_right: Scalar,
    pub margin_top: Scalar,
    pub margin_bottom: Scalar,
}

impl Default for SlotInfo {
    fn default() -> SlotInfo {
        SlotInfo {
            hidden: false,
            minimum_size: Dimensions::zero(),
            expand_x: false,
            expand_y: false,
            grow_x: GrowDirection::End,
            grow_y: GrowDirection::End,
            anchor_left: 0.0,
            anchor_right: 0.0,
            anchor_top: 0.0,
            anchor_bottom: 0.0,
            margin_left: scalar::ZERO,
            margin_right: scalar::ZERO,
            margin_top: scalar::ZERO,
            margin_bottom: scalar::ZERO,
        }
    }
}

pub struct Slot {
    pub info: SlotInfo,
    pub bounds: Bounds,
    pub widget_id: Option<WidgetId>,
    parent: Option<SlotId>,
    children: Vec<SlotId>,
}

impl Slot {
    pub fn children(&self) -> &[SlotId] {
        &self.children
    }
}

pub struct Slots {
    map: HashMap<SlotId, Slot>,
    next_slot_id: SlotId,
    dirty: bool,
}

impl Slots {
    pub fn add(&mut self, parent_id: SlotId, info: SlotInfo) -> SlotId {
        let slot_id = self.next_slot_id;
        self.next_slot_id.0 += 1;
        self.map.insert(
            slot_id,
            Slot {
                bounds: Bounds::zero(),
                parent: Some(parent_id),
                children: Vec::new(),
                widget_id: None,
                info,
            },
        );
        let parent_slot = self.get_mut(parent_id);
        parent_slot.children.push(slot_id);
        slot_id
    }

    pub fn set_size(&mut self, slot_id: SlotId, size: Dimensions) {
        let slot = self.get_mut(slot_id);
        slot.bounds.size = size;
        self.dirty = true;
    }

    pub fn get(&self, slot_id: SlotId) -> &Slot {
        &self.map[&slot_id]
    }

    pub fn get_mut(&mut self, slot_id: SlotId) -> &mut Slot {
        self.dirty = true;
        self.map.get_mut(&slot_id).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleFocusDirection {
    Previous,
    Next,
}

pub struct Gui<C: Context> {
    pub slots: Slots,
    widgets: HashMap<WidgetId, Box<dyn Widget<C>>>,
    slot_style_overrides: HashMap<SlotId, HashMap<String, C::StyleFieldValue>>,
    root_slot_id: SlotId,
    next_widget_id: WidgetId,
    focused_slot_id: Option<SlotId>,
    dirty: bool,
}

impl<C: Context> Gui<C> {
    pub fn new(root_bounds: Bounds) -> Gui<C> {
        let root_slot_id = SlotId(0);
        let mut slots = HashMap::new();
        slots.insert(
            root_slot_id,
            Slot {
                bounds: Bounds::zero(),
                parent: None,
                children: Vec::new(),
                widget_id: None,
                info: SlotInfo {
                    hidden: false,
                    minimum_size: root_bounds.size,
                    expand_x: false,
                    expand_y: false,
                    grow_x: GrowDirection::End,
                    grow_y: GrowDirection::End,
                    anchor_left: 0.0,
                    anchor_right: 1.0,
                    anchor_top: 0.0,
                    anchor_bottom: 1.0,
                    margin_left: scalar::ZERO,
                    margin_right: scalar::ZERO,
                    margin_top: scalar::ZERO,
                    margin_bottom: scalar::ZERO,
                },
            },
        );
        Gui {
            slots: Slots {
                map: slots,
                next_slot_id: SlotId(1),
                dirty: true,
            },
            slot_style_overrides: HashMap::new(),
            widgets: HashMap::new(),
            root_slot_id,
            next_widget_id: WidgetId(0),
            focused_slot_id: None,
            dirty: true,
        }
    }

    pub fn override_slot_style<T: Into<String>, U: Into<C::StyleFieldValue>>(
        &mut self,
        slot_id: SlotId,
        field_name: T,
        value: U,
    ) {
        self.slot_style_overrides
            .entry(slot_id)
            .or_insert_with(HashMap::new)
            .insert(field_name.into(), value.into());
    }

    fn find_focusable_recursive(&self, slot_id: SlotId, reverse: bool) -> Option<SlotId> {
        let slot = self.slots.get(slot_id);
        if let Some(widget_id) = slot.widget_id {
            let widget = &self.widgets[&widget_id];
            if widget.takes_focus() {
                return Some(slot_id);
            }
        }
        // TODO: Probably can do this without allocating?
        let mut children = slot.children().to_vec();
        if reverse {
            children.reverse();
        }
        for child_id in children {
            if let Some(found) = self.find_focusable_recursive(child_id, reverse) {
                return Some(found);
            }
        }
        None
    }

    pub fn cycle_focus(&mut self, direction: CycleFocusDirection) {
        if let Some(focused_slot_id) = self.focused_slot_id {
            let mut current_slot_id = focused_slot_id;
            while let Some(parent_slot_id) = self.slots.get(current_slot_id).parent {
                let parent_slot = self.slots.get(parent_slot_id);
                let index_in_parent = parent_slot
                    .children()
                    .iter()
                    .position(|&s| s == current_slot_id)
                    .unwrap();
                // TODO: Feels like there should be a way to do this without allocating.
                let candidate_slot_ids: Vec<SlotId> = match direction {
                    CycleFocusDirection::Next => {
                        parent_slot
                            .children()
                            .iter()
                            .cloned()
                            .skip(index_in_parent + 1)
                            .rev()
                            .collect()
                    }
                    CycleFocusDirection::Previous => {
                        parent_slot
                            .children()
                            .iter()
                            .cloned()
                            .take(index_in_parent)
                            .collect()
                    }
                };
                for candidate_slot_id in candidate_slot_ids.into_iter().rev() {
                    if let Some(slot_id) = self.find_focusable_recursive(candidate_slot_id, false) {
                        self.focused_slot_id = Some(slot_id);
                        return;
                    }
                }
                current_slot_id = parent_slot_id;
            }
        }
        let reverse = match direction {
            CycleFocusDirection::Next => false,
            CycleFocusDirection::Previous => true,
        };
        self.focused_slot_id = self.find_focusable_recursive(self.root_slot_id(), reverse);
    }

    pub fn set_focus(&mut self, slot_id: Option<SlotId>) {
        self.focused_slot_id = slot_id;
    }

    pub fn root_slot_id(&self) -> SlotId {
        self.root_slot_id
    }

    pub fn add_widget<W: Widget<C>>(&mut self, widget: W) -> WidgetId {
        let widget_id = self.next_widget_id;
        self.next_widget_id.0 += 1;
        self.widgets.insert(widget_id, Box::new(widget));
        widget_id
    }

    pub fn add_slot_with_widget<W: Widget<C>>(
        &mut self,
        parent_id: SlotId,
        slot_info: SlotInfo,
        widget: W,
    ) -> (SlotId, WidgetHandle<C, W>) {
        let slot_id = self.slots.add(parent_id, slot_info);
        let widget_id = self.add_widget(widget);
        let slot = self.slots.get_mut(slot_id);
        slot.widget_id = Some(widget_id);
        (slot_id, WidgetHandle(widget_id, PhantomData))
    }

    pub fn get_widget<W: Widget<C>>(&self, handle: WidgetHandle<C, W>) -> &W {
        self.widgets
            .get(&handle.0)
            .and_then(|w| w.downcast_ref())
            .unwrap()
    }

    pub fn get_widget_mut<W: Widget<C>>(&mut self, handle: WidgetHandle<C, W>) -> &mut W {
        // Very possible we will change something that could affect layout, so mark as dirty.
        self.dirty = true;
        self.widgets
            .get_mut(&handle.0)
            .and_then(|w| w.downcast_mut())
            .unwrap()
    }

    fn calculate_minimum_sizes_recursive(
        &mut self,
        slot_id: SlotId,
        theme: &Theme<C::ThemeResources, C::StyleFieldValue>,
        cache: &mut HashMap<SlotId, Dimensions>,
    ) {
        let (children, widget_id) = {
            let slot = self.slots.get(slot_id);
            (slot.children.clone(), slot.widget_id)
        };
        // Update children first, since parent minimum size may depend on children.
        for child_id in children {
            self.calculate_minimum_sizes_recursive(child_id, theme, cache);
        }
        let slot = self.slots.get(slot_id);
        if let Some(widget_id) = widget_id {
            let widget = self.widgets.get_mut(&widget_id).unwrap();
            let style_overrides = self.slot_style_overrides.get(&slot_id);
            let style = SlotStyle {
                widget_kind_id: widget.kind_id(),
                theme,
                field_overrides: style_overrides,
            };
            let args = MinimumSizeArgs {
                slots: &self.slots,
                slot_id,
                minimum_size_cache: cache,
                resources: &theme.resources,
                style: &style,
            };
            let widget_minimum_size = widget.minimum_size(args);
            let slot = self.slots.get(slot_id);
            let minimum_size = Dimensions::new(
                slot.info.minimum_size.width.max(widget_minimum_size.width),
                slot.info.minimum_size.height.max(widget_minimum_size.height),
            );
            cache.insert(slot_id, minimum_size);
        } else {
            cache.insert(slot_id, slot.info.minimum_size);
        }
    }

    fn layout_recursive(
        &mut self,
        slot_id: SlotId,
        minimum_size_cache: &HashMap<SlotId, Dimensions>,
    ) {
        let (bounds, children) = {
            let slot = self.slots.get(slot_id);
            (slot.bounds, slot.children.clone())
        };

        // TODO: Cleanup
        // Do basic anchor/margin calculations first
        for child_id in &children {
            let child = self.slots.get_mut(*child_id);

            // First we calculate the size of this child.
            let child_size = {
                let left = (bounds.size.width as f32 * child.info.anchor_left) as Scalar;
                let right = (bounds.size.width as f32 * child.info.anchor_right) as Scalar;
                let top = (bounds.size.height as f32 * child.info.anchor_top) as Scalar;
                let bottom = (bounds.size.height as f32 * child.info.anchor_bottom) as Scalar;
                Dimensions::new(right - left, bottom - top)
            };

            let child_minimum_size = &minimum_size_cache[child_id];
            let (origin_top, origin_bottom) = match (
                child_minimum_size.height > child_size.height,
                child.info.grow_y,
            ) {
                (true, GrowDirection::Begin) => (
                    (bounds.size.height as f32 * child.info.anchor_top) as Scalar
                        - child_minimum_size.height
                        - child.info.margin_top
                        + child.info.margin_bottom,
                    (bounds.size.height as f32 * child.info.anchor_bottom) as Scalar,
                ),
                (true, GrowDirection::Both) => (
                    (bounds.size.height as f32 * child.info.anchor_top) as Scalar
                        - (child_minimum_size.height - child.info.margin_top
                            + child.info.margin_bottom)
                            / scalar::TWO,
                    (bounds.size.height as f32 * child.info.anchor_bottom) as Scalar
                        + (child_minimum_size.height - child.info.margin_top
                            + child.info.margin_bottom)
                            / scalar::TWO,
                ),
                _ => (
                    (bounds.size.height as f32 * child.info.anchor_top) as Scalar,
                    (bounds.size.height as f32 * child.info.anchor_bottom) as Scalar,
                ),
            };

            let (origin_left, origin_right) = match (
                child_minimum_size.width > child_size.height,
                child.info.grow_x,
            ) {
                (true, GrowDirection::Begin) => (
                    (bounds.size.width as f32 * child.info.anchor_left) as Scalar
                        - child_minimum_size.width
                        - child.info.margin_left
                        + child.info.margin_right,
                    (bounds.size.width as f32 * child.info.anchor_right) as Scalar,
                ),
                (true, GrowDirection::Both) => (
                    (bounds.size.width as f32 * child.info.anchor_left) as Scalar
                        - (child_minimum_size.width - child.info.margin_left
                            + child.info.margin_right)
                            / scalar::TWO,
                    (bounds.size.width as f32 * child.info.anchor_right) as Scalar
                        + (child_minimum_size.width - child.info.margin_left
                            + child.info.margin_right)
                            / scalar::TWO,
                ),
                _ => (
                    (bounds.size.width as f32 * child.info.anchor_left) as Scalar,
                    (bounds.size.width as f32 * child.info.anchor_right) as Scalar,
                ),
            };
            child.bounds.x = origin_left + child.info.margin_left;
            child.bounds.y = origin_top + child.info.margin_top;
            child.bounds.size.width =
                (origin_right - origin_left + child.info.margin_right - child.info.margin_left).max(
                    child_minimum_size.width);
            child.bounds.size.height = 
                (origin_bottom - origin_top + child.info.margin_bottom - child.info.margin_top).max(
                    child_minimum_size.height)
        }

        let slot = self.slots.get(slot_id);
        if let Some(widget_id) = slot.widget_id {
            let widget = &self.widgets[&widget_id];
            if !slot.children.is_empty() {
                let args = LayoutChildrenArgs {
                    slots: &mut self.slots,
                    slot_id,
                    minimum_size_cache,
                    phantom: PhantomData,
                };
                widget.layout_children(args);
            }
        }
        for child_id in &children {
            self.layout_recursive(*child_id, minimum_size_cache);
        }
    }

    /// Returns `true` if layout was needed and performed.
    pub fn layout_if_needed(
        &mut self,
        theme: &Theme<C::ThemeResources, C::StyleFieldValue>,
    ) -> bool {
        if self.dirty || self.slots.dirty {
            let mut minimum_size_cache = HashMap::new();
            self.calculate_minimum_sizes_recursive(
                self.root_slot_id,
                theme,
                &mut minimum_size_cache,
            );
            self.layout_recursive(self.root_slot_id, &minimum_size_cache);
            self.dirty = false;
            self.slots.dirty = false;
            true
        } else {
            false
        }
    }

    // TODO: Cleanup. This event offset shenanigans is extremely obnoxious...
    fn process_event_recursive(
        &mut self,
        slot_id: SlotId,
        theme: &Theme<C::ThemeResources, C::StyleFieldValue>,
        event: &mut C::InputEvent,
        signals: &mut Vec<(SlotId, Signal)>,
    ) {
        let (bounds, children) = {
            let slot = self.slots.get(slot_id);
            if slot.info.hidden {
                return;
            }
            (slot.bounds, slot.children.clone())
        };

        event.offset_coordinates(-bounds.x, -bounds.y);
        for child_id in children.iter().rev() {
            self.process_event_recursive(*child_id, theme, event, signals);
        }

        event.offset_coordinates(bounds.x, bounds.x);
        let slot = self.slots.get(slot_id);
        let slot_bounds = slot.bounds;
        if let Some(widget_id) = slot.widget_id {
            let widget = self.widgets.get_mut(&widget_id).unwrap();
            let focused = self.focused_slot_id == Some(slot_id);
            let bounds = Bounds::new(
                scalar::ZERO,
                scalar::ZERO,
                slot_bounds.size.width,
                slot_bounds.size.height,
            );
            event.offset_coordinates(-slot_bounds.x, -slot_bounds.y);
            let style_overrides = self.slot_style_overrides.get(&slot_id);
            let style = SlotStyle {
                widget_kind_id: widget.kind_id(),
                theme,
                field_overrides: style_overrides,
            };
            let args = ProcessEventArgs {
                slots: &mut self.slots,
                slot_id,
                bounds,
                event,
                focused,
                resources: &theme.resources,
                style: &style,
            };
            let event_result = widget.process_event(args);
            for signal in event_result.signals {
                signals.push((slot_id, signal));
            }
            event.offset_coordinates(slot_bounds.x, slot_bounds.y);
            if event_result.request_focus && widget.takes_focus() {
                self.focused_slot_id = Some(slot_id);
            }
        }
    }

    pub fn process_event(
        &mut self,
        theme: &Theme<C::ThemeResources, C::StyleFieldValue>,
        event: &mut C::InputEvent,
    ) -> Vec<(SlotId, Signal)> {
        let mut signals = Vec::new();
        self.process_event_recursive(self.root_slot_id, theme, event, &mut signals);
        if event.dirty() {
            self.dirty = true;
        }
        signals
    }

    pub fn draw(
        &self,
        theme: &Theme<C::ThemeResources, C::StyleFieldValue>,
        draw_context: C::DrawContext,
    ) -> Vec<C::DrawCommand> {
        let mut draw_commands = Vec::new();
        let mut slot_ids = vec![(self.root_slot_id, (scalar::ZERO, scalar::ZERO), draw_context)];
        while let Some((slot_id, parent_offset, mut draw_context)) = slot_ids.pop() {
            let slot = self.slots.get(slot_id);
            if slot.info.hidden {
                continue;
            }
            if let Some(widget_id) = slot.widget_id {
                let widget = &self.widgets[&widget_id];
                let style_overrides = self.slot_style_overrides.get(&slot_id);
                let style = SlotStyle {
                    widget_kind_id: widget.kind_id(),
                    theme,
                    field_overrides: style_overrides,
                };
                let focused = self.focused_slot_id == Some(slot_id);
                let bounds = Bounds::new(
                    slot.bounds.x + parent_offset.0,
                    slot.bounds.y + parent_offset.1,
                    slot.bounds.size.width,
                    slot.bounds.size.height,
                );
                let args = DrawArgs {
                    bounds,
                    focused,
                    resources: &theme.resources,
                    style: &style,
                    context: &mut draw_context,
                    commands: &mut draw_commands,
                };
                widget.draw(args);
            }
            for child_id in slot.children.iter().rev() {
                let child_offset_x = parent_offset.0 + slot.bounds.x;
                let child_offset_y = parent_offset.1 + slot.bounds.y;
                slot_ids.push((*child_id, (child_offset_x, child_offset_y), draw_context));
            }
        }
        draw_commands
    }
}
