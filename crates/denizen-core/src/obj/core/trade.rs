//! `TradeTag`: a villager merchant trade, identified as `trade@trade[...]`.

use crate::obj::core::item::ItemTag;
use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use crate::tags::Attribute;
use std::any::Any;

/// A merchant trade: inputs and a result item.
#[derive(Clone, Debug)]
pub struct TradeTag {
    pub result: ItemTag,
    pub inputs: Vec<ItemTag>,
    pub max_uses: i64,
    pub uses: i64,
    pub experience_reward: i64,
    pub price_multiplier: f64,
    pub prefix: String,
}

impl Default for TradeTag {
    fn default() -> Self {
        TradeTag {
            result: ItemTag::air(),
            inputs: Vec::new(),
            max_uses: 7,
            uses: 0,
            experience_reward: 0,
            price_multiplier: 0.05,
            prefix: "trade".to_string(),
        }
    }
}

impl TradeTag {
    pub fn new() -> Self {
        TradeTag::default()
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 6 && s[..6].eq_ignore_ascii_case("trade@") {
            s = &s[6..];
        }
        let mut trade = TradeTag::new();
        if let Some(start) = s.find('[') {
            if s.ends_with(']') {
                let props = &s[start + 1..s.len() - 1];
                for entry in props.split(';') {
                    if let Some((k, v)) = entry.split_once('=') {
                        match k.trim().to_lowercase().as_str() {
                            "result" => trade.result = ItemTag::from_identify(v.trim()).unwrap_or_else(ItemTag::air),
                            "inputs" => {
                                trade.inputs = v.split('|').filter_map(|i| ItemTag::from_identify(i.trim())).collect();
                            }
                            "max_uses" => trade.max_uses = v.trim().parse().unwrap_or(7),
                            "uses" => trade.uses = v.trim().parse().unwrap_or(0),
                            "experience_reward" => trade.experience_reward = v.trim().parse().unwrap_or(0),
                            "price_multiplier" => trade.price_multiplier = v.trim().parse().unwrap_or(0.05),
                            _ => {}
                        }
                    }
                }
            }
        }
        Some(trade)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for TradeTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        let mut props = Vec::new();
        if !self.result.is_empty() {
            props.push(format!("result={}", self.result.identify()));
        }
        if !self.inputs.is_empty() {
            let ins: Vec<String> = self.inputs.iter().map(|i| i.identify()).collect();
            props.push(format!("inputs={}", ins.join("|")));
        }
        if self.max_uses != 7 {
            props.push(format!("max_uses={}", self.max_uses));
        }
        if self.uses != 0 {
            props.push(format!("uses={}", self.uses));
        }
        if self.experience_reward != 0 {
            props.push(format!("experience_reward={}", self.experience_reward));
        }
        if props.is_empty() {
            "trade@trade".to_string()
        } else {
            format!("trade@trade[{}]", props.join(";"))
        }
    }
    fn identify_simple(&self) -> String {
        "trade@trade".to_string()
    }
    fn object_type_name(&self) -> &'static str {
        "TradeTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_unique(&self) -> bool {
        false
    }
}

fn param_item(attr: &mut Attribute) -> Option<ItemTag> {
    let p = attr.get_param_object()?;
    ItemTag::from_identify(&p.as_element().value)
}

/// Register TradeTag's tags.
pub fn register_tags() {
    let p = get_processor("TradeTag");

    p.register_tag("ItemTag", "result", |_a, o| {
        o.downcast_ref::<TradeTag>().map(|t| t.result.obj())
    }, true, &[]);

    p.register_tag("ListTag", "inputs", |_a, o| {
        let t = o.downcast_ref::<TradeTag>()?;
        let items: Vec<ObjectTag> = t.inputs.iter().map(|i| i.obj()).collect();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &["ingredients"]);

    p.register_tag("ElementTag", "max_uses", |_a, o| {
        o.downcast_ref::<TradeTag>().map(|t| ObjectTag::new(ElementTag::from_int(t.max_uses)))
    }, true, &[]);

    p.register_tag("ElementTag", "uses", |_a, o| {
        o.downcast_ref::<TradeTag>().map(|t| ObjectTag::new(ElementTag::from_int(t.uses)))
    }, true, &[]);

    p.register_tag("ElementTag", "experience_reward", |_a, o| {
        o.downcast_ref::<TradeTag>().map(|t| ObjectTag::new(ElementTag::from_int(t.experience_reward)))
    }, true, &[]);

    p.register_tag("ElementTag", "price_multiplier", |_a, o| {
        o.downcast_ref::<TradeTag>().map(|t| ObjectTag::new(ElementTag::from_double(t.price_multiplier)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_tradeable", |_a, o| {
        let t = o.downcast_ref::<TradeTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(t.uses < t.max_uses && !t.result.is_empty())))
    }, true, &[]);

    p.register_tag_with_param("TradeTag", "ElementTag", "with_result", |_a, o, param| {
        let mut t = o.downcast_ref::<TradeTag>()?.clone();
        if let Some(item) = ItemTag::from_identify(&param.as_element().value) {
            t.result = item;
        }
        Some(t.obj())
    }, true, &[]);

    p.register_tag_with_param("TradeTag", "ElementTag", "with_inputs", |_a, o, param| {
        let mut t = o.downcast_ref::<TradeTag>()?.clone();
        t.inputs = param
            .as_element()
            .value
            .split('|')
            .filter_map(|i| ItemTag::from_identify(i.trim()))
            .collect();
        Some(t.obj())
    }, true, &[]);

    p.register_tag_with_param("TradeTag", "ElementTag", "with_max_uses", |_a, o, param| {
        let mut t = o.downcast_ref::<TradeTag>()?.clone();
        t.max_uses = param.as_element().to_int().unwrap_or(7);
        Some(t.obj())
    }, true, &[]);

    p.register_tag("ElementTag", "simple", |_a, o| {
        let t = o.downcast_ref::<TradeTag>()?;
        Some(ObjectTag::new(ElementTag::new(t.identify())))
    }, true, &[]);


    p.register_tag("ElementTag", "input_count", |_a, o| {
        o.downcast_ref::<TradeTag>().map(|t| ObjectTag::new(ElementTag::from_int(t.inputs.len() as i64)))
    }, true, &[]);

    p.register_tag("ItemTag", "input", |a, o| {
        let t = o.downcast_ref::<TradeTag>()?;
        let idx = a.get_param_object().and_then(|p| p.as_element().to_int()).unwrap_or(0);
        t.inputs.get(idx.max(0) as usize).map(|i| i.obj())
    }, true, &[]);

    p.register_tag("ElementTag", "has_result", |_a, o| {
        let t = o.downcast_ref::<TradeTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(!t.result.is_empty())))
    }, true, &[]);

    p.register_tag("ElementTag", "remaining_uses", |_a, o| {
        let t = o.downcast_ref::<TradeTag>()?;
        Some(ObjectTag::new(ElementTag::from_int((t.max_uses - t.uses).max(0))))
    }, true, &[]);

    p.register_tag_with_param("TradeTag", "ElementTag", "with_experience_reward", |_a, o, param| {
        let mut t = o.downcast_ref::<TradeTag>()?.clone();
        t.experience_reward = param.as_element().to_int().unwrap_or(0);
        Some(t.obj())
    }, true, &[]);

    let _ = param_item;
}
