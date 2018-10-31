# graphql_client

[![Build Status](https://travis-ci.org/graphql-rust/graphql-client.svg?branch=master)](https://travis-ci.org/graphql-rust/graphql-client)
[![docs](https://docs.rs/graphql_client/badge.svg)](https://docs.rs/graphql_client/latest/graphql_client/)
[![crates.io](https://img.shields.io/crates/v/graphql_client.svg)](https://crates.io/crates/graphql_client)
[![Join the chat](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/juniper-graphql/graphql-client)

A typed GraphQL client library for Rust.

## Features

- Precise types for query variables and responses
- Supports GraphQL fragments, objects, unions, inputs, enums, custom scalars and input objects
- Works in the browser (WebAssembly)
- Subscriptions support (serialization-deserialization only at the moment)
- Copies documentation from the GraphQL schema to the generated Rust code
- Arbitrary derives on the generated responses
- Arbitrary custom scalars
- Supports multiple operations per query document
- Supports setting GraphQL fields as deprecated and having the Rust compiler check
  their use.

## Getting started

- If you are not familiar with GraphQL, the [official website](https://graphql.org/) provides a very good and comprehensive introduction.

- Once you have written your query (most likely in something like [graphiql](https://github.com/graphql/graphiql)), save it in a `.graphql` file in your project.

- In order to provide precise types for a response, graphql_client needs to read the query and the schema at compile-time.

  To download the schema, you have multiple options. This projects provides a [CLI](https://github.com/graphql-rust/graphql-client/tree/master/graphql_client_cli), but there are also more mature tools like [apollo-cli](https://github.com/apollographql/apollo-cli). It does not matter which one you use, the resulting `schema.json` is the same.

- We now have everything we need to derive Rust types for our query. This is achieved through a procedural macro, as in the following snippet:

  ```rust
  extern crate serde;
  #[macro_use]
  extern crate serde_derive;
  #[macro_use]
  extern crate graphql_client;

  // The paths are relative to the directory where your `Cargo.toml` is located.
  // Both json and the GraphQL schema language are supported as sources for the schema
  #[derive(GraphQLQuery)]
  #[graphql(
      schema_path = "src/graphql/schema.json",
      query_path = "src/graphql/queries/my_query.graphql",
  )]
  pub struct MyQuery;
  ```

  The `derive` will generate a module named `my_query` in this example - the name is the struct's name, but in snake case.

  That module contains all the struct and enum definitions necessary to deserialize a response to that query.

  The root type for the response is named `ResponseData`. The GraphQL response will take the form of a `Response<ResponseData>` (the [Response](https://docs.rs/graphql_client/latest/graphql_client/struct.Response.html) type is always the same).

  The module also contains a struct called `Variables` representing the variables expected by the query.

* We now need to create the complete payload that we are going to send to the server. For convenience, the [GraphQLQuery trait](https://docs.rs/graphql_client/latest/graphql_client/trait.GraphQLQuery.html), is implemented for the struct under derive, so a complete query body can be created this way:

  ```rust
  extern crate failure;
  #[macro_use]
  extern crate graphql_client;
  extern crate reqwest;

  use graphql_client::{GraphQLQuery, Response};

  fn perform_my_query(variables: &my_query::Variables) -> Result<(), failure::Error> {

      // this is the important line
      let request_body = MyQuery::build_query(variables);

      let client = reqwest::Client::new();
      let mut res = client.post("/graphql").json(&request_body).send()?;
      let response_body: Response<my_query::ResponseData> = res.json()?;
      println!("{:#?}", response_body);
      Ok(())
  }
  ```

[A complete example using the GitHub GraphQL API is available](https://github.com/graphql-rust/graphql-client/tree/master/graphql_client/examples/github), as well as sample [rustdoc output](https://www.tomhoule.com/docs/example_module/).

## Deriving specific traits on the response

The generated response types always derive `serde::Deserialize` but you may want to print them (`Debug`), compare them (`PartialEq`) or derive any other trait on it. You can achieve this with the `response_derives` option of the `graphql` attribute. Example:

```rust
#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/search_schema.graphql",
  query_path = "src/search_query.graphql"
  response_derives = "Serialize,PartialEq",
)]
struct SearchQuery;
```

## Custom scalars

The generated code will reference the scalar types as defined in the server schema. This means you have to provide matching rust types in the scope of the struct under derive. It can be as simple as declarations like `type Email = String;`. This gives you complete freedom on how to treat custom scalars, as long as they can be deserialized.

## Deprecations

The generated code has support for [`@deprecated`](http://facebook.github.io/graphql/June2018/#sec-Field-Deprecation)
field annotations. You can configure how deprecations are handled via the `deprecated` argument in the `GraphQLQuery` derive:

```rust
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/my_query.graphql",
    deprecated = "warn"
)]
pub struct MyQuery;
```

Valid values are:

- `allow`: the response struct fields are not marked as deprecated.
- `warn`: the response struct fields are marked as `#[deprecated]`.
- `deny`: The struct fields are not included in the response struct and
  using them is a compile error.

The default is `warn`.

## Query documents with multiple operations

You can write multiple operations in one query document (one `.graphql` file). You can then select one by naming the struct you `#[derive(GraphQLQuery)]` on with the same name as one of the operations. This is neat, as it allows sharing fragments between operations.

There is an example [in the tests](./graphql_client/tests/operation_selection).

## Documentation for the generated modules

You can use `cargo doc --document-private-items` to generate rustdoc documentation on the generated code.

## Make cargo recompile when .graphql files have changed

There is an [`include`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields-optional) option you can add to your `Cargo.toml`. It currently has issues however (see [this issue](https://github.com/rust-lang/cargo/issues/6031#issuecomment-422160178)).

## Examples

See the examples directory in this repository.

## Roadmap

A lot of desired features have been defined in issues.

graphql_client does not provide any networking, caching or other client functionality yet. Integration with different HTTP libraries is planned, although building one yourself is trivial (just send the constructed request payload as JSON with a POST request to a GraphQL endpoint, modulo authentication).

There is an embryonic CLI for downloading schemas - the plan is to make it something similar to `apollo-codegen`.

## Contributors

Warmest thanks to all those who contributed in any way (not only code) to this project:

- Alex Vlasov (@indifferentalex)
- Ben Boeckel (@mathstuf)
- Christian Legnitto (@LegNeato)
- Dirkjan Ochtman (@djc)
- Fausto Nunez Alberro (@brainlessdeveloper)
- Hirokazu Hata (@h-michael)
- Peter Gundel (@peterfication)
- Sonny Scroggin (@scrogson)
- Sooraj Chandran (@SoorajChandran)
- Tom Houlé (@tomhoule)

## Code of conduct

Anyone who interacts with this project in any space, including but not limited to
this GitHub repository, must follow our [code of conduct](https://github.com/graphql-rust/graphql-client/blob/master/CODE_OF_CONDUCT.md).

## License

Licensed under either of these:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

### Contributing

Unless you explicitly state otherwise, any contribution you intentionally submit
for inclusion in the work, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.
