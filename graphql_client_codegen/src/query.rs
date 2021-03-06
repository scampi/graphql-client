use deprecation::DeprecationStrategy;
use failure;
use fragments::GqlFragment;
use itertools::Itertools;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use schema::Schema;
use selection::Selection;
use std::collections::BTreeMap;
use syn::Ident;

/// This holds all the information we need during the code generation phase.
pub(crate) struct QueryContext {
    pub fragments: BTreeMap<String, GqlFragment>,
    pub schema: Schema,
    pub deprecation_strategy: DeprecationStrategy,
    variables_derives: Vec<Ident>,
    response_derives: Vec<Ident>,
}

impl QueryContext {
    /// Create a QueryContext with the given Schema.
    pub(crate) fn new(schema: Schema, deprecation_strategy: DeprecationStrategy) -> QueryContext {
        QueryContext {
            fragments: BTreeMap::new(),
            schema,
            deprecation_strategy,
            variables_derives: vec![Ident::new("Serialize", Span::call_site())],
            response_derives: vec![Ident::new("Deserialize", Span::call_site())],
        }
    }

    pub(crate) fn require(&self, typename_: &str) {
        if let Some(fragment) = self.fragments.get(typename_) {
            fragment.is_required.set(true)
        }
    }

    /// For testing only. creates an empty QueryContext with an empty Schema.
    #[cfg(test)]
    pub(crate) fn new_empty() -> QueryContext {
        QueryContext {
            fragments: BTreeMap::new(),
            schema: Schema::new(),
            deprecation_strategy: DeprecationStrategy::Allow,
            variables_derives: vec![Ident::new("Serialize", Span::call_site())],
            response_derives: vec![Ident::new("Deserialize", Span::call_site())],
        }
    }

    pub(crate) fn maybe_expand_field(
        &self,
        ty: &str,
        selection: &Selection,
        prefix: &str,
    ) -> Result<TokenStream, failure::Error> {
        if let Some(enm) = self.schema.enums.get(ty) {
            enm.is_required.set(true);
            Ok(quote!()) // we already expand enums separately
        } else if let Some(obj) = self.schema.objects.get(ty) {
            obj.is_required.set(true);
            obj.response_for_selection(self, &selection, prefix)
        } else if let Some(iface) = self.schema.interfaces.get(ty) {
            iface.is_required.set(true);
            iface.response_for_selection(self, &selection, prefix)
        } else if let Some(unn) = self.schema.unions.get(ty) {
            unn.is_required.set(true);
            unn.response_for_selection(self, &selection, prefix)
        } else {
            Ok(quote!())
        }
    }

    pub(crate) fn ingest_additional_derives(
        &mut self,
        attribute_value: &str,
    ) -> Result<(), failure::Error> {
        if self.response_derives.len() > 1 {
            return Err(format_err!(
                "ingest_additional_derives should only be called once"
            ));
        }

        self.variables_derives.extend(
            attribute_value
                .split(',')
                .map(|s| s.trim())
                .map(|s| Ident::new(s, Span::call_site())),
        );
        self.response_derives.extend(
            attribute_value
                .split(',')
                .map(|s| s.trim())
                .map(|s| Ident::new(s, Span::call_site())),
        );
        Ok(())
    }

    pub(crate) fn variables_derives(&self) -> TokenStream {
        let derives = self.variables_derives.iter().unique();

        quote! {
            #[derive( #(#derives),* )]
        }
    }

    pub(crate) fn response_derives(&self) -> TokenStream {
        let derives = self.response_derives.iter().unique();

        quote! {
            #[derive( #(#derives),* )]
        }
    }

    pub(crate) fn response_enum_derives(&self) -> TokenStream {
        let enum_derives: Vec<_> = self
            .response_derives
            .iter()
            .filter(|derive| {
                !derive.to_string().contains("erialize")
                    && !derive.to_string().contains("Deserialize")
            }).collect();

        if !enum_derives.is_empty() {
            quote! {
                #[derive( #(#enum_derives),* )]
            }
        } else {
            quote!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_derives_ingestion_works() {
        let mut context = QueryContext::new_empty();

        context
            .ingest_additional_derives("PartialEq, PartialOrd, Serialize")
            .unwrap();

        assert_eq!(
            context.response_derives().to_string(),
            "# [ derive ( Deserialize , PartialEq , PartialOrd , Serialize ) ]"
        );
    }

    #[test]
    fn response_enum_derives_does_not_produce_empty_list() {
        let context = QueryContext::new_empty();
        assert_eq!(context.response_enum_derives().to_string(), "");
    }

    #[test]
    fn response_enum_derives_works() {
        let mut context = QueryContext::new_empty();

        context
            .ingest_additional_derives("PartialEq, PartialOrd, Serialize")
            .unwrap();

        assert_eq!(
            context.response_enum_derives().to_string(),
            "# [ derive ( PartialEq , PartialOrd ) ]"
        );
    }

    #[test]
    fn response_derives_fails_when_called_twice() {
        let mut context = QueryContext::new_empty();

        assert!(
            context
                .ingest_additional_derives("PartialEq, PartialOrd")
                .is_ok()
        );
        assert!(context.ingest_additional_derives("Serialize").is_err());
    }
}
