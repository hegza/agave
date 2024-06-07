mod replace;

pub use replace::*;
pub use roxmltree;

use itertools::Itertools;
use roxmltree::Node;

pub const PERIPHERAL_TAG: &str = "peripheral";
pub const CLUSTER_TAG: &str = "cluster";
pub const REGISTER_TAG: &str = "register";

const NAME_TAG: &str = "name";

pub fn get_descendants_by_tag<'n>(root: Node<'n, 'n>, tag: &str) -> Vec<Node<'n, 'n>> {
    root.descendants()
        .filter(|node| (node.tag_name().name() == tag))
        .collect_vec()
}

pub fn get_children_by_tag<'n>(root: Node<'n, 'n>, tag: &str) -> Vec<Node<'n, 'n>> {
    root.children()
        .filter(|node| (node.tag_name().name() == tag))
        .collect_vec()
}

pub fn get_svd_parent_chain(node: Node) -> Vec<String> {
    let mut parent_chain = vec![];
    let mut cur_node = node;
    loop {
        let tag_name = cur_node.tag_name().name();
        if [PERIPHERAL_TAG, CLUSTER_TAG, REGISTER_TAG].contains(&tag_name) {
            let name = get_name(cur_node).to_string();
            parent_chain.push(name);
        }

        match cur_node.parent() {
            Some(parent) => cur_node = parent,
            None => break,
        }
    }
    parent_chain.reverse();
    parent_chain
}

pub fn get_name<'n>(node: Node<'n, 'n>) -> &'n str {
    let name_node = *get_descendants_by_tag(node, NAME_TAG).first().unwrap();
    let cl_name = name_node.text().unwrap();
    cl_name
}

pub fn interpret_svd_num(s: &str) -> u64 {
    if &s[0..2] == "0x" || &s[0..2] == "0X" {
        u64::from_str_radix(&s[2..], 16).unwrap()
    } else if &s[0..1] == "#" {
        u64::from_str_radix(&s[1..], 16).unwrap()
    }
    // Decimal
    else {
        u64::from_str_radix(s, 10).unwrap()
    }
}

pub fn get_address_offset(cl: Node) -> u64 {
    let address_offset_nodes = get_descendants_by_tag(cl, "addressOffset");
    let address_offset_node = address_offset_nodes.first().unwrap();
    let text = address_offset_node
        .children()
        .nth(0)
        .unwrap()
        .text()
        .unwrap();
    let address_offset = interpret_svd_num(text);
    address_offset
}

pub fn create_jenga_op_for_cluster(cl: roxmltree::Node) -> ReplaceRange {
    // Make sure we're not trying to remove a cluster with a non-zero offset
    let address_offset = get_address_offset(cl);
    assert!(address_offset == 0);

    // Remove the cluster
    let cl_range = cl.range();

    // Add back the registers directly underneath the cluster
    let reg_ranges = get_children_by_tag(cl, REGISTER_TAG)
        .into_iter()
        .map(|reg_node| reg_node.range())
        .collect_vec();

    ReplaceRange {
        remove_range: cl_range,
        add_ranges: reg_ranges,
    }
}
