/*
Dioxus is a react-like framework for Rust.

Instead of adopting a pure virtualdom, it uses a new technique called Template Dom.

In template composition, new nodes are never created at runtime. Instead, a template is defined at compile time
and then compared against other templates at runtime. Only the dynamic aspects of each template are compared.

This strategy combines the ergonomics of virtualdom with the performance of purely reactive systems like SolidJS.

For most apps, this will be the best of both worlds. For some apps that involve a *lot* of dynamic content, it may
not be the best choice.

However, the costs of diffing and re-rendering said dynamic content is likely to be more than adopting the template dom.

It's important to re-think composition of elements in a template system.



set_dynamic_attribute(instance: 0, attr: 0, name: "asd", value: "asd")
set_dynamic_attribute(instance: 0, attr: 1, name: "asd", value: "asd")

load_template(instance: 0, id: "jon/box.rs:32");     |
load_template(instance: 1, id: "jon/cube.rs:100");   |
replace_element(instance: 0, el: "aaabcde");         |
load_template(instance: 1, id: "jon/cube.rs:100");   |
replace_element(instance: 0, el: "aaabcdf");         |

*/

use arena::Arena;
use mutation::Mutation;
use node::{DynamicNode, VTemplate};

use crate::node::{Attribute, AttributeLocation, Template, TemplateAttribute, TemplateNode};

mod arena;
mod diff;
mod mutation;
mod node;

#[derive(Default)]
pub struct VirtualDom {
    arena: Arena,
}

impl VirtualDom {
    pub fn create<'a>(&mut self, mutations: &mut Vec<Mutation<'a>>, template: &'a VTemplate<'a>) {
        let id = self.arena.next();

        mutations.push(Mutation::LoadTemplate {
            name: template.template.id,
            id,
        });

        for (idx, dyn_node) in template.dynamic_attrs.iter().enumerate() {
            let id = self.arena.next();

            let path = template.template.attr_pathways[idx];
            mutations.push(Mutation::AssignId { path, id });

            for attr in dyn_node.attrs {
                mutations.push(Mutation::SetAttribute {
                    name: attr.name,
                    value: attr.value,
                    id,
                });
            }
        }

        for (idx, dyn_node) in template.dynamic_nodes.iter().enumerate() {
            match dyn_node {
                DynamicNode::Component { name } => todo!("not yet"),
                DynamicNode::Text { value, id } => {
                    let id = self.arena.next();
                    let path = template.template.node_pathways[idx];
                    mutations.push(Mutation::HydrateText { path, value, id });
                }
                DynamicNode::Fragment { children } => todo!(),
            }
        }
    }

    pub fn diff<'a>(
        &mut self,
        mutations: &mut Vec<Mutation<'a>>,
        left: &'a VTemplate<'a>,
        right: &'a VTemplate<'a>,
    ) {
        if left.template.id != right.template.id {
            self.create(mutations, right);
            let id = left.node_id.get();
            mutations.push(Mutation::Replace { id });
            return;
        }

        // Set the attributes
        for (left_node, right_node) in left.dynamic_attrs.iter().zip(right.dynamic_attrs.iter()) {
            for (left, right) in left_node.attrs.iter().zip(right_node.attrs.iter()) {
                // use ptr shortcircuting before the memcmp
                if !std::ptr::eq(left.value, right.value) && left.value != right.value {
                    mutations.push(Mutation::SetAttribute {
                        id: right_node.mounted_element.get(),
                        name: left.name,
                        value: left.value,
                    });
                }
            }
        }

        for (left, right) in left.dynamic_nodes.iter().zip(right.dynamic_nodes.iter()) {
            match (left, right) {
                (DynamicNode::Component { .. }, DynamicNode::Component { .. }) => todo!(),
                (
                    DynamicNode::Text {
                        value: v1, id: id1, ..
                    },
                    DynamicNode::Text {
                        value: v2, id: id2, ..
                    },
                ) => {
                    id2.set(id1.get());

                    if v1 != v2 {
                        mutations.push(Mutation::SetText {
                            id: id2.get(),
                            value: v2,
                        });
                    }
                }
                (
                    DynamicNode::Fragment { children: c1 },
                    DynamicNode::Fragment { children: c2 },
                ) => {
                    // todo: keyed diffing
                    for (left, right) in c1.iter().zip(c2.iter()) {
                        self.diff(mutations, left, right);
                    }
                }
                _ => todo!(),
            }
        }
    }
}

#[test]
fn makes_muts() {
    let mut dom = VirtualDom::default();
    let mut mutations = Vec::default();

    static TEMPLATE: Template = Template {
        id: "123",
        root: TemplateNode::Element {
            tag: "div",
            namespace: None,
            attrs: &[
                TemplateAttribute {
                    name: "class",
                    value: "an amazing class",
                    namespace: None,
                    volatile: false,
                },
                TemplateAttribute {
                    name: "id",
                    value: "an amazing id",
                    namespace: None,
                    volatile: false,
                },
            ],
            children: &[
                //
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::Text("some basic text 123"),
                TemplateNode::DynamicText(0),
                TemplateNode::DynamicText(1),
                TemplateNode::DynamicText(2),
            ],
        },

        node_pathways: &[&[0], &[1], &[2]],
        attr_pathways: &[&[0]],
    };

    let template = VTemplate {
        node_id: Default::default(),
        template: TEMPLATE,
        dynamic_nodes: &[
            DynamicNode::Text {
                value: "abc",
                id: Default::default(),
            },
            DynamicNode::Text {
                value: "def",
                id: Default::default(),
            },
            DynamicNode::Text {
                value: "efg",
                id: Default::default(),
            },
        ],
        dynamic_attrs: &[AttributeLocation {
            mounted_element: Default::default(),
            attrs: &[Attribute {
                name: "hidden",
                value: "true",
                namespace: None,
            }],
        }],
    };

    dom.create(&mut mutations, &template);
    dbg!(&mut mutations).clear();

    dom.diff(&mut mutations, &template, &template);
    dbg!(&mut mutations).clear();

    let template_new = VTemplate {
        node_id: Default::default(),
        template: TEMPLATE,
        dynamic_nodes: &[
            DynamicNode::Text {
                value: "bork",
                id: Default::default(),
            },
            DynamicNode::Text {
                value: "def",
                id: Default::default(),
            },
            DynamicNode::Text {
                value: "efg",
                id: Default::default(),
            },
        ],
        dynamic_attrs: &[],
    };

    dom.diff(&mut mutations, &template, &template_new);
    dbg!(&mut mutations).clear();
}

#[test]
fn fragments_too() {
    //
}

#[test]
fn two_strs_same_ptr() {}
