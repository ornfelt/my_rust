extern crate mysql;
use mysql::*;
use mysql::prelude::*;
use std::error::Error;

// Define the Node struct here
#[derive(Debug)]
struct Node {
    id: u32,
    x: f64,
    y: f64,
    z: f64,
    links: String,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Define the connection URL
    let url = "mysql://trinity:trinity@localhost:3306/world";

    // Establish the connection
    let pool = Pool::new(url)?;
    let mut conn = pool.get_conn()?;

    println!("Connected to the database.");

    // Define the query
    let query = r"
        SELECT id, x, y, z, links
        FROM creature_template_npcbot_wander_nodes
        WHERE mapid = 0
    ";

    // Execute the query and map the results to a Vec of Node structs
    let nodes: Vec<Node> = conn.query_map(
        query,
        |(id, x, y, z, links)| {
            Node { id, x, y, z, links }
        },
    )?;

    println!("Retrieved {} nodes.", nodes.len());

    let my_position = Node {
        id: 0,
        x: 1.0,
        y: 1.0,
        z: 1.0,
        links: String::new(),
    };

    if let Some(closest_node) = get_closest_node(&my_position, &nodes) {
        println!("Closest Node: {:?}", closest_node);
    } else {
        println!("No nodes found.");
    }

    Ok(())
}

// Now `Node` is defined and can be used in this function
fn get_closest_node<'a>(my_position: &Node, nodes: &'a [Node]) -> Option<&'a Node> {
    nodes.iter().min_by(|a, b| {
        let dist_a = distance(my_position, a);
        let dist_b = distance(my_position, b);
        dist_a.partial_cmp(&dist_b).unwrap()
    })
}

fn distance(a: &Node, b: &Node) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

// use: 'cargo tree' to visualize dependencies 

