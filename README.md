# Molecule

[![License]](#license)

Another serialization system: minimalist and canonicalization.

[License]: https://img.shields.io/badge/License-MIT-blue.svg

## Grammar of the Schema Language

```ebnf
zero                    =   "0";
nonzero                 =   "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
digit                   =   zero | nonzero;
lowercase               =   "a" | "b" | "c" | "d" | "e" | "f" | "g"
                          | "h" | "i" | "j" | "k" | "l" | "m" | "n"
                          | "o" | "p" | "q"       | "r" | "s" | "t"
                          | "u" | "v" | "w"       | "x" | "y" | "z";
uppercase               =   "A" | "B" | "C" | "D" | "E" | "F" | "G"
                          | "H" | "I" | "J" | "K" | "L" | "M" | "N"
                          | "O" | "P" | "Q"       | "R" | "S" | "T"
                          | "U" | "V" | "W"       | "X" | "Y" | "Z";
letter                  =   lowercase | uppercase;
ifs                     =   " " | "\t";
newline                 =   "\n" | "\r\n";

identifier              =   letter , { letter | digit | "_" };
number                  =   nonzero, { digit };

whitespace              =   ifs | newline;
break                   =   whitespace, { whitespace };
break_opt               =   { whitespace };

field_end               =   ",";
stmt_end                =   ";";

field_decl              =   identifier, break, ":", break_opt,
                            identifier, break_opt,
                            field_end;
array_decl              =   "array", break, identifier, break_opt,
                            "[", break_opt,
                                identifier, break_opt, ";", break_opt, number, break_opt,
                            "]", break_opt,
                            stmt_end;
struct_decl             =   "struct", break, identifier, break_opt,
                            "{", break_opt,
                                field_decl, break_opt,
                                { field_decl, break_opt },
                            "}";
vector_decl             =   "vector", break, identifier, break_opt,
                            "<", break_opt,
                                identifier, break_opt,
                            ">", break_opt,
                            stmt_end;
table_decl              =   "table", break, identifier, break_opt,
                            "{", break_opt,
                                field_decl, break_opt,
                                { field_decl, break_opt },
                            "}";
decl_stmt               =   array_decl | struct_decl | vector_decl | table_decl;

path_super              =   "../";
path                    =   { path_super }, { identifier, "/" }, identifier;
import_stmt             =   "import", break, path, break_opt, stmt_end;

grammar                 =   break_opt,
                            { import_stmt, break_opt },
                            decl_stmt,
                            { break_opt, decl_stmt }
                            break_opt;
```

## License

Licensed under [MIT License].

[MIT License]: LICENSE
