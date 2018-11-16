extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .set_in_dir("src/parser/")
        .process_current_dir()
        .unwrap();
}
