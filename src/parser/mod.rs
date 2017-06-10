use chrono::prelude::*;
use chrono::{DateTime, NaiveDateTime};
use std::io::Read;
use xml5ever::rcdom::{RcDom, NodeData, Handle};
use xml5ever::{Attribute};
use xml5ever::tendril::{TendrilSink};
use xml5ever::driver::{parse_document};
use uuid::Uuid;
use feed::Feed;

mod rss2;
mod atom;

pub fn parse<R>(input: &mut R) -> Option<Feed> where R: Read {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(input)
        .unwrap();
    walk(dom.document)
}

fn walk(handle: Handle) -> Option<Feed> {
    let node = handle;
    match node.data {
        NodeData::Document => println!("#document"),
        NodeData::Element { ref name, ref attrs, .. } => {
            let tag_name = name.local.as_ref();
            let version  = attr("version", &attrs.borrow()).unwrap_or("".to_string());
            match (tag_name, version.as_ref()) {
                ("feed", _)    => return atom::handle_atom(node.clone()),
                ("rss", "2.0") => return rss2::handle_rss2(node.clone()),
                _ => (),
            }
        },
        _ => {},
    }
    for child in node.children.borrow().iter() {
        if let Some(feed) = walk(child.clone()) {
            return Some(feed)
        }
    }
    None
}

pub fn uuid_gen() -> String {
    Uuid::new_v4().to_string()
}

pub fn attr(attr_name: &str, attrs: &Vec<Attribute>) -> Option<String> {
    for attr in attrs.iter() {
        if attr.name.local.as_ref() == attr_name {
            return Some(attr.value.to_string())
        }
    }
    None
}

pub fn text(handle: Handle) -> Option<String> {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Text { ref contents } =>
                return Some(contents.borrow().to_string()),
            _ => (),
        }
    }
    return None
}

pub fn timestamp_from_rfc3339(handle: Handle) -> Option<NaiveDateTime> {
    text(handle)
        .and_then(|s| DateTime::parse_from_rfc3339(&s.trim()).ok())
        .map(|n| n.naive_utc())
}

pub fn timestamp_from_rfc2822(handle: Handle) -> Option<NaiveDateTime> {
    text(handle)
        .and_then(|s| DateTime::parse_from_rfc2822(&s.trim()).ok())
        .map(|n| n.naive_utc())
}
