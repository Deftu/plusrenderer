//! Per-frame cost of the player with the demo emote and a hat: `cargo run --release --example bench`.

use std::hint::black_box;
use std::time::Instant;

use plusrenderer::{java, Arms, BodySlot, Camera, Scene};

const EMOTE: &str = include_str!("../../../packages/demo/public/emote.json");
const FRAMES: u32 = 20000;

const HAT: &str = r#"{"format_version":"1.12.0","minecraft:geometry":[{
    "description":{"identifier":"geometry.hat","texture_width":64,"texture_height":64},
    "bones":[{"name":"brim","pivot":[0,24,0],"cubes":[
      {"origin":[-5,32,-5],"size":[10,1,10],"uv":[0,0]},
      {"origin":[-3,33,-3],"size":[6,4,6],"uv":[0,16]},
      {"origin":[-1,37,-1],"size":[2,2,2],"rotation":[0,45,0],"uv":[0,32]}]}]}]}"#;

fn main() {
    let mut scene = Scene::default();
    scene.load_player(0, Arms::Wide);
    scene.load_animation(0, EMOTE).expect("emote");
    let hat = plusrenderer::bedrock::parse_one(HAT).expect("hat");
    scene.load_cosmetic(1, hat, BodySlot::Hat, None, None);
    scene.play(0, 0.0);

    let mut time = |name: &str, frame: &mut dyn FnMut(&mut Scene, f32)| {
        let start = Instant::now();
        for i in 0..FRAMES {
            frame(&mut scene, i as f32 / 60.0);
        }
        let per = start.elapsed() / FRAMES;
        println!("{name}: {per:?} per frame");
    };

    time("pose", &mut |s, t| s.pose(t));
    let mut tris = 0;
    time("pose + build", &mut |s, t| {
        s.pose(t);
        let camera = Camera::orbit(t * 40.0, 12.0, 40.0, [0.0, 18.0, 0.0]);
        tris += black_box(s.build(&camera, [512.0, 512.0])).indices.len() / 3;
    });
    println!("  {} triangles per frame", tris / FRAMES as usize);
    time("pose + build_mesh", &mut |s, t| {
        s.pose(t);
        black_box(s.build_mesh(&java::MODEL));
    });
}
