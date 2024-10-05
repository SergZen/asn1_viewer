use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem;
use der::{Decode, Result, asn1::{Any, ObjectIdentifier}, Tagged, Tag, Reader};
use der::asn1::{BitString, Ia5String, OctetString, PrintableString, UtcTime};
use log::{warn};

#[derive(Debug)]
pub struct Asn1Error(pub der::Error);

impl Error for Asn1Error {}

impl From<der::Error> for Asn1Error {
    fn from(error: der::Error) -> Self {
        Asn1Error(error)
    }
}

impl From<ASN1Node> for ASN1Value {
    fn from(node: ASN1Node) -> Self {
        node.value
    }
}

impl Display for Asn1Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ASN.1 Error: {:?}", self.0)
    }
}

#[derive(Clone, Debug)]
pub enum ASN1Value {
    Boolean(bool),
    Integer(u128),
    BitString(BitString),
    OctetString(Box<ASN1Node>),
    Null,
    ObjectIdentifier(ObjectIdentifier),
    Utf8String(Vec<u8>),
    Sequence(Vec<ASN1Node>),
    Set(Box<ASN1Node>),
    PrintableString(PrintableString),
    Ia5String(Ia5String),
    UtcTime(UtcTime),
    Application(Box<ASN1Node>),
    ContextSpecific(Box<ASN1Node>),
    Private(Box<ASN1Node>),
    Other(String),
}

#[derive(Clone, Debug)]
pub struct ASN1Node {
    pub(crate) tag: Tag,
    pub(crate) value: ASN1Value,
    pub(crate) level: u32,
    pub(crate) expandable: bool,
    pub(crate) expanded: bool,
    pub(crate) visible: bool,
    pub(crate) index: usize,
}

impl ASN1Node {
    pub(crate) fn new(tag: Tag, value: ASN1Value, level: u32, expandable: bool, visible: bool) -> Self {
        ASN1Node {
            tag,
            value,
            level,
            expandable,
            expanded: false,
            visible,
            index: 0
        }
    }

    pub(crate) fn toggle_expand(&mut self) {
        if let ASN1Value::Sequence(_) |
        ASN1Value::Set(_) |
        ASN1Value::OctetString(_) |
        ASN1Value::ContextSpecific(_) |
        ASN1Value::Application(_) |
        ASN1Value::Private(_) = self.value {
            self.expanded = !self.expanded;
        }
    }

    pub(crate) fn display_value(&self) -> String {
        match &self.value {
            ASN1Value::Integer(i) => format!("{:?}", i),
            ASN1Value::OctetString(_) => "".to_string(),
            ASN1Value::Sequence(children) => format!("(field(s): {:?})", children.len()),
            ASN1Value::Set(_) => "".to_string(),
            ASN1Value::Boolean(b) => format!("{}", b),
            ASN1Value::Null => "Null".to_string(),
            ASN1Value::ObjectIdentifier(oid) => format!("{}", oid.to_string()),
            ASN1Value::Utf8String(vec) => {
                let string = String::from_utf8(vec.clone()).unwrap_or_else(|_| "".to_string());
                format!("{:?}", string)
            }
            ASN1Value::PrintableString(s) => format!("{:?}", s.to_string()),
            ASN1Value::Ia5String(s) => format!("{:?}", s.to_string()),
            ASN1Value::UtcTime(utc) => format!("{:?}", utc.to_date_time().to_string()),
            ASN1Value::BitString(bits) => {
                let bits_string = bits.raw_bytes().iter()
                    .map(|&byte| format!("{:08b}", byte))
                    .collect::<Vec<String>>()
                    .join("");
                format!("{}", bits_string)
            }
            ASN1Value::ContextSpecific(_) => "".to_string(),
            ASN1Value::Application(_) => "".to_string(),
            ASN1Value::Private(_) => "".to_string(),
            ASN1Value::Other(s) => format!("{:?}", s.to_string()),
        }
    }

    pub(crate) fn get_view_content(&mut self) -> String {
        let level_repeat = " ".repeat((self.level * 2) as usize);
        let expand = if self.expandable { if self.expanded { "- " } else { "+ " } } else { "  " };
        format!("{} {} {} {}", expand, level_repeat, self.tag.to_string(), self.display_value())
    }
}

pub fn parse_asn1(data: &[u8], level: u32) -> Result<ASN1Node> {
    let mut data_vec = data.to_vec();
    match data_vec[0] {
        // fix der for parsing GeneralString
        27 => mem::swap(&mut data_vec[0], &mut 19),
        _ => {}
    }
    let data = data_vec.as_slice();

    let any: Any = Any::from_der(data).unwrap_or_else(|_| {
        Any::null()
    });

    Ok(parse_any(any, level)?)
}

