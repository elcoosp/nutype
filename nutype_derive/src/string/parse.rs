use crate::common::parse::{parse_nutype_attributes, parse_value_as_number, try_unwrap_ident, split_and_parse, is_comma, is_eq};
use crate::models::{StringSanitizer, StringValidator};
use crate::string::models::NewtypeStringMeta;
use crate::string::models::RawNewtypeStringMeta;
use proc_macro2::{Span, TokenStream, TokenTree};

use super::models::{SpannedStringSanitizer, SpannedStringValidator};
use super::validate::validate_string_meta;

pub fn parse_attributes(input: TokenStream) -> Result<NewtypeStringMeta, syn::Error> {
    parse_raw_attributes(input).and_then(validate_string_meta)
}

fn parse_raw_attributes(input: TokenStream) -> Result<RawNewtypeStringMeta, syn::Error> {
    parse_nutype_attributes(parse_sanitize_attrs, parse_validate_attrs)(input)
}

fn parse_sanitize_attrs(stream: TokenStream) -> Result<Vec<SpannedStringSanitizer>, syn::Error> {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    split_and_parse(tokens, is_comma, parse_sanitize_attr)
}

fn parse_sanitize_attr(tokens: Vec<TokenTree>) -> Result<SpannedStringSanitizer, syn::Error> {
    let mut token_iter = tokens.iter();
    let token = token_iter.next();
    if let Some(TokenTree::Ident(ident)) = token {
        let san = match ident.to_string().as_ref() {
            "trim" => StringSanitizer::Trim,
            "lowercase" => StringSanitizer::Lowercase,
            "uppercase" => StringSanitizer::Uppercase,
            "with" => {
                {
                    // Take `=` sign
                    if let Some(eq_t) = token_iter.next() {
                        if !is_eq(eq_t) {
                            let span = ident.span().join(eq_t.span()).unwrap();
                            return Err(syn::Error::new(
                                span,
                                "Invalid syntax for `with`. Expected `=`, got `{eq_t}`",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new(
                            ident.span(),
                            "Invalid syntax for `with`. Missing `=`",
                        ));
                    }
                }

                // Preserve the rest as `custom_sanitizer_fn`
                let ts = TokenStream::from_iter(token_iter.cloned());
                StringSanitizer::With(ts)
            }
            unknown_sanitizer => {
                let msg = format!("Unknown sanitizer `{unknown_sanitizer}`");
                let error = syn::Error::new(ident.span(), msg);
                return Err(error);
            }
        };
        Ok(SpannedStringSanitizer {
            span: ident.span(),
            item: san,
        })
    } else {
        Err(syn::Error::new(Span::call_site(), "Invalid syntax."))
    }
}

fn parse_validate_attrs(stream: TokenStream) -> Result<Vec<SpannedStringValidator>, syn::Error> {
    let mut output = vec![];

    let mut token_iter = stream.into_iter();
    while let Some((validator, rest_iter)) = parse_validation_rule(token_iter)? {
        token_iter = rest_iter;
        output.push(validator);
    }

    Ok(output)
}

fn parse_validation_rule<ITER: Iterator<Item = TokenTree>>(
    mut iter: ITER,
) -> Result<Option<(SpannedStringValidator, ITER)>, syn::Error> {
    match iter.next() {
        Some(mut token) => {
            // Skip punctuations between validators
            if token.to_string() == "," {
                token = iter.next().unwrap();
            }

            let ident = try_unwrap_ident(token)?;
            match ident.to_string().as_ref() {
                "max_len" => {
                    let (value, iter) = parse_value_as_number(iter)?;
                    let validator = StringValidator::MaxLen(value);
                    let parsed_validator = SpannedStringValidator {
                        item: validator,
                        span: ident.span(),
                    };
                    Ok(Some((parsed_validator, iter)))
                }
                "min_len" => {
                    let (value, iter) = parse_value_as_number(iter)?;
                    let validator = StringValidator::MinLen(value);
                    let parsed_validator = SpannedStringValidator {
                        item: validator,
                        span: ident.span(),
                    };
                    Ok(Some((parsed_validator, iter)))
                }
                "present" => {
                    let validator = StringValidator::Present;
                    let parsed_validator = SpannedStringValidator {
                        item: validator,
                        span: ident.span(),
                    };
                    Ok(Some((parsed_validator, iter)))
                }
                validator => {
                    let msg = format!("Unknown validation rule `{validator}`");
                    let error = syn::Error::new(ident.span(), msg);
                    Err(error)
                }
            }
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_validate_attrs() {
        let tokens = quote!(max_len = 13, min_len = 7, present);
        let validators = parse_validate_attrs(tokens).unwrap();
        let validators: Vec<StringValidator> = validators.into_iter().map(|v| v.item).collect();
        assert_eq!(
            validators,
            vec![
                StringValidator::MaxLen(13),
                StringValidator::MinLen(7),
                StringValidator::Present,
            ]
        );
    }

    #[test]
    fn test_validate_attrs_with_errors() {
        let tokens = quote!(max_len = -1);
        assert!(parse_validate_attrs(tokens).is_err());

        let tokens = quote!(present = 3);
        assert!(parse_validate_attrs(tokens).is_err());
    }
}
