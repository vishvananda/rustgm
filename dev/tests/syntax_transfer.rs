use rustgm::syntax::{
    ast::{Kind, NodeId},
    expression::{self, parse_crate},
    token_tree::{Span, error},
    transfer,
};

#[test]
fn copied_arenas_preserve_grammar_edges_attributes_and_capture_spans() -> Result<(), String> {
    let source = r#"
#![allow(unused)]
/// Record documentation.
pub(crate) struct Record<'a, T: Copy, const N:usize> where T:Sized { value:&'a [T;N] }
enum E {A, B(u8), C{value:u8}=3}
union U {a:u32,b:f32}
trait Tr<T> {type A:Copy;const K:usize;fn run(&self,x:T)->Self::A;}
impl<T:Copy> Tr<T> for Record<'_,T,1> {type A=T;const K:usize=1;fn run(&self,x:T)->T{x}}
use crate::a::{self,b as c,*};
extern crate alloc as memory;
unsafe extern "C" {fn foreign(x:i32,...);static mut G:i32;}
type Callback=for<'a> unsafe extern "C" fn(&'a u8)->u8;
macro_rules! made {($x:expr)=>{{ $x }}}
fn body(mut x:i32)->i32 {
    let [a,rest @ ..]=[1,2,3];
    let Record{value:v,..}=record;
    let closure=move |v:i32|->i32 {v+x};
    let y=if let Some(ref mut v)=some {(*v as i32)+1} else {0};
    for value in [1,2] {x+=value;}
    'a: loop {while x>0 {x-=1;continue 'a;}break 'a;}
    match x {0..=1=>x, _ if true=>{return made!(y);}, _=>0}
}
"#;
    let ast = parse_crate(source).map_err(|e| format!("{e:?}"))?;
    let prefix = "fn initial(){}\n";
    let mut target = parse_crate(prefix).map_err(|e| format!("{e:?}"))?;
    let base = u32::try_from(target.nodes.len()).map_err(|e| e.to_string())?;
    let shift = u32::try_from(prefix.len()).map_err(|e| e.to_string())?;
    let capture_base = target.captures.len();
    let root = transfer::append(
        &ast,
        &mut target,
        |s| {
            Ok(Span {
                start: s.start + shift,
                end: s.end + shift,
            })
        },
        |v| Ok(v.clone()),
    )
    .map_err(|e| format!("{e:?}"))?;
    assert_eq!(root, NodeId(ast.root.0 + base));
    let mut original_children = Vec::new();
    let mut copied_children = Vec::new();
    for (index, node) in ast.nodes.iter().enumerate() {
        let old = NodeId(u32::try_from(index).map_err(|e| e.to_string())?);
        let new = NodeId(old.0 + base);
        assert_eq!(
            target.node(new).span,
            Span {
                start: node.span.start + shift,
                end: node.span.end + shift
            }
        );
        original_children.clear();
        copied_children.clear();
        ast.push_children(old, &mut original_children);
        target.push_children(new, &mut copied_children);
        assert_eq!(
            copied_children,
            original_children
                .iter()
                .map(|n| NodeId(n.0 + base))
                .collect::<Vec<_>>(),
            "{index}: {:?}",
            node.kind
        );
    }
    for (index, capture) in ast.captures.iter().enumerate() {
        let copied = &target.captures[capture_base + index];
        assert_eq!(capture.nodes().len(), copied.nodes().len());
        for (original, copied) in capture.nodes().iter().zip(copied.nodes()) {
            assert_eq!(
                copied.span,
                Span {
                    start: original.span.start + shift,
                    end: original.span.end + shift
                }
            );
        }
    }
    let text = format!("{prefix}{source}");
    let mut output = Vec::new();
    expression::render::write_ast(&text, &target, &mut output).map_err(|e| e.to_string())?;
    assert!(output.windows(b"Record".len()).any(|s| s == b"Record"));
    Ok(())
}

#[test]
fn repeated_occurrences_get_independent_nodes_without_reinterpreting_literals() -> Result<(), String>
{
    let ast = expression::parse("true + 7").map_err(|e| format!("{e:?}"))?;
    let mut destination = parse_crate("").map_err(|e| format!("{e:?}"))?;
    let first = transfer::append(&ast, &mut destination, Ok, |v| Ok(v.clone()))
        .map_err(|e| format!("{e:?}"))?;
    let second = transfer::append(&ast, &mut destination, Ok, |v| Ok(v.clone()))
        .map_err(|e| format!("{e:?}"))?;
    assert_ne!(first, second);
    assert_eq!(destination.literals.len(), 4);
    let Kind::Binary {
        left: first_left, ..
    } = destination.node(first).kind
    else {
        return Err("expected binary".into());
    };
    let Kind::Binary {
        left: second_left, ..
    } = destination.node(second).kind
    else {
        return Err("expected binary".into());
    };
    assert_ne!(first_left, second_left);
    assert_eq!(ast.literals.len(), 2);
    Ok(())
}

#[test]
fn failed_span_or_literal_conversion_restores_the_destination() -> Result<(), String> {
    let ast = expression::parse("1+2").map_err(|e| format!("{e:?}"))?;
    let mut destination = parse_crate("fn before(){}").map_err(|e| format!("{e:?}"))?;
    let nodes = destination.nodes.clone();
    let edges = destination.edges.clone();
    let items = destination.items.clone();
    let mut seen = 0;
    let result = transfer::append(
        &ast,
        &mut destination,
        |span| {
            seen += 1;
            if seen == 2 {
                Err(error(0, "test span rejection"))
            } else {
                Ok(span)
            }
        },
        |v| Ok(v.clone()),
    );
    assert!(result.is_err());
    assert_eq!(destination.nodes, nodes);
    assert_eq!(destination.edges, edges);
    assert_eq!(destination.items, items);
    assert!(
        transfer::append(&ast, &mut destination, Ok, |_| Err(error(
            0,
            "test literal rejection"
        )))
        .is_err()
    );
    assert_eq!(destination.nodes, nodes);
    assert!(destination.literals.is_empty());
    Ok(())
}
