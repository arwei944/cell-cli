use syn::File;
use syn::visit::Visit;

#[derive(Debug, Default)]
pub struct CodeStructure {
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub traits: Vec<TraitInfo>,
    pub imports: Vec<String>,
    pub total_lines: usize,
    pub total_functions: usize,
    pub total_structs: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub params_count: usize,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub line: usize,
    pub fields_count: usize,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub line: usize,
    pub variants_count: usize,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub line: usize,
    pub methods_count: usize,
}

pub fn parse_rust_file(source: &str) -> Result<CodeStructure, syn::Error> {
    let syntax: File = syn::parse_str(source)?;
    let total_lines = source.lines().count();

    let mut visitor = RustVisitor::default();
    visitor.visit_file(&syntax);

    let total_functions = visitor.functions.len();
    let total_structs = visitor.structs.len();

    Ok(CodeStructure {
        functions: visitor.functions,
        structs: visitor.structs,
        enums: visitor.enums,
        traits: visitor.traits,
        imports: visitor.imports,
        total_lines,
        total_functions,
        total_structs,
    })
}

#[derive(Debug, Default)]
struct RustVisitor {
    functions: Vec<FunctionInfo>,
    structs: Vec<StructInfo>,
    enums: Vec<EnumInfo>,
    traits: Vec<TraitInfo>,
    imports: Vec<String>,
}

impl<'ast> Visit<'ast> for RustVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let is_public = matches!(node.vis, syn::Visibility::Public(_));
        let is_async = node.sig.asyncness.is_some();
        let params_count = node.sig.inputs.len();
        let line = node.sig.ident.span().start().line;

        self.functions.push(FunctionInfo {
            name: node.sig.ident.to_string(),
            line,
            is_public,
            is_async,
            params_count,
        });

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let is_public = matches!(node.vis, syn::Visibility::Public(_));
        let fields_count = match &node.fields {
            syn::Fields::Named(named) => named.named.len(),
            syn::Fields::Unnamed(unnamed) => unnamed.unnamed.len(),
            syn::Fields::Unit => 0,
        };
        let line = node.ident.span().start().line;

        self.structs.push(StructInfo {
            name: node.ident.to_string(),
            line,
            fields_count,
            is_public,
        });

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let line = node.ident.span().start().line;
        self.enums.push(EnumInfo {
            name: node.ident.to_string(),
            line,
            variants_count: node.variants.len(),
        });

        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let line = node.ident.span().start().line;
        self.traits.push(TraitInfo {
            name: node.ident.to_string(),
            line,
            methods_count: node.items.len(),
        });

        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.imports.push(quote::quote!(#node).to_string());
        syn::visit::visit_item_use(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = r#"
fn hello() {
    println!("hello");
}
"#;
        let result = parse_rust_file(source).unwrap();
        assert_eq!(result.total_functions, 1);
        assert_eq!(result.functions[0].name, "hello");
        assert!(!result.functions[0].is_public);
        assert!(!result.functions[0].is_async);
    }

    #[test]
    fn test_parse_public_async_function() {
        let source = r#"
pub async fn fetch_data(url: String) -> Result<(), ()> {
    Ok(())
}
"#;
        let result = parse_rust_file(source).unwrap();
        assert_eq!(result.total_functions, 1);
        assert_eq!(result.functions[0].name, "fetch_data");
        assert!(result.functions[0].is_public);
        assert!(result.functions[0].is_async);
        assert_eq!(result.functions[0].params_count, 1);
    }

    #[test]
    fn test_parse_struct() {
        let source = r#"
pub struct User {
    name: String,
    age: u32,
}
"#;
        let result = parse_rust_file(source).unwrap();
        assert_eq!(result.total_structs, 1);
        assert_eq!(result.structs[0].name, "User");
        assert!(result.structs[0].is_public);
        assert_eq!(result.structs[0].fields_count, 2);
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"
enum Color {
    Red,
    Green,
    Blue,
}
"#;
        let result = parse_rust_file(source).unwrap();
        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.enums[0].name, "Color");
        assert_eq!(result.enums[0].variants_count, 3);
    }

    #[test]
    fn test_parse_trait() {
        let source = r#"
trait Repository {
    fn save(&self);
    fn load(&self);
}
"#;
        let result = parse_rust_file(source).unwrap();
        assert_eq!(result.traits.len(), 1);
        assert_eq!(result.traits[0].name, "Repository");
        assert_eq!(result.traits[0].methods_count, 2);
    }

    #[test]
    fn test_parse_invalid_rust() {
        let source = "this is not valid rust {{{";
        let result = parse_rust_file(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_total_lines() {
        let source = "fn a() {}\nfn b() {}\nfn c() {}\n";
        let result = parse_rust_file(source).unwrap();
        assert_eq!(result.total_lines, 3);
    }
}
