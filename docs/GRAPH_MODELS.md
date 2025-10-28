# SurrealDB Graph Queries: Implementation Guide for LLM Agents

This document synthesizes SurrealDB's graph modeling and querying from official references. It provides exact SurrealQL syntax, best practices, and decision criteria for implementing graph databases. Focus on nodes (records), edges (relation tables with `in`/`out`), and traversals via arrow operators (`->`, `<-`, `<->`). Use graphs for bidirectional links, edge metadata, or complex paths; prefer record links for simple unidirectional ownership.

Assumes schemaless mode; add `DEFINE TABLE/FIELD` for schema enforcement, type safety, and Surrealist visualization.

## Core Concepts

- **Nodes**: Entities in tables (e.g., `user:alice`). Properties: Fields like `name: string`.
- **Edges**: Relation tables (e.g., `wrote`) linking nodes bidirectionally. Auto-generates `in` (source ID) and `out` (target ID) fields. Properties: Metadata (e.g., `timestamp`, `strength`).
- **Semantic Triples**: `in → edge → out` (or `node → relation → node`).
- **Traversal Syntax**:
  - `->edge->table`: Outbound (from current to out).
  - `<-edge<-table`: Inbound (from current to in).
  - `<->edge<->table`: Bidirectional (symmetric, e.g., "friends").
- **Links vs. Graphs**:
  - **Record Links**: Unidirectional pointers (e.g., `user.comments: array<record<comment>>`). Efficient for ownership; bidirectional via schema `REFERENCE` since v3.0. Use for simple cases without edge metadata.
  - **Graph Edges**: For bidirectional traversals, weighting, or context (e.g., `wrote` with `mood`). Requires `RELATE`; enables arrow queries.
- **When Graphs Excel**:
  - Metadata on relations (e.g., friendship strength).
  - Complex queries (e.g., paths, recursion).
  - Visualization in Surrealist.
  - Symmetric relations (e.g., partnerships; use unique indexes to deduplicate).

## Schema Definition (Optional: For Safety & Visualization)

Define before creation to restrict types and enable Surrealist graphs.

```
/Users/davidmaple/kodegen/docs/surrealdb/schema.surrealql#L1-10
-- Nodes
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string;

-- Edge (relation)
DEFINE TABLE wrote TYPE RELATION IN (user) OUT (comment) PERMISSIONS FULL;
DEFINE FIELD strength ON wrote TYPE float DEFAULT 0.5;
DEFINE FIELD created_at ON wrote TYPE datetime;

-- Symmetric edge (deduplicate via sorted key)
DEFINE TABLE friends_with TYPE RELATION IN (person) OUT (person);
DEFINE FIELD key ON friends_with VALUE array::sort([in, out]) TYPE string;
DEFINE INDEX unique_friend ON friends_with FIELDS key UNIQUE;
```

- `TYPE RELATION`: Edge-only; can't create standalone.
- `IN/OUT`: Enforces endpoints (e.g., only `user` to `comment`).
- For delete behavior: `DEFINE FIELD ... ON DELETE CASCADE/RESTRICT/SET NULL/IGNORE;`.
- Symmetric: Use `array::sort([in, out])` key + `UNIQUE` index to prevent duplicates.

## Creating Nodes and Edges

Use transactions for atomicity.

1. **Nodes** (`CREATE` for single, `INSERT` for batch):
   ```
   /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L1-6
   CREATE user:alice SET
     name = 'Alice',
     email = 'alice@example.com',
     created_at = time::now();

   INSERT INTO comment [
     {id: 'one', text: 'Great post!', created_at: time::now()},
     {id: 'two', text: 'Thanks!', created_at: time::now()}
   ];
   ```

