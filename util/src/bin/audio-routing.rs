use std::fs;
use std::path;
use std::io::Read;

use clap::Parser;
use wwise_format::*;
use wwise_analysis::dictionary::parse_dictionary;
use wwise_analysis::label::get_label;
use wwise_analysis::audio_routable::get_output_nodes;
use tabbycat::{GraphBuilder, GraphType, Identity, StmtList, Edge, AttrType, AttrList, SubGraph};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    dictionary: path::PathBuf,

    #[arg(short, long, num_args = 0..)]
    soundbanks: Vec<path::PathBuf>,
}

fn main() {
    let args = Arguments::parse();

    let dictionary_file = fs::read_to_string(args.dictionary)
        .expect("Could not read dictionary");
    let dictionary = parse_dictionary(&dictionary_file);

    let mut stmt = StmtList::new()
        .add_attr(
            AttrType::Graph,
            AttrList::default()
                .add(
                    Identity::String("splines".into()),
                    Identity::String("ortho".into()),
                )
                .add(
                    Identity::String("ranksep".into()),
                    Identity::quoted("4.0 equally"),
                )
                .add(
                    Identity::String("colorscheme".into()),
                    Identity::String("oranges9".into()),
                )
                .add(
                    Identity::String("concentrate".into()),
                    Identity::String("false".into()),
                )
        )
        .add_attr(
            AttrType::Node,
            AttrList::default()
                .add(
                    Identity::String("shape".into()),
                    Identity::String("record".into()),
                )
        );

    for path in args.soundbanks {
        let mut handle = fs::File::open(&path)
            .expect("Could not acquire file handle");

        let mut file_buffer = vec![];
        handle.read_to_end(&mut file_buffer)
            .expect("Could not read input file");

        let bnk_name = path.file_name()
            .unwrap()
            .to_string_lossy();

        let parsed = wwise_format::parse_soundbank(&file_buffer)
            .expect("Could not parse bnk");

        let hirc = match get_hirc(&parsed) {
            None => continue,
            Some(h) => h,
        };

        let mut subgraph_stmt = StmtList::new()
            .add_attr(
                AttrType::Graph,
                AttrList::default()
                    .add(
                        Identity::String("label".into()),
                        Identity::quoted(bnk_name.to_string())
                    )
                    .add(
                        Identity::String("style".into()),
                        Identity::String("filled".into())
                    )
                    .add(
                        Identity::String("color".into()),
                        Identity::String("lightgrey".into())
                    )
                    .add(
                        Identity::String("concentrate".into()),
                        Identity::Bool(true),
                    )
            )
            .add_attr(
                AttrType::Node,
                AttrList::default()
                    .add(
                        Identity::String("style".into()),
                        Identity::String("filled".into())
                    )
                    .add(
                        Identity::String("color".into()),
                        Identity::String("white".into())
                    )
            );

        for object in hirc.objects.iter() {
            let output_nodes = match get_output_nodes(object) {
                Some(r) => r,
                None => continue,
            };

            subgraph_stmt = subgraph_stmt.add_node(
                Identity::from(object.id.as_hash()),
                None,
                Some(
                     AttrList::default()
                        .add(
                            Identity::String("label".into()),
                            Identity::quoted(get_label(object, Some(&dictionary))),
                        )
                )
            );

            for output_node in output_nodes.into_iter() {
                subgraph_stmt = subgraph_stmt.add_edge(
                    Edge::head_node(
                        Identity::from(object.id.as_hash()),
                        None
                    )
                    .arrow_to_node(
                        Identity::from(output_node),
                        None
                        //Some(Port::Compass(Compass::North))
                    )
                )
            }
        }

        let subgraph = SubGraph::subgraph(Some(Identity::quoted(format!("cluster_{}", bnk_name))), subgraph_stmt);

        stmt = stmt.add_subgraph(subgraph);
    }

    let graph = GraphBuilder::default()
        .graph_type(GraphType::DiGraph)
        .strict(true)
        .id(Identity::id("G").unwrap())
        .stmts(stmt)
        .build()
        .unwrap();

    println!("{}", graph);
}

fn get_hirc(sb: &Soundbank) -> Option<&HIRCSection> {
    for section in sb.sections.iter() {
        if let SectionBody::HIRC(h) = &section.body {
            return Some(h)
        }
    }

    None
}
