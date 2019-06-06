use std::collections::HashMap;

pub struct SlotStyle<'a, R, S> {
    pub(super) widget_kind_id: &'static str,
    pub(super) theme: &'a Theme<R, S>,
    pub(super) field_overrides: Option<&'a HashMap<String, S>>,
}

impl<'a, R, S> SlotStyle<'a, R, S> {
    pub fn get_field(&self, name: &str) -> &S {
        self.get_field_opt(name).unwrap()
    }

    pub fn get_field_opt(&self, name: &str) -> Option<&S> {
        self.field_overrides
            .and_then(|overrides| overrides.get(name))
            .or_else(|| {
                let default_style = self.theme.get_widget_style(self.widget_kind_id);
                default_style.get(name)
            })
    }
}

pub struct Theme<R, S> {
    pub resources: R,
    default_widget_styles: HashMap<String, HashMap<String, S>>,
}

impl<R, S> Theme<R, S> {
    pub fn new(resources: R) -> Theme<R, S> {
        Theme {
            resources,
            default_widget_styles: HashMap::new(),
        }
    }

    pub fn set_widget_style(&mut self, kind_id: &str, style: HashMap<String, S>) {
        self.default_widget_styles
            .insert(kind_id.to_string(), style);
    }

    fn get_widget_style(&self, kind_id: &str) -> &HashMap<String, S> {
        self.default_widget_styles
            .get(kind_id)
            .ok_or_else(|| format!("theme missing style for widget: `{}`", kind_id))
            .unwrap()
    }
}
