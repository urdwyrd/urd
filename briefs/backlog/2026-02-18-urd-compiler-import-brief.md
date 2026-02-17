# URD — Compiler Phase 2: IMPORT

*Import resolution, dependency graph construction, and file ordering*

February 2026 | Engineering Phase

`FileAST (entry) → IMPORT → DependencyGraph + ordered FileAST[]`

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:**
**Status:**

### What was done

-

### What changed from the brief

-

---

> **Document status: BRIEF** — Defines the IMPORT phase of the Urd compiler. IMPORT is the second phase of the five-phase pipeline. It takes the entry file's `FileAST`, discovers all imported files by recursively following `ImportDecl` nodes, parses each discovered file via PARSE, builds a dependency graph, and produces a topologically sorted list of `FileAST`s for LINK.

> **Dependencies:** This brief builds on the Compiler Architecture Brief (dependency graph structure, path normalisation rules, topological ordering, diagnostic codes, input limits) and the PARSE Phase Brief (FileAST structure, ImportDecl node). Both are required reading.


## Purpose

IMPORT transforms a single parsed entry file into a complete compilation unit: the entry file plus every file reachable through its transitive import graph, all parsed and ordered for downstream processing.

IMPORT has exactly three jobs:

1. **Discover files.** Follow `ImportDecl` nodes from the entry file's frontmatter, recursively, loading and parsing each imported file.
2. **Build the dependency graph.** Record which file imports which, detect cycles, enforce depth limits.
3. **Order files.** Produce a topologically sorted list of `FileAST`s where dependencies appear before dependents.

### What IMPORT Does

- Extracts `ImportDecl` nodes from the entry file's frontmatter.
- For each import declaration, resolves the path relative to the importing file's directory.
- Normalises paths per the architecture brief's File Path Normalisation rules (forward slashes, relative to entry directory, no `..` segments in stored paths, case-sensitive comparison).
- Checks whether the file has already been loaded (by normalised path). If so, adds the dependency edge without re-parsing.
- If the file has not been loaded: reads it from disk, checks file size (URD103), calls PARSE to produce a `FileAST`, and adds it to the graph.
- Detects casing mismatches between the import path as written and the path as discovered on disk (URD206 warning).
- Detects circular imports (URD202 error) during graph traversal.
- Enforces the 64-level import depth limit (URD204 error).
- After all files are discovered: checks the file count limit (URD205, 256 files max).
- Enforces file stem uniqueness across the compilation unit (URD203 error).
- Produces a topologically sorted list of `FileAST`s with deterministic tiebreaking (alphabetical by normalised path).
- Passes all content nodes through unchanged — IMPORT does not inspect or modify narrative content.

### What IMPORT Does Not Do

- Resolve references. It sees `@guard` in a content node and ignores it. That is LINK's job.
- Validate types or semantics. It does not inspect type definitions, entity declarations, or any frontmatter beyond `import:` entries.
- Modify ASTs. It passes `FileAST`s through to LINK exactly as PARSE produced them. IMPORT adds no annotations.
- Merge scopes. It records which file imports which, but does not compute visible scopes. Visible scope computation is LINK's responsibility using the dependency edges IMPORT provides.


## Interface Contract

### Input

```
import_resolve(entry_ast: FileAST, diagnostics: DiagnosticCollector) → CompilationUnit
```

- `entry_ast`: The `FileAST` produced by PARSE for the entry file. IMPORT reads its `ImportDecl` nodes and its `file_path`.
- `diagnostics`: The shared diagnostic collector. IMPORT appends to it.

IMPORT is the only phase that touches the filesystem (to read imported files). All other phases operate on in-memory data structures.

### Output

```
CompilationUnit {
  graph: DependencyGraph,
  ordered_asts: FileAST[],   // topologically sorted, entry file last
}
```

IMPORT always returns a `CompilationUnit`. It never returns `null`. Even if every import fails, the entry file is still a valid single-file compilation unit. Catastrophic entry file parse failure is handled by the orchestrator before IMPORT is called — IMPORT's `entry_ast` input is always a valid `FileAST`.

