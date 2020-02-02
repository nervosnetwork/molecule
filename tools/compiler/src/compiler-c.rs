use molecule_codegen::{IntermediateFormat, Language};

fn main() {
    moleculec::build_commandline(Language::C, IntermediateFormat::JSON).execute();
}
