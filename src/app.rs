use std::cmp::Ordering;
use tui::widgets::ListState;
use crate::asn1_der::{Asn1Error, ASN1Node, flatten_nodes, parse_asn1};

#[derive(Debug)]
pub struct App {
    pub(crate) view: Vec<ASN1Node>,
    pub(crate) nodes: Vec<ASN1Node>,
    pub(crate) state: ListState,
}

impl App {
    pub(crate) fn new(input: Vec<u8>) -> Result<Self, Asn1Error> {
        let mut root = parse_asn1(&input, 0)?;
        root.visible = true;

        let mut state = ListState::default();
        state.select(Some(0));

        let mut nodes: Vec<ASN1Node> = flatten_nodes(&root);
        let mut i = 0;
        while i < nodes.len() {
            nodes[i].index = i;
            i = i + 1;
        }

        let app = App {
            view: get_view(nodes.clone()),
            nodes: nodes.clone(),
            state,
        };

        Ok(app)
    }

    pub(crate) fn toggle_selected(&mut self) {
        if let Some(selected_index) = self.state.selected() {
            let node_index = self.view[selected_index].index;
            if self.nodes[node_index].expandable {
                self.nodes[node_index].toggle_expand();
                self.update_nodes(node_index);
                self.view = get_view(self.nodes.clone());
            }
        }
    }

    fn update_nodes(&mut self, index: usize) {
        let mut i = index + 1;
        let next_level = self.nodes[index].level + 1;

        loop {
            if i >= self.nodes.len() {
                break;
            }
            match self.nodes[i].level.cmp(&next_level) {
                Ordering::Equal | Ordering::Greater => {
                    if !self.nodes[index].expanded {
                        self.nodes[i].visible = false;
                    } else {
                        self.nodes[i].expanded = false;
                        if self.nodes[i].level > next_level {
                            self.nodes[i].visible = false;
                        } else {
                            self.nodes[i].visible = true;
                        }
                    }
                }
                Ordering::Less => break,
            };
            i = i + 1;
        }
    }

    pub(crate) fn copy_selected_value(&self) -> Option<String> {
        self.state.selected().map(|selected_index| {
            let node_index = self.view[selected_index].index;
            self.nodes[node_index].display_value()
        })
    }

    pub(crate) fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.view.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub(crate) fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.view.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub(crate) fn first(&mut self) {
        self.state.select(Some(0));
    }

    pub(crate) fn last(&mut self) {
        self.state.select(Some(self.view.len() - 1));
    }
}

fn get_view(nodes: Vec<ASN1Node>) -> Vec<ASN1Node> {
    nodes
        .into_iter()
        .filter(|node| node.visible)
        .collect()
}