`ordered_asts` contains every successfully parsed `FileAST` in the compilation unit, including the entry file. The list is topologically sorted: if file A imports file B, B appears before A in the list. The entry file is always last. Ties (files at the same topological depth with no dependency between them) are broken alphabetically by normalised path.

**Failed imports do not add nodes or edges.** Imports that fail path validation, filesystem load, file size check, or PARSE catastrophic failure simply do not appear in the graph. No broken edges, no stub nodes, no phantom entries. The diagnostic (URD201, URD103, etc.) is the sole record of the failure. LINK discovers unresolved references downstream and reports them — it does not need to know why an import failed.

### Guarantees

After IMPORT completes, the following properties hold:

1. **The dependency graph is acyclic.** Any import that would create a cycle is skipped before an edge is added (URD202). The graph that reaches LINK is guaranteed acyclic.
2. **All file paths are normalised.** Every path in the graph follows the architecture brief's normalisation rules. No backslashes, no `..` segments, all relative to entry directory. Comparisons are case-sensitive on normalised paths (canonical casing is applied during normalisation on case-insensitive systems).
3. **The topological order is deterministic.** Same input files always produce the same ordering. Ties are broken alphabetically by normalised path.
4. **File stems are unique (if no URD203).** URD203 is a fatal error. If any stem collision is detected, the orchestrator must stop compilation before LINK begins. If IMPORT completes without URD203, LINK can unconditionally rely on file stems being unique for section ID construction.
5. **The file count is within limits (if no URD205).** URD205 is a fatal error. If the graph exceeds 256 files, the orchestrator must stop compilation before LINK begins. If IMPORT completes without URD205, the file count is within limits.
6. **Import depth is within limits.** No import chain exceeds 64 FileASTs (including the entry file). See Depth Limit Definition below.
7. **Every FileAST is unmodified.** IMPORT does not add annotations, modify nodes, or reorder content. The `FileAST`s in `ordered_asts` are the same instances PARSE produced, with no mutations performed by IMPORT.
8. **The entry file is always present and always last.** Even if all imports fail, the entry file appears in `ordered_asts`.
9. **No broken edges or stub nodes.** Every node in the graph has a valid `FileAST`. Every edge connects two nodes that are both present in the graph. Failed imports are absent entirely.
10. **Duplicate imports produce one edge.** Multiple `ImportDecl` entries from the same source file to the same resolved target produce a single edge. If diagnostics later reference this edge, the span of the first `ImportDecl` is used.


## The Algorithm

IMPORT performs a depth-first traversal of the import graph, starting from the entry file. The algorithm is conceptually simple but has several invariants to maintain.

### Initialisation

```
graph = new DependencyGraph()
entry_node = FileNode(path: entry_ast.file_path, ast: entry_ast, imports: [])
graph.add_node(entry_node)
traversal_stack = [entry_ast.file_path]   // for cycle detection
visited = { entry_ast.file_path }          // for deduplication
```

### Recursive Discovery

For each file being processed (starting with the entry file):

1. **Extract import declarations.** Scan `file_ast.frontmatter.entries` for `ImportDecl` nodes. If there is no frontmatter, or no import declarations, the file is a leaf node — skip to the next file.

