use gltf::Gltf;

fn main() {
    let gltf = Gltf::open("/Users/john/Dev/Assets/glTF-Sample-Models/2.0/CesiumMan/glTF/CesiumMan.gltf").unwrap();
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!("Node #{} has {} children", node.index(), node.children().count(),);
        }
    }
}
