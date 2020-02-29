use std::{collections::HashMap, env, fs, io::Read, process, rc::Rc};

use tests_loader::GenCTest as _;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("{}: arguments error", args[0]);
        process::exit(1);
    }
    let ast = {
        let filepath = &args[1];
        use codegen::ast::HasName;
        let ast = codegen::Parser::parse(filepath);
        ast.decls()
            .iter()
            .map(|decl| (decl.name().to_owned(), Rc::clone(decl)))
            .collect::<HashMap<_, _>>()
    };
    let test_data = {
        let filepath = &args[2];
        let mut file = fs::File::open(filepath)
            .unwrap_or_else(|err| panic!("failed to open tests from {}: {}", filepath, err));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|err| panic!("failed to load tests from {}: {}", filepath, err));
        contents
    };
    let all: tests_loader::TestSet = serde_yaml::from_str(&test_data)
        .unwrap_or_else(|err| panic!("failed to parse tests: {}", err));
    let tests = all.iter().map(|any| any.gen_test(&ast)).collect::<Vec<_>>();

    println!(
        r#"#include "tests-utils.h"
#define test_build_for(Name)                                        \
    uint32_t size = sizeof(expected);                               \
    char *name = #Name;                                             \
    if (res.errno != MOL_OK) {{                                      \
        printf("Error %s: failed to build\n", name);                \
        free(res.seg.ptr);                                          \
        return 1;                                                   \
    }} else if (res.seg.size != size) {{                              \
        printf("Error %s: size is not match (%d != %d)\n",          \
                name, res.seg.size, size);                          \
        free(res.seg.ptr);                                          \
        return 1;                                                   \
    }} else if (memcmp(res.seg.ptr, expected, size) != 0) {{          \
        printf("Error %s: content is not match\n", name);           \
        free(res.seg.ptr);                                          \
        return 1;                                                   \
    }}                                                               \
    mol_errno errno = MolReader_ ## Name ## _verify(&res.seg,false);\
    if (errno != MOL_OK) {{                                          \
        printf("Error %s: failed to verify (%d)\n", name, errno);   \
        free(res.seg.ptr);                                          \
        return 1;                                                   \
    }}"#
    );
    println!();
    for (id, stmts) in tests.iter().enumerate() {
        println!("uint32_t test_{} () {{", id);
        for stmt in stmts.iter() {
            println!("    {}", stmt);
        }
        println!("}}");
    }
    println!("int main(int argc, char *argv[]) {{");
    println!("    test_start(\"Test Vector ({})\");", args[2]);
    println!("    int failed_cnt = 0;");
    let cnt = tests.len();
    for id in 0..cnt {
        println!("    if (test_{}() != 0) {{", id);
        println!("        printf(\"test test_{} ... failed\\n\");", id);
        println!("        failed_cnt += 1;");
        println!("    }}");
    }
    println!("    if (failed_cnt != 0) {{");
    println!(
        "        printf(\"[Error] %d/{} tests are failed.\\n\", failed_cnt);",
        cnt
    );
    println!("        return 1;");
    println!("    }} else {{");
    println!("        printf(\"ALL tests are passed ({}).\\n\");", cnt);
    println!("    }}");
    println!("    return 0;");
    println!("}}");
}
