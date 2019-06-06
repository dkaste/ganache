use std::collections::HashMap;

use crate::{scalar, Scalar, SlotId, Slots, Dimensions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

pub fn default(
    slots: &mut Slots,
    slot_id: SlotId,
    minimum_size_cache: &HashMap<SlotId, Dimensions>,
    layout_axis: Axis,
    padding: Scalar,
    child_spacing: Scalar,
) {
    let bounds = slots.get(slot_id).bounds;
    let children = crate::visible_children(slots, slot_id).collect::<Vec<_>>();
    let num_children = children.len();
    let size = match layout_axis {
        Axis::Horizontal => bounds.size.width,
        Axis::Vertical => bounds.size.height,
    };
    let available_size = size - padding * scalar::TWO - child_spacing * ((num_children - 1) as Scalar);

    let mut irregular_sizes = HashMap::new();
    let mut num_regular_children = num_children;
    let mut reserved_size = scalar::ZERO;

    let mut expand_children = Vec::new();
    // Children not set as `expand` will always be their minimum size.
    for child_id in children.iter().rev() {
        let child = slots.get(*child_id);
        let child_minimum_size = minimum_size_cache[child_id];
        let (axis_expand, minimum_size) = match layout_axis {
            Axis::Horizontal => (child.info.expand_x, child_minimum_size.width),
            Axis::Vertical => (child.info.expand_y, child_minimum_size.height),
        };
        if !axis_expand {
            irregular_sizes.insert(*child_id, minimum_size);
            num_regular_children -= 1;
            reserved_size += minimum_size;
        } else {
            expand_children.push((*child_id, minimum_size));
        }
    }

    // Children set as `expand` will be at least their minimum size or at most the "regular" size.
    // (The leftover size that is distributed equally (for now) between children marked `expand`.)
    // We need to figure out which children will be irregularly-sized.
    //
    // It makes sense to check this on children ordered from largest to smallest minimum sizes.
    expand_children.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
    for (child_id, minimum_size) in expand_children.into_iter().rev() {
        let regular_size = if num_regular_children > 0 {
            (available_size - reserved_size) / num_regular_children as Scalar
        } else {
            scalar::ZERO
        };
        if minimum_size > regular_size {
            irregular_sizes.insert(child_id, minimum_size);
            num_regular_children -= 1;
            reserved_size += minimum_size;
        }
    }

    let regular_size = if num_regular_children > 0 {
        (available_size - reserved_size) / num_regular_children as Scalar
    } else {
        scalar::ZERO
    };
    let mut offset = padding;
    for (i, child_id) in children.iter().enumerate() {
        let child = slots.get_mut(*child_id);
        let other_expand = match layout_axis {
            Axis::Horizontal => child.info.expand_y,
            Axis::Vertical => child.info.expand_x,
        };
        let child_size = irregular_sizes
            .get(child_id)
            .cloned()
            .unwrap_or(regular_size);
        let child_minimum_size = minimum_size_cache[child_id];
        match layout_axis {
            Axis::Horizontal => {
                child.bounds.x = offset;
                child.bounds.y = padding;
                child.bounds.size.width = child_size.max(child_minimum_size.width);
                child.bounds.size.height = if other_expand {
                    (bounds.size.height - (padding * scalar::TWO)).max(child_minimum_size.height)
                } else {
                    child_minimum_size.height
                };
                offset += child.bounds.size.width;
            }
            Axis::Vertical => {
                child.bounds.x = padding;
                child.bounds.y = offset;
                child.bounds.size.width = if other_expand {
                    (bounds.size.width - (padding * scalar::TWO)).max(child_minimum_size.width)
                } else {
                    child_minimum_size.width
                };
                child.bounds.size.height = child_size.max(child_minimum_size.height);
                offset += child.bounds.size.height;
            }
        }
        if i < children.len() - 1 {
            offset += child_spacing;
        }
    }
}