2. **For each `ImportDecl`:**

   a. **Trim and validate the path.** Trim leading and trailing whitespace from `ImportDecl.path`. Then run path validation checks (see Path Validation below). If any check fails, emit the diagnostic and skip this import. No node or edge is added.

   b. **Resolve the path.** The trimmed path is relative to the importing file's directory. Resolve it to a normalised path relative to the entry file's directory. Apply all normalisation rules (forward slashes, collapse `..` segments, no symlink resolution). URD208 (outside project root) is checked during this step.

   **Path variable definitions.** Three path values are in play during import processing:
   - `written_path`: the author's import value after whitespace trimming and backslash normalisation. Used for validation checks and diagnostic messages.
   - `normalised_path`: the canonical path stored in the graph, relative to the entry directory. Initially derived from `written_path` via resolution; may be updated by URD206 casing correction. Used for visited checks, cycle detection, depth tracking, and graph storage.
   - `fs_path`: the absolute filesystem path, computed from `normalised_path` and the entry file's directory. Used only for reading the file from disk.

   **Diagnostic path convention.** Import diagnostics fall into two categories:
   - **Declaration diagnostics** (URD201, URD207, URD208, URD209, URD210, URD211) use `written_path` — the form the author wrote. This helps authors find and fix the declaration. URD201 is a filesystem lookup failure, but it reports `written_path` because it is primarily a declaration fix — the author needs to correct what they wrote in the `import:` line.
   - **File-identity diagnostics** (URD203, URD205, URD212, URD213, URD214) use `normalised_path` — the resolved canonical identity. These concern files the compiler has already located or loaded, where the author's written form may differ from the actual file opened. For URD213 and URD214, report the best available `normalised_path` at the time of failure (before or after casing correction), since I/O errors may occur during directory enumeration before canonical casing is known.

   Note: URD201 is a filesystem failure but is reported using `written_path` because the author needs to correct the `import:` declaration, not a resolved path.

   c. **Check for self-import.** Let `importer_path = file_ast.file_path` (already normalised as graph identity). If `normalised_path` equals `importer_path`, emit URD207: *"File imports itself: '{written_path}'."* Skip this import. No edge added.

   d. **Check for cycles.** If `normalised_path` is in `traversal_stack`, a cycle has been detected. Emit URD202: *"Circular import detected: {cycle_path}."* The cycle path is the full chain from the current file back to the repeated file, formatted using `normalised_path` values: `a.urd.md → b.urd.md → a.urd.md`. Skip this import. No edge added.

   e. **Check import depth.** If `traversal_stack.length >= 64` (the stack already contains the current file chain; recursing would exceed the 64-file limit), emit URD204: *"Import depth limit exceeded (64 files in chain)."* Skip this import. No edge added.

   f. **Check for already-loaded file.** If `normalised_path` is in `visited`, the file is already in the graph. Add a dependency edge from the importing file to the existing node (if one does not already exist — multiple `ImportDecl` entries from the same source file to the same resolved target produce one edge). Do not re-parse or re-traverse. Continue with the next declaration.

**Duplicate import processing and diagnostics.** Each `ImportDecl` is processed independently in source order. If the second of two imports targeting the same file fails pre-validation (e.g., `import: ./types.urd.md` succeeds, then `import:  ` with whitespace-only value triggers URD211), the diagnostic attaches to the specific `ImportDecl` that failed. Edge metadata (used only for cycle reporting when the same edge appears via multiple declarations) uses the span of the first `ImportDecl` that created the edge.

   g. **Load the new file.** Compute `fs_path` by joining the entry file's directory with `normalised_path`. Read the file from disk at `fs_path`.
      - If the file does not exist, emit URD201: *"Imported file not found: '{written_path}' (imported from {source_file}:{line})."* Skip this import. No node or edge added.
      - If the file is found on disk, perform the casing mismatch check (see Casing Mismatch Detection) and possibly update `normalised_path`. If a mismatch is found, emit URD206 (warning) and update `normalised_path` to the discovered casing. Recompute `fs_path` from the updated `normalised_path` if needed, and ensure any subsequent read uses the updated path. **After updating `normalised_path`, re-check the visited set with the new value.** If the canonical path is already visited (the file was previously loaded under its canonical name via a different import), add a dependency edge to the existing node and continue with the next declaration — do not re-parse or add a duplicate node.
      - Validate UTF-8 and check file size (URD103, 1 MB limit). These checks may occur in any order or during the read itself, but the externally visible behaviour is: if the file exceeds 1 MB, emit URD103 and skip this import (no node or edge added); if the file contains invalid UTF-8 sequences, emit URD212: *"File contains invalid UTF-8: '{normalised_path}'."* and skip this import (no node or edge added). If both conditions apply, emit whichever is detected first.

   h. **Parse the file.** Call `parse(source, normalised_path, diagnostics)` to produce a `FileAST`. The `normalised_path` is the canonical path after any URD206 casing correction — this is the path stored on the `FileAST` and in the graph. If PARSE returns `null` (catastrophic parse failure), skip this import. No node or edge added. The diagnostic from PARSE (URD101 or URD103) is the sole record of the failure.

   i. **Add to graph.** Create a `FileNode` for the parsed file. Add it to the graph. Add a dependency edge from the importing file to this new node. Mark `normalised_path` as visited.

   j. **Recurse.** Push `normalised_path` onto `traversal_stack`. Process the new file's imports (step 1). Pop `normalised_path` from `traversal_stack` when done.

