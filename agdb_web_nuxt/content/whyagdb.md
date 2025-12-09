---
title: Why agdb?
description: "Why agdb?, Agnesoft Graph Database"
---

import { Callout } from "nextra/components";

# Executive Summary

<Callout >
-   `agdb` is application native database.

- No query language, queries are written in the same language as the application.

- Performance without limits, constant time lookups and traversals regardless of db size.

- Simple to build, use & scale.

</Callout>

## Why agdb?

Databases today are complicated systems that require complex setup, special maintenance and can quickly become very costly. They can only be interacted with using special languages that developers need to learn in addition to their existing knowledge stack. All that adds up to very high costs, slow development cycles and increases complexity of your software solution.

But not anymore.

Introducing `agdb`, the first application native database. The `agdb` is designed so that it is simple to use and can be seamlessly added to any application as the queries are always written in the same language as the application itself. No cognitive overload for developers requiring extra languages like SQL, Cypher, SPARQL etc. The `agdb` is designed to be used by developers or data engineers. It's simple and transparent design contributes to the ease of use, low maintenance costs while being fully [ACID](https://en.wikipedia.org/wiki/ACID) compliant.

## No Query language

Context switching is one of the worst things that can happen to a developer. It takes time to switch between languages, and it is very easy to make mistakes that way. It is akin to having a three-way conversation in two languages. The `agdb` is designed to be used with the same language as the application itself. This means that developers can write queries in the same language as the application. That makes it easier to write queries and reduces the chances of making mistakes that could lead to security vulnerabilities, data leaks and other issues.

Instead of a dedicated query language the `agdb` is using "native object queries". Queries in `agdb` are binary objects describing the requested data and its constraints. That not only reduces cognitive load of switching to another language but completely eliminates for example SQL injection attacks.

Object queries are the most expressive way to query data and receive them back natively in your application language without the need for interpretation.

## Performance Without Limits

Databases traditionally tackles performance and scalability issues with caching, indexing, sharding and other techniques. While `agdb` offers these features as well the growing data sets are quickly outgrowing these techniques. The only viable solution is to fundamentally reduce the work required by the database when handling the data.

The most scalable solution is therefore to limit the required work without sacrificing any of the important properties like data availability and latency. To do that he `agdb` plots data on a graph that allows for constant time lookups and traversals. This means that the `agdb` is just as fast with a data set of 10 records as it is with a data set of 10 billion records. When querying the database you specify what subgraph you are interested in and the `agdb` will return the results in constant time, depending only on the size of the subgraph, and not the entire data set. Furthermore, the data on the graph can be arbitrarily connected removing the need for rigid schemas and completely eliminating database migrations.

## 10X Cost Reduction

All the properties of `agdb` will reduce costs of your database solution while being enjoyable to work with. The `agdb` is designed to be used by developers in their native environment of their software solution, not as an unwieldy accessory to be interacted with in a special way. The `agdb` is also very simple to use and runs in Docker, Kubernetes, any of the public clouds or bare metal. It is written in safe Rust with minimal hardware footprint and high degree of security. It does not need any special care, linking, special hardware etc. For resiliency, it offers cluster mode with custom enhanced consensus protocol for fastest possible guaranteed consistency. It is fully ACID compliant and can be used for any use case where you need a database like big data processing, web, embedded development or large business applications.

---

The `agdb` redefines how databases are used in applications akin to how DevOps revolutionized software delivery. The `agdb` proves that database does not need to be unwieldy complex system that is hard to learn and costly in many ways. Quite the opposite, it can be easy, native to your tech stack and solid foundation of your project. Instead of worrying about your choice of database and its ever-growing costs use `agdb`. And focus on building great software for the future!

## Where to next?

- Intrigued or unsure? Book a [technical consultation](/enterprise/consultation) with our experts.

- Want to learn more? Head to our [documentation](/docs/docs).

- Interested in peeking behind the scenes? Read our [blog](/blog/blog).
