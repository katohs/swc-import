use swc_plugin::{ast::*, plugin_transform};

pub struct TransformVisitor;




impl TransformVisitor {
    pub fn new() -> Self {
        TransformVisitor
    }

    fn visit_mut_module_items_to_transform_import(&mut self, module_body: &mut Vec<ModuleItem>) {
        let mut imports = Vec::new();

        for (index, item) in module_body.iter_mut().enumerate() {
            let replacement = match item {
                ModuleItem::ModuleDecl(decl) => match decl {
                    ModuleDecl::Import(import) => {
                        let spec = import.specifiers.iter_mut().next().unwrap();

                        let spec = match spec {
                            ImportSpecifier::Named(spec) => spec,
                            ImportSpecifier::Default(_spec) => continue,
                            ImportSpecifier::Namespace(_spec) => continue,
                        };
                        let replacement = ImportDecl {
                            span: swc_plugin::syntax_pos::DUMMY_SP,
                            specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
                                span: swc_plugin::syntax_pos::DUMMY_SP,
                                local: spec.local.clone(),
                            })],
                            src: Str::from(format!("@mui/material/{}", spec.local.as_ref())),
                            type_only: import.type_only,
                            asserts: import.asserts.clone(),
                        };

                        Some(replacement)
                    }
                    _ => None,
                }
                _ => None,
            };
            
            if let Some(replacement) = replacement {
                imports.push((index, replacement));
            }
        }
        
        for (index, import) in imports {
            module_body.remove(index);

            module_body.insert(0, ModuleItem::ModuleDecl(ModuleDecl::Import(import)));
        }
    
    }
}

impl VisitMut for TransformVisitor {

  

    fn visit_mut_module(&mut self, module: &mut Module) {
        self.visit_mut_module_items_to_transform_import(&mut module.body);
        module.visit_mut_children_with(self);
    }
    // Implement necessary visit_mut_* methods for actual custom transform.
    // A comprehensive list of possible visitor methods can be found here:
    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
}

/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually via
/// `__plugin_process_impl(
///     ast_ptr: *const u8,
///     ast_ptr_len: i32,
///     config_str_ptr: *const u8,
///     config_str_ptr_len: i32,
///     context_str_ptr: *const u8,
///     context_str_ptr_len: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */
///
/// if plugin need to handle low-level ptr directly. However, there are
/// important steps manually need to be performed like sending transformed
/// results back to host. Refer swc_plugin_macro how does it work internally.
#[plugin_transform]
pub fn process_transform(program: Program, _plugin_config: String, _context: String) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
}

#[cfg(test)]
mod transform_visitor_tests {
    use swc_ecma_transforms_testing::test;

    use super::*;

    fn transform_visitor() -> impl 'static + Fold + VisitMut + ::swc_ecma_visit::Fold {
        as_folder(TransformVisitor {})
    }


    test!(
        ::swc_ecma_parser::Syntax::Es(::swc_ecma_parser::EsConfig {
            jsx: true,
            ..Default::default()
        }),
        |_| transform_visitor(),
        use_proxy_macros,
        r#"
        import { Button } from '@mui/material'
        "#,
        r#"
        import Button from '@mui/material/Button';
        "#
    );
}