**Edges are added only for imports that succeed.** For already-loaded targets, the edge is added in step f. For newly loaded targets, the edge is added in step i. All failure paths (validation, cycle, depth, filesystem, parse) skip the import without adding nodes or edges.

### Depth Limit Definition

The 64-level limit counts FileASTs in the import chain, including the entry file. If the entry file is at depth 1, a file it directly imports is at depth 2, and so on. The check is: when about to recurse into a new file, if `traversal_stack.length >= 64`, reject. The stack already contains the current chain; adding one more would make 65. This means the deepest possible file in the chain is at depth 64.

Example: entry (depth 1) → A (2) → B (3) → ... → file at depth 64 is allowed. File at depth 65 triggers URD204.

### Post-Discovery Checks

After all files have been discovered and parsed:

1. **File count check.** If `graph.nodes.size` exceeds 256, emit URD205: *"Compilation unit exceeds 256 files ({count} discovered)."* **URD205 is fatal.** After emitting the diagnostic, the orchestrator must stop compilation before LINK begins.

2. **File stem uniqueness check.** For every pair of files in the graph, compute the file stem (filename without directory path or `.urd.md` extension). If any two files share the same stem, emit URD203: *"File stem collision: '{stem}' is produced by both {path_a} and {path_b}. Rename one file to avoid section ID conflicts."* Collect and report all collisions, not just the first. **URD203 is fatal.** After emitting all collision diagnostics, the orchestrator must stop compilation before LINK begins.

