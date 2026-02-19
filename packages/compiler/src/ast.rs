/// AST node types for Urd Schema Markdown.
///
/// The AST is the central data structure. PARSE produces it, every subsequent
/// phase reads it, and LINK annotates it. EMIT traverses it to produce JSON.
///
/// Design principles:
/// - File-scoped: each file produces its own `FileAST`, never merged.
/// - Annotatable: nodes carry optional annotation slots that LINK fills in.
/// - Span-tracked: every node records its exact source position.

use crate::span::Span;

// ── File-level nodes ──

/// Root node for a single parsed `.urd.md` file.
#[derive(Debug, Clone)]
pub struct FileAst {
    pub path: String,
    pub frontmatter: Option<Frontmatter>,
    pub content: Vec<ContentNode>,
    pub span: Span,
}

/// The `---`-delimited frontmatter block.
#[derive(Debug, Clone)]
pub struct Frontmatter {
    pub entries: Vec<FrontmatterEntry>,
    pub span: Span,
}

/// A key-value pair in frontmatter.
#[derive(Debug, Clone)]
pub struct FrontmatterEntry {
    pub key: String,
    pub value: FrontmatterValue,
    pub span: Span,
}

/// Typed frontmatter value.
#[derive(Debug, Clone)]
pub enum FrontmatterValue {
    Scalar(Scalar),
    List(Vec<FrontmatterValue>),
    Map(Vec<FrontmatterEntry>),
    InlineObject(Vec<FrontmatterEntry>),
    EntityDecl(EntityDecl),
    TypeDef(TypeDef),
    ImportDecl(ImportDecl),
    WorldBlock(WorldBlock),
}

/// A frontmatter value (primitives, lists, entity references).
#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    List(Vec<Scalar>),
    EntityRef(String),
}

// ── Frontmatter-specific nodes ──

/// `import: ./path.urd.md`
#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: String,
    pub span: Span,
}

/// The `world:` block in frontmatter.
#[derive(Debug, Clone)]
pub struct WorldBlock {
    pub fields: Vec<(String, Scalar)>,
    pub span: Span,
}

/// A type definition: `TypeName [traits]: properties`.
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub traits: Vec<String>,
    pub properties: Vec<PropertyDef>,
    pub span: Span,
}

/// A property within a type definition.
#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name: String,
    pub property_type: String,
    pub default: Option<Scalar>,
    pub visibility: Option<String>,
    pub values: Option<Vec<String>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub ref_type: Option<String>,
    pub element_type: Option<String>,
    pub element_values: Option<Vec<String>>,
    pub element_ref_type: Option<String>,
    pub description: Option<String>,
    pub span: Span,
}

