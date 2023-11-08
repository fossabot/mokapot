
use crate::elements::{Class, Method};

use super::stack_frame::StackFrameAnalyzer;

fn get_test_class() -> Class {
    let bytes = include_bytes!(concat!(
        env!("OUT_DIR"),
        "/java_classes/org/mokapot/test/TestAnalysis.class"
    ));
    Class::from_reader(&bytes[..]).unwrap()
}

fn get_test_method() -> Method {
    let class = get_test_class();
    class
        .methods
        .into_iter()
        .find(|it| it.name == "test")
        .unwrap()
}

#[test]
fn load_test_method() {
    get_test_method();
}

#[test]
fn analyze(){
    let method = get_test_method();
    let analyzer = StackFrameAnalyzer::default();
    let defs = analyzer.definitions(&method);
    println!("{:?}", defs);
}