fn parse_any(any: Any, level: u32) -> Result<ASN1Node> {
    let tag = any.tag();
    let value = match tag {
        Tag::Integer => ASN1Value::Integer(any.decode_as::<u128>()?),
        Tag::OctetString => {
            let inner = any.decode_as::<OctetString>()?;
            let parsed_inner = parse_asn1(inner.as_bytes(), level + 1)?;
            ASN1Value::OctetString(Box::new(parsed_inner))
        }
        Tag::Sequence => {
            any.sequence(|decoder| {
                let mut children: Vec<ASN1Node> = Vec::new();
                while !decoder.is_finished() {
                    let child: Any = decoder.decode()?;
                    let parsed_child: ASN1Node = parse_any(child.clone(), level + 1)?;
                    children.push(parsed_child);
                }
                Ok::<ASN1Value, der::Error>(ASN1Value::Sequence(children))
            })?
        }
        Tag::Set => {
            let inner: Any = any.decode_as()?;
            let parsed_inner = parse_asn1(inner.value(), level + 1)?;
            ASN1Value::Set(Box::new(parsed_inner))
        }
        Tag::BitString => ASN1Value::BitString(any.decode_as::<BitString>()?),
        Tag::Boolean => ASN1Value::Boolean(any.decode_as::<bool>()?),
        Tag::Null => ASN1Value::Null,
        Tag::ObjectIdentifier => ASN1Value::ObjectIdentifier(any.decode_as::<ObjectIdentifier>()?),
        Tag::Utf8String => ASN1Value::Utf8String(String::from_utf8_lossy(any.value()).to_string().into()),
        Tag::Ia5String => ASN1Value::Ia5String(any.decode_as::<Ia5String>()?),
        Tag::UtcTime => ASN1Value::UtcTime(any.decode_as::<UtcTime>()?),
        Tag::PrintableString => ASN1Value::PrintableString(any.decode_as::<PrintableString>()?),
        Tag::Application { .. } => {
            let inner: Any = any.decode_as()?;
            let parsed_inner = parse_asn1(inner.value(), level + 1)?;
            Ok::<ASN1Value, der::Error>(ASN1Value::Application(Box::new(parsed_inner)))
        }?,
        Tag::ContextSpecific { .. } => {
            let inner: Any = any.decode_as()?;
            if inner.tag().is_constructed() {
                let parsed_inner = parse_asn1(inner.value(), level + 1)?;
                Ok::<ASN1Value, der::Error>(ASN1Value::ContextSpecific(Box::new(parsed_inner)))
            } else {
                let node = ASN1Node::new(
                    Tag::Utf8String,
                    ASN1Value::Utf8String(Vec::from(inner.value())),
                    level + 1,
                    false,
                    false
                );
                Ok::<ASN1Value, der::Error>(ASN1Value::ContextSpecific(Box::new(node)))
            }
        }?,
        Tag::Private { .. } => {
            let inner: Any = any.decode_as()?;
            let parsed_inner = parse_asn1(inner.value(), level + 1)?;
            Ok::<ASN1Value, der::Error>(ASN1Value::Private(Box::new(parsed_inner)))
        }?,
        _ => {
            warn!("Unsupported tag: {:?}", tag.to_string());
            ASN1Value::Other(format!("Unsupported tag: {:?}", tag.to_string()))
        }
    };
    Ok(ASN1Node::new(tag, value, level, is_node_expandable(tag), false))
}

fn is_node_expandable(tag: Tag) -> bool {
    match tag {
        Tag::Sequence |
        Tag::Set |
        Tag::OctetString |
        Tag::ContextSpecific { .. } |
        Tag::Application { .. } |
        Tag::Private { .. }
        => true,

        _ => false,
    }
}

pub(crate) fn flatten_nodes(node: &ASN1Node) -> Vec<ASN1Node> {
    let mut nodes = vec![node.clone()];

    match &node.value {
        ASN1Value::Sequence(children) => {
            for child in children {
                nodes.extend(flatten_nodes(&child));
            }
        }
        ASN1Value::Set(child) |
        ASN1Value::OctetString(child) |
        ASN1Value::ContextSpecific(child) |
        ASN1Value::Application(child) |
        ASN1Value::Private(child) => {
            nodes.extend(flatten_nodes(child.as_ref()));
        }
        _ => {}
    }

    nodes
}