3. **Topological sort.** Produce the sorted file list. The algorithm is a standard topological sort (Kahn's algorithm or DFS-based post-order). Ties are broken by alphabetical order of normalised path. The entry file always appears last.

### Topological Sort Detail

The topological sort must be deterministic. Two implementations given the same graph must produce the same ordering. To achieve this:

- Use Kahn's algorithm with a priority queue (sorted alphabetically by normalised path) instead of an arbitrary queue.
- Or use DFS-based post-order and sort the adjacency lists of each node alphabetically before traversal.

Either approach produces the same deterministic order. The key property: when multiple files are ready to be processed (in-degree zero, or all children visited), the one with the alphabetically smallest normalised path goes first.

The entry file has no incoming edges — any discovered import that targets the entry file is rejected as a cycle (URD202), because the entry file is always on the traversal stack at depth 1. It is the root of the dependency tree, so it always finishes last in post-order and always appears last in the sorted list. **Regardless of algorithm choice (Kahn's or DFS), the entry file must be the final element in `ordered_asts`.** If using Kahn's algorithm, exclude the entry file from the initial ready set and append it after all other nodes have been sorted. If using DFS post-order, the entry file naturally appears last as the root. Either way, the entry file is never interleaved with other nodes during sorting.


## Path Resolution

Import paths in Schema Markdown are relative to the importing file. IMPORT must resolve them to normalised paths relative to the entry file's directory.

### Resolution Algorithm

Given an `ImportDecl` with `path: "./shared/types.urd.md"` in file `content/tavern.urd.md`:

0. **Trim whitespace.** Strip leading and trailing whitespace from the path value. Frontmatter parsing may preserve surrounding spaces; IMPORT normalises them away. If the result is empty after trimming, emit URD211.
1. **Convert backslashes to forward slashes.** (Should not appear in source, but handle defensively.)
2. **Strip the leading `./` if present.** `./shared/types.urd.md` → `shared/types.urd.md`.
3. **Resolve relative to the importing file's directory.** The importing file is `content/tavern.urd.md`, so its directory is `content/`. Joining: `content/shared/types.urd.md`.
4. **Collapse `..` segments.** If the path contains `..`, resolve it lexically. `content/../shared/types.urd.md` → `shared/types.urd.md`. If collapsing would go above the entry file's directory (the normalised path would start with `../`), emit URD208. **URD208 is a purely lexical check** — it operates on path strings, not filesystem `realpath`. Symlinks inside the project root that point outside are not detected and are not IMPORT's responsibility. **IMPORT must not call `realpath` or equivalent symlink-resolving functions when normalising paths** — doing so would introduce platform-dependent behaviour.
5. **Store the result.** The normalised path is now relative to the entry file's directory and uses forward slashes only.

### Path Validation

IMPORT validates import paths after whitespace trimming and backslash normalisation, but before filesystem access. The checks run on `written_path` (not the fully resolved `normalised_path`):

| Condition | Diagnostic | Behaviour |
|-----------|-----------|-----------|
| Empty path (after trimming) | URD211: *"Empty import path at {source_file}:{line}."* | Skip this import. No node or edge. |
| Absolute path (starts with `/` or drive letter) | URD209: *"Absolute import paths are not supported: '{written_path}'."* | Skip this import. No node or edge. |
| Path does not end with `.urd.md` | URD210: *"Import path '{written_path}' does not have the .urd.md extension."* | Skip this import. No node or edge. |
| Path resolves outside project root (after `..` collapse) | URD208: *"Import path '{written_path}' resolves outside the project root."* | Checked during path resolution (step 4), not during pre-validation. Skip this import. No node or edge. |

All path validation errors are non-fatal for the compilation unit. The import is skipped, but compilation continues. Failed imports do not add nodes or edges.


## Filesystem Interaction

IMPORT is the only compiler phase that reads from the filesystem. All other phases operate on in-memory `FileAST`s.

### File Reading

For each discovered file, IMPORT:

1. Resolve `fs_path` from `normalised_path` and the entry file's directory.
2. Read the file from disk at `fs_path`.
3. Enforce the 1 MB size limit (URD103) and validate UTF-8 (URD212), in any order or during the read. If the file exceeds 1 MB, emit URD103 and skip this import. If the file contains invalid UTF-8 sequences, emit URD212 and skip this import. If both conditions apply, emit whichever is detected first.
4. If checks pass and the resolved canonical path is not already present in the compilation unit, call `parse(source, normalised_path, diagnostics)` to produce a `FileAST`.

### Casing Mismatch Detection

On case-insensitive filesystems (macOS, Windows), the file may exist at a different casing than the import path specifies. Detection is best-effort:

1. After locating the file on disk, compare the filename component of the discovered directory entry against the filename component of `normalised_path` (before casing correction).
2. If they differ, emit URD206 (warning). Update `normalised_path` to the discovered casing.

The exact mechanism for discovering the canonical filename casing is platform-dependent (e.g., directory enumeration on macOS, `FindFirstFile` on Windows). Use OS APIs to discover canonical casing only — the result must not be used for symlink resolution or path normalisation beyond the filename component. Do not use `realpath` for this purpose, as that would violate the no-symlink-resolution rule. If the platform provides no way to discover canonical casing, skip the check — URD206 is a portability warning, not a correctness constraint.

On case-sensitive filesystems (Linux), a casing mismatch means the file does not exist — URD201 is emitted instead. URD206 only fires on case-insensitive systems where the file was found despite the casing difference.

**Normalisation and casing.** After casing mismatch detection, the canonical (discovered) casing becomes the normalised path stored in the graph. All subsequent comparisons (deduplication, cycle detection, stem computation) use this normalised path, which is always compared case-sensitively. This means: on case-insensitive systems, IMPORT discovers the canonical casing once and uses it consistently; on case-sensitive systems, the written casing is already canonical.

### Filesystem Error Handling

| Error | Diagnostic | Behaviour |
|-------|-----------|-----------|
| File not found | URD201: *"Imported file not found: '{written_path}' (imported from {source_file}:{line})."* | Skip import. No node or edge added. |
| Permission denied | URD213: *"Cannot read file '{normalised_path}': permission denied."* | Skip import. No node or edge added. |
| I/O error (other) | URD214: *"I/O error reading '{normalised_path}': {system_error}."* | Skip import. No node or edge added. |
| Invalid UTF-8 bytes | URD212: *"File contains invalid UTF-8: '{normalised_path}'."* | Skip import. No node or edge added. |

All filesystem errors are non-fatal for the compilation unit. IMPORT continues discovering other files. Failed imports leave no trace in the graph — the diagnostic is the sole record.


## Cycle Detection

Cycles in the import graph are forbidden. The Schema Markdown specification states this as a normative rule.

### Detection Method

IMPORT uses the `traversal_stack` (the current chain of files being processed) for cycle detection. When about to recurse into a file, IMPORT checks whether that file's path is already on the stack. If it is, a cycle exists.

This is O(depth) per check, which is acceptable given the 64-level depth limit. No separate visited-set cycle detection (Tarjan's, etc.) is needed — the stack check is sufficient for this use case because IMPORT processes the graph depth-first and the deduplication set prevents revisiting completed subtrees.

### Cycle Reporting

When a cycle is detected, the diagnostic includes the full cycle path:

```
URD202: Circular import detected: tavern.urd.md → harbor.urd.md → tavern.urd.md
```

The cycle path is constructed from `traversal_stack` (which contains `normalised_path` values): starting at the repeated file's position in the stack, through to the current file, and back to the repeated file. The cycle path is always reported using `normalised_path` values for consistency with graph identity.

### Cycle Recovery

The import that would close the cycle is skipped. No edge is added to the graph. This is consistent with the edges-only-on-success rule — the cycle check occurs before any edge or node creation for that import.

The importing file proceeds as if that `ImportDecl` did not exist:

- The importing file's `visible_scope` (computed by LINK) will not include the cyclically imported file via this edge.
- If the cyclically targeted file is reachable through a non-cyclic path, it remains in the graph and its declarations are available to files that validly import it.
- References to declarations that are only available through the skipped cyclic import will be unresolved — LINK will report them.

IMPORT does not remove the file from the graph if it was already loaded through a valid (non-cyclic) path. Only the specific import declaration that creates the cycle is skipped.


## File Stem Uniqueness

File stems are used to construct section IDs (`file_stem/section_name`). If two files share the same stem, section IDs would collide.

### Check Timing

The stem uniqueness check runs after all files are discovered (post-discovery), before LINK begins. This ensures the check covers the complete compilation unit.

### Check Algorithm

1. Build a map from stem to list of file paths: `Map<string, FilePath[]>`.
2. For each entry with more than one path, emit URD203 listing all conflicting files.
3. Report all collisions, not just the first.

### Stem Derivation

The file stem is the filename component with the `.urd.md` extension removed:

- `tavern.urd.md` → `tavern`
- `content/tavern.urd.md` → `tavern`
- `shared/types.urd.md` → `types`
- `my-world.urd.md` → `my-world`

The directory path is stripped. Only the filename matters. This means `content/tavern.urd.md` and `scenes/tavern.urd.md` both produce stem `tavern` and would collide.


## Diagnostic Catalog

All diagnostics emitted by IMPORT are in the URD200–URD299 range.

### Errors Emitted by IMPORT

| Code | Message Template | Trigger | Recovery |
|------|-----------------|---------|----------|
| URD201 | *"Imported file not found: '{written_path}' (imported from {source_file}:{line})."* | File does not exist on disk. | Skip import. Import absent from graph. |
| URD202 | *"Circular import detected: {cycle_path}."* | Import would create a cycle in the dependency graph. | Skip import. No node or edge added. |
| URD203 | *"File stem collision: '{stem}' is produced by both {path_a} and {path_b}."* | Two files share the same stem. | Report all collisions. **Fatal.** Orchestrator stops before LINK. |
| URD204 | *"Import depth limit exceeded (64 files in chain)."* | Import chain exceeds 64 files without cycling. | Skip import. No node or edge added. |
| URD205 | *"Compilation unit exceeds 256 files ({count} discovered)."* | More than 256 files in the graph. | **Fatal.** Orchestrator stops before LINK. |
| URD207 | *"File imports itself: '{written_path}'."* | `import:` points to the file it appears in. | Skip import. |
| URD208 | *"Import path '{written_path}' resolves outside the project root."* | `..` segments push the path above the entry file's directory. | Skip import. |
| URD209 | *"Absolute import paths are not supported: '{written_path}'."* | Path starts with `/` or a drive letter. | Skip import. |
| URD210 | *"Import path '{written_path}' does not have the .urd.md extension."* | Missing or wrong extension. | Skip import. |
| URD211 | *"Empty import path at {source_file}:{line}."* | `import:` with no value. | Skip import. |
| URD212 | *"File contains invalid UTF-8: '{normalised_path}'."* | Invalid UTF-8 bytes in file. Reports best available `normalised_path` at time of failure. | Skip import. |
| URD213 | *"Cannot read file '{normalised_path}': permission denied."* | OS permission error. | Skip import. |
| URD214 | *"I/O error reading '{normalised_path}': {system_error}."* | Other filesystem error. | Skip import. |

### Warnings

| Code | Message Template | Trigger |
|------|-----------------|---------|
| URD206 | *"Import path '{written_path}' differs in filename casing from discovered file '{discovered_path}'. Using discovered casing."* | File found on a case-insensitive filesystem with different casing than `normalised_path` before correction. |

### Informational

IMPORT does not emit info-level diagnostics.


## Acceptance Criteria

### Unit Tests: Path Resolution

| Test | Import Path | Importing File | Expected Normalised Path |
|------|------------|---------------|------------------------|
| Simple relative | `./types.urd.md` | `world.urd.md` | `types.urd.md` |
| Subdirectory | `./shared/types.urd.md` | `world.urd.md` | `shared/types.urd.md` |
| Parent directory | `../shared/types.urd.md` | `content/tavern.urd.md` | `shared/types.urd.md` |
| Nested parent | `../../lib/core.urd.md` | `content/scenes/tavern.urd.md` | `lib/core.urd.md` |
| No leading dot-slash | `types.urd.md` | `world.urd.md` | `types.urd.md` |
| Backslash conversion | `.\shared\types.urd.md` | `world.urd.md` | `shared/types.urd.md` |
| Outside project root | `../../outside.urd.md` | `tavern.urd.md` | URD208 error |
| Absolute path | `/usr/shared/types.urd.md` | `world.urd.md` | URD209 error |
| Missing extension | `./types.md` | `world.urd.md` | URD210 error |
| Empty path | (empty string) | `world.urd.md` | URD211 error |
| Self-import | `./world.urd.md` | `world.urd.md` | URD207 error |
| Whitespace in path | `  ./types.urd.md  ` | `world.urd.md` | `types.urd.md` (leading/trailing whitespace trimmed) |

### Unit Tests: Graph Construction

| Test | Setup | Expected |
|------|-------|----------|
| Single file, no imports | Entry file has no `import:` declarations. | Graph: 1 node, 0 edges. `ordered_asts`: `[entry]`. |
| Linear chain A→B→C | A imports B, B imports C, C has no imports. | Graph: 3 nodes, 2 edges. `ordered_asts`: `[C, B, A]`. |
| Diamond A→B, A→C, B→D, C→D | Shared dependency D. | Graph: 4 nodes, 4 edges. D loaded once. `ordered_asts`: `[D, B, C, A]` or `[D, C, B, A]` depending on alphabetical tie. |
| Duplicate import | A imports B twice (two `import: ./b.urd.md` lines). | B loaded once. One edge A→B. No error. If later diagnostics reference this edge, the span of the first `ImportDecl` is used. |
| Cycle A→B→A | Circular import. | URD202. Import B→A skipped, no edge added. Both files in graph (A imported B successfully). `ordered_asts`: `[B, A]`. |
| Longer cycle A→B→C→A | Three-file cycle. | URD202 with full path `a → b → c → a`. Import C→A skipped, no edge added. All three files in graph. |
| Deep chain (65 levels) | Linear chain exceeding 64. | URD204 at level 65. Import skipped, no edge added. Files 1–64 in graph. |
| Missing file | A imports B, B does not exist. | URD201 for B (reported using the author's `written_path`). B absent from graph. `ordered_asts`: `[A]`. |
| File too large | A imports B, B exceeds 1 MB. | URD103 for B. A in graph. `ordered_asts`: `[A]`. |
| Stem collision | `content/tavern.urd.md` and `scenes/tavern.urd.md` both in graph. | URD203 naming both files. |

### Unit Tests: Topological Sort Determinism

| Test | Graph | Expected Order |
|------|-------|---------------|
| Linear | A→B→C | `[C, B, A]` |
| Diamond, alphabetical tie | A→B, A→C (B and C independent) | `[B, C, A]` (B before C alphabetically) |
| Wide fan | A imports B, C, D, E (all independent) | `[B, C, D, E, A]` (alphabetical) |
| Entry imports two, both share dep | A (entry) imports B and C. Both B and C import D. | `[D, B, C, A]` (D first as shared dep, B before C alphabetically, A last as entry) |

### Integration Tests

| Test | Setup | Expected |
|------|-------|----------|
| Two Room Key Puzzle | Single file, no imports. | Trivial graph. One FileAST. Zero import diagnostics. |
| Interrogation with import | Entry file imports `world.urd.md`. | Two-file graph. `world.urd.md` first in order. |
| Multi-file project | Entry imports `types.urd.md` and `npcs.urd.md`. `npcs.urd.md` imports `types.urd.md`. | Three-file graph. Diamond resolved correctly. `types.urd.md` first. |

### Error Recovery Tests

| Test | Setup | Expected |
|------|-------|----------|
| One bad import among good | A imports B (exists) and C (missing). | URD201 for C. A and B in graph. `ordered_asts`: `[B, A]`. |
| Cycle with other imports | A imports B and C. B imports A (cycle) and D. | URD202 for B→A. All four files in graph (A, B, C, D). |
| Parse failure in import | A imports B. B has unclosed frontmatter. | URD101 from PARSE for B. B not in graph (no node or edge). `ordered_asts`: `[A]`. |
| Cascading: C missing, B depends on C, A depends on B | Three-level chain with broken leaf. | URD201 for C (reported using `written_path`). B and A both in graph. LINK will report B's unresolved references. |
| Invalid UTF-8 in import | A imports B. B contains invalid UTF-8 bytes. | URD212 for B (reported using `normalised_path`). B not in graph. `ordered_asts`: `[A]`. |

### Span Reference Tests

All IMPORT diagnostics that reference a source location must point to the `ImportDecl` node's span in the importing file. Verify:

- URD201 message includes the correct file path and line number of the `import:` declaration.
- URD202 message includes the correct source location of the import that closes the cycle.
- URD206 warning references the `ImportDecl` span, not the discovered file.


## Relationship to Other Phases

### From PARSE

IMPORT receives `FileAST`s from PARSE. It inspects only `ImportDecl` nodes in the frontmatter. All content nodes pass through untouched.

When IMPORT needs to parse a newly discovered file, it calls PARSE directly. IMPORT is PARSE's only caller in the standard compilation pipeline (the orchestrator calls PARSE for the entry file, then hands the result to IMPORT, which calls PARSE for all subsequent files).

### To LINK

LINK receives the `CompilationUnit` from IMPORT and needs:

1. **The `DependencyGraph`** to compute visible scopes (`visible_scope(F) = {F} ∪ {direct imports of F}`).
2. **The `ordered_asts` list** to process files in topological order during the collection sub-pass.
3. **File stem data** (derivable from `FileNode.path`) for section ID construction.
4. **Confidence that the graph is acyclic and the ordering is deterministic**, because LINK's collection order determines symbol table insertion order, which determines EMIT's output order.

LINK does not need to understand how IMPORT resolved paths or detected cycles. It trusts the guarantees.

*End of Brief*