2. **Edges** (`RELATE`: Creates edge table record with `in`/`out`; bidirectional by default):
   - Basic:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L8-11
     RELATE user:alice -> wrote -> comment:one CONTENT {
       strength: 0.8,
       created_at: time::now()
     };
     ```
   - With auto-node creation (use ULID/RAND for edge IDs):
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L13-16
     RELATE person:john -> wishlist:ulid() -> product:phone
       SET created_at = time::now();
     ```
   - Batch:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L18-19
     RELATE [user:alice, user:bob] -> follows -> user:charlie;
     ```
   - Pull data:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L21-26
     RELATE user:alice -> orders -> product:phone CONTENT {
       quantity: 2,
       product_name: ->product.name,
       price: ->product.price,
       user_address: <-user.address
     };
     ```
   - Symmetric (fails duplicate due to index):
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L28-30
     RELATE person:one -> friends_with -> person:two
       SET friends_since = time::now();
     ```

3. **RELATE Existing/Auto-Create Nodes**:
   ```
   /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L32-34
   RELATE user:alice -> knows -> person:unknown CONTENT {met_at: time::now()};
   -- Auto-creates person:unknown if missing.
   ```

4. **Upsert**:
   ```
   /Users/davidmaple/kodegen/docs/surrealdb/create.surrealql#L36-38
   RELATE user:alice -> follows -> user:bob IF NONE
     CONTENT {since: time::now()};
   ```

## Querying Graphs

`SELECT` with arrows; auto-flattens multi-hops. Start from table or ID.

1. **Basic Traversal**:
   - Outbound:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L1-3
     SELECT ->wrote->comment.* AS posts FROM user:alice;
     -- Returns: [{id: 'one', text: '...'}]
     ```
   - Inbound:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L5-7
     SELECT <-wrote-.* AS authors FROM comment:one;
     -- Returns: [{id: user:alice, name: 'Alice'}]
     ```
   - Bidirectional:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L9-12
     SELECT <->friends_with<->person AS friends FROM person:one;
     -- Filter self: array::complement(..., [id])
     ```

2. **Multi-Hop** (flattens automatically):
   - Paths:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L14-17
     SELECT ->works_for->person->works_for->person AS chain FROM person:employee1;
     -- employee → manager → president.
     ```
   - Filtered:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L19-21
     SELECT ->knows.{greeted > 5, out AS friend} FROM npc:1;
     -- Edge props filter.
     ```
   - Weighted count:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L23-25
     SELECT count() AS strength, out FROM greeted
       WHERE in = npc:1 GROUP BY out;
     ```

3. **Direct from ID**:
   ```
   /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L27-29
   user:alice.{name, posts: ->wrote->comment};
   -- Destructure + traverse.
   ```

4. **Advanced**:
   - **Recursive** (all descendants; limit depth):
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L31-35
     SELECT * FROM person:alice ->family->person
       FETCH ->family->person ON person;  -- Recurses 1 level; repeat or use CTE for deeper.
     -- CTE alternative:
     WITH RECURSIVE path AS (
       SELECT * FROM person WHERE id = person:alice
       UNION
       SELECT p.* FROM path, person p WHERE path.id ->family-> p.id
     ) SELECT * FROM path;
     ```
   - **Mutual**:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L37-40
     SELECT array::intersection(
       person:alice ->friends_with->person,
       person:bob ->friends_with->person
     ) AS mutuals FROM person;
     ```
   - **Conditional**:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L42-45
     SELECT *, ->orders->product FROM user:alice
       WHERE ->orders.created_at > time::now() - 1d;
     ```
   - **Optimize**:
     ```
     /Users/davidmaple/kodegen/docs/surrealdb/query.surrealql#L47-48
     SELECT ->wrote->comment FROM user:alice EXPLAIN;
     ```

## Updating and Deleting

1. **Update Edges/Nodes**:
   ```
   /Users/davidmaple/kodegen/docs/surrealdb/update.surrealql#L1-4
   UPDATE wrote SET strength += 0.1
     WHERE in = user:alice AND out = comment:one;

   UPDATE user:alice SET name = 'Alicia';
   ```

2. **Delete** (schema controls cascade):
   ```
   /Users/davidmaple/kodegen/docs/surrealdb/update.surrealql#L6-8
   DELETE wrote WHERE in = user:alice;
   -- Or: REMOVE wrote;  -- Drops table.
   DELETE user:alice;  -- Edges auto-handled by schema.
   ```

## Best Practices

- **Decide Early**: Record links for perf/simple; graphs for flexibility/metadata.
- **Bidirectional Default**: Arrows traverse both ways.
- **Performance**: Index edges (e.g., `DEFINE INDEX idx_time ON wrote COLUMNS created_at;`). Paginate: `START 10 LIMIT 20`. Use `EXPLAIN`.
- **Flattening**: Arrows maintain flat structure in paths (no manual `flatten()`).
- **Surrealist**: Define `TYPE RELATION IN/OUT` for visual graphs.
- **Transactions**: Wrap multi-ops: `BEGIN TRANSACTION; ... COMMIT;`.
- **Errors**: Catch index violations (duplicates). Use `IF NONE` for upserts.
- **Neo4j Equivalents**:
  - `CREATE (a:User)-[:WROTE]->(b:Post)` → `RELATE user:a -> wrote -> post:b`
  - `MATCH (a)-[:FRIENDS]-(b) RETURN a,b` → `SELECT <->friends<->person FROM person:a`
- **Limits**: For huge graphs, use `GROUP BY`, conditions, or subqueries. Test recursion depth.

For full details: [RELATE Docs](https://surrealdb.com/docs/surrealql/statements/relate). Implement by generating/executing SurrealQL iteratively.