/// Entity declaration: `@name: Type { overrides }`.
#[derive(Debug, Clone)]
pub struct EntityDecl {
    pub id: String,
    pub type_name: String,
    pub property_overrides: Vec<(String, Scalar)>,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

// ── Content nodes ──

/// A node in the narrative content region of a `.urd.md` file.
#[derive(Debug, Clone)]
pub enum ContentNode {
    LocationHeading(LocationHeading),
    SequenceHeading(SequenceHeading),
    PhaseHeading(PhaseHeading),
    SectionLabel(SectionLabel),
    EntityPresence(EntityPresence),
    EntitySpeech(EntitySpeech),
    StageDirection(StageDirection),
    Prose(Prose),
    Choice(Choice),
    Condition(Condition),
    OrConditionBlock(OrConditionBlock),
    Effect(Effect),
    Jump(Jump),
    ExitDeclaration(ExitDeclaration),
    BlockedMessage(BlockedMessage),
    RuleBlock(RuleBlock),
    Comment(Comment),
    ErrorNode(ErrorNode),
}

/// `# Display Name` — a location heading.
#[derive(Debug, Clone)]
pub struct LocationHeading {
    pub display_name: String,
    pub span: Span,
}

/// `## Display Name` — a sequence heading.
#[derive(Debug, Clone)]
pub struct SequenceHeading {
    pub display_name: String,
    pub span: Span,
}

/// `### Name (auto)` — a phase heading.
#[derive(Debug, Clone)]
pub struct PhaseHeading {
    pub display_name: String,
    pub auto: bool,
    pub span: Span,
}

/// `== name` — a section label.
#[derive(Debug, Clone)]
pub struct SectionLabel {
    pub name: String,
    pub span: Span,
}

/// `[@arina, @barrel]` — entity presence in a location.
#[derive(Debug, Clone)]
pub struct EntityPresence {
    pub entity_refs: Vec<String>,
    pub annotations: Vec<Option<Annotation>>,
    pub span: Span,
}

/// `@arina: What'll it be?` — entity speech.
#[derive(Debug, Clone)]
pub struct EntitySpeech {
    pub entity_ref: String,
    pub text: String,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// `@arina leans in close.` — stage direction.
#[derive(Debug, Clone)]
pub struct StageDirection {
    pub entity_ref: String,
    pub text: String,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// Plain narrative text.
#[derive(Debug, Clone)]
pub struct Prose {
    pub text: String,
    pub span: Span,
}

/// `*` or `+` choice with nested content.
#[derive(Debug, Clone)]
pub struct Choice {
    pub sticky: bool,
    pub label: String,
    pub target: Option<String>,
    pub target_type: Option<String>,
    pub content: Vec<ContentNode>,
    pub indent_level: usize,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// `? expression` — a condition.
#[derive(Debug, Clone)]
pub struct Condition {
    pub expr: ConditionExpr,
    pub indent_level: usize,
    pub span: Span,
}

/// `? any:` block with multiple bare condition expressions.
#[derive(Debug, Clone)]
pub struct OrConditionBlock {
    pub conditions: Vec<ConditionExpr>,
    pub indent_level: usize,
    pub span: Span,
}

/// `> effect` — an effect node.
#[derive(Debug, Clone)]
pub struct Effect {
    pub effect_type: EffectType,
    pub indent_level: usize,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// `-> name` or `-> exit:name` — a jump.
#[derive(Debug, Clone)]
pub struct Jump {
    pub target: String,
    pub is_exit_qualified: bool,
    pub indent_level: usize,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// `-> direction: Destination` — an exit declaration.
#[derive(Debug, Clone)]
pub struct ExitDeclaration {
    pub direction: String,
    pub destination: String,
    pub children: Vec<ContentNode>,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// `! message` — a blocked message.
#[derive(Debug, Clone)]
pub struct BlockedMessage {
    pub text: String,
    pub indent_level: usize,
    pub span: Span,
}

/// `// text` — a comment, retained for potential LSP use.
#[derive(Debug, Clone)]
pub struct Comment {
    pub text: String,
    pub span: Span,
}

/// A syntax error marker — PARSE places these where recovery occurred.
#[derive(Debug, Clone)]
pub struct ErrorNode {
    pub raw_text: String,
    pub attempted_rule: Option<String>,
    pub span: Span,
}

// ── Rule blocks ──

/// `rule name:` block — a complete rule definition.
#[derive(Debug, Clone)]
pub struct RuleBlock {
    pub name: String,
    pub actor: String,
    pub trigger: String,
    pub select: Option<SelectClause>,
    pub where_clauses: Vec<ConditionExpr>,
    pub effects: Vec<Effect>,
    pub span: Span,
}

/// The `selects...from...where` clause inside a rule block.
#[derive(Debug, Clone)]
pub struct SelectClause {
    pub variable: String,
    pub entity_refs: Vec<String>,
    pub where_clauses: Vec<ConditionExpr>,
    pub span: Span,
}

// ── Condition expressions ──

/// Discriminated union of condition expression types.
/// Parsed by PARSE, consumed by LINK, VALIDATE, and EMIT.
#[derive(Debug, Clone)]
pub enum ConditionExpr {
    PropertyComparison(PropertyComparison),
    ContainmentCheck(ContainmentCheck),
    ExhaustionCheck(ExhaustionCheck),
}

/// `@entity.property op value`
#[derive(Debug, Clone)]
pub struct PropertyComparison {
    pub entity_ref: String,
    pub property: String,
    pub operator: String,
    pub value: String,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// `@entity in container` or `@entity not in container`
#[derive(Debug, Clone)]
pub struct ContainmentCheck {
    pub entity_ref: String,
    pub container_ref: String,
    pub negated: bool,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

/// `exhausted section_name`
#[derive(Debug, Clone)]
pub struct ExhaustionCheck {
    pub section_name: String,
    pub annotation: Option<Annotation>,
    pub span: Span,
}

// ── Effect subtypes ──

/// Discriminated effect types.
#[derive(Debug, Clone)]
pub enum EffectType {
    /// `> @entity.prop = value` or `> @entity.prop + N`
    Set {
        target_prop: String,
        operator: String,
        value_expr: String,
    },
    /// `> move @entity -> container`
    Move {
        entity_ref: String,
        destination_ref: String,
    },
    /// `> reveal @entity.prop`
    Reveal {
        target_prop: String,
    },
    /// `> destroy @entity`
    Destroy {
        entity_ref: String,
    },
}

// ── Annotations ──

/// Discriminator for container references in ContainmentCheck.
/// LINK resolves `container_ref` into one of these during the resolution sub-pass.
/// VALIDATE and EMIT read the discriminator, not the raw string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerKind {
    KeywordPlayer,
    KeywordHere,
    EntityRef(String),
    LocationRef(String),
}

/// Discriminator for destination references in Move effects.
/// Same resolution model as ContainerKind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationKind {
    KeywordPlayer,
    KeywordHere,
    EntityRef(String),
    LocationRef(String),
}

/// Annotation slot populated by LINK during the resolution sub-pass.
/// Initially `None` on all fields — LINK fills in resolved references.
#[derive(Debug, Clone, Default)]
pub struct Annotation {
    pub resolved_entity: Option<String>,
    pub resolved_type: Option<String>,
    pub resolved_section: Option<String>,
    pub resolved_property: Option<String>,
    pub resolved_location: Option<String>,
    pub container_kind: Option<ContainerKind>,
    pub destination_kind: Option<DestinationKind>,
}
