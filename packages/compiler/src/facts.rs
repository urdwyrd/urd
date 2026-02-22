/// Analysis IR: FactSet — normalized tuples extracted from the resolved world.
///
/// The FactSet is a flat, immutable, deterministic set of typed tuples
/// produced after LINK. Each tuple represents one atomic relationship
/// established by declarations, references, or resolutions. The facts are
/// direct translations of what LINK resolved — no inference, no transitive
/// closure, no runtime simulation.
///
/// Consumers access data through slice accessors and lookup helpers.
/// Private fields enforce immutability at the type level.

use indexmap::IndexMap;

use crate::ast::{ContentNode, ConditionExpr, EffectType};
use crate::graph::DependencyGraph;
use crate::slugify::slugify;
use crate::span::Span;
use crate::symbol_table::{PropertyType, SymbolTable};

// ── Identity type aliases ──

/// Resolved type name (e.g., "Guard").
pub type TypeId = String;
/// Resolved property name (e.g., "trust").
pub type PropertyId = String;
/// Slugified location ID.
pub type LocationId = String;
/// Compiled section ID (file_stem/section_name).
pub type SectionId = String;
/// Compiled choice ID (section_id/slugified_label).
pub type ChoiceId = String;
/// Composite exit ID: "location_id/exit_name".
pub type ExitId = String;
/// Rule identifier.
pub type RuleId = String;

// ── PropertyKey ──

/// Normalized key for property-level queries and indexing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PropertyKey {
    pub entity_type: TypeId,
    pub property: PropertyId,
}

// ── Enums ──

/// Typed comparison operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

impl CompareOp {
    fn from_token(token: &str) -> Option<CompareOp> {
        match token {
            "==" => Some(CompareOp::Eq),
            "!=" => Some(CompareOp::Ne),
            "<" => Some(CompareOp::Lt),
            ">" => Some(CompareOp::Gt),
            "<=" => Some(CompareOp::Le),
            ">=" => Some(CompareOp::Ge),
            _ => None,
        }
    }
}

/// Typed write operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOp {
    Set,
    Add,
    Sub,
}

impl WriteOp {
    fn from_token(token: &str) -> Option<WriteOp> {
        match token {
            "=" => Some(WriteOp::Set),
            "+" => Some(WriteOp::Add),
            "-" => Some(WriteOp::Sub),
            _ => None,
        }
    }
}

/// Distinguishes literal kinds for downstream reasoning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralKind {
    Bool,
    Int,
    Str,
    /// Bare identifier, typically an enum variant.
    Ident,
}

/// Typed jump destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpTarget {
    Section(SectionId),
    Exit(ExitId),
    End,
}

/// Discriminator for where a read or write occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FactSite {
    Choice(ChoiceId),
    Exit(ExitId),
    Rule(RuleId),
}

/// The result of resolving a FactSite to its owning construct.
pub enum SiteOwner<'a> {
    Choice(&'a ChoiceFact),
    Exit(&'a ExitEdge),
    Rule(&'a RuleFact),
}

// ── Fact structs ──

/// A condition reads an entity's property.
#[derive(Debug, Clone)]
pub struct PropertyRead {
    pub site: FactSite,
    pub entity_type: TypeId,
    pub property: PropertyId,
    pub operator: CompareOp,
    pub value_literal: String,
    pub value_kind: LiteralKind,
    pub span: Span,
}

impl PropertyRead {
    pub fn key(&self) -> PropertyKey {
        PropertyKey {
            entity_type: self.entity_type.clone(),
            property: self.property.clone(),
        }
    }
}

/// An effect writes an entity's property.
#[derive(Debug, Clone)]
pub struct PropertyWrite {
    pub site: FactSite,
    pub entity_type: TypeId,
    pub property: PropertyId,
    pub operator: WriteOp,
    pub value_expr: String,
    pub value_kind: Option<LiteralKind>,
    pub span: Span,
}

impl PropertyWrite {
    pub fn key(&self) -> PropertyKey {
        PropertyKey {
            entity_type: self.entity_type.clone(),
            property: self.property.clone(),
        }
    }
}

/// An exit connects two locations.
#[derive(Debug, Clone)]
pub struct ExitEdge {
    pub from_location: LocationId,
    pub to_location: LocationId,
    pub exit_name: String,
    pub is_conditional: bool,
    pub guard_reads: Vec<usize>,
    pub span: Span,
}

impl ExitEdge {
    /// Derive the canonical ExitId from components.
    pub fn exit_id(&self) -> ExitId {
        make_exit_id(&self.from_location, &self.exit_name)
    }
}

/// A jump connects two dialogue sections, or a section to an exit or terminal.
#[derive(Debug, Clone)]
pub struct JumpEdge {
    pub from_section: SectionId,
    pub target: JumpTarget,
    pub span: Span,
}

/// A choice exists within a section.
#[derive(Debug, Clone)]
pub struct ChoiceFact {
    pub section: SectionId,
    pub choice_id: ChoiceId,
    pub label: String,
    pub sticky: bool,
    pub condition_reads: Vec<usize>,
    pub effect_writes: Vec<usize>,
    pub span: Span,
}

/// A rule, with its conditions and effects indexed into the FactSet.
#[derive(Debug, Clone)]
pub struct RuleFact {
    pub rule_id: RuleId,
    pub condition_reads: Vec<usize>,
    pub effect_writes: Vec<usize>,
    pub span: Span,
}

// ── Helpers ──

/// Compose an ExitId from its components.
pub fn make_exit_id(location_id: &str, exit_name: &str) -> ExitId {
    debug_assert!(
        !exit_name.contains('/'),
        "exit_name must not contain slashes"
    );
    format!("{}/{}", location_id, exit_name)
}

/// Split an ExitId into its (location_id, exit_name) components.
pub fn split_exit_id(exit_id: &str) -> Option<(&str, &str)> {
    exit_id.split_once('/')
}

// ── FactSet container ──

/// The complete set of facts extracted from a resolved world.
/// Immutable after construction. Deterministic for a given linked compilation unit.
pub struct FactSet {
    reads: Vec<PropertyRead>,
    writes: Vec<PropertyWrite>,
    exits: Vec<ExitEdge>,
    jumps: Vec<JumpEdge>,
    choices: Vec<ChoiceFact>,
    rules: Vec<RuleFact>,
}

impl FactSet {
    // Slice accessors.

    pub fn reads(&self) -> &[PropertyRead] {
        &self.reads
    }

    pub fn writes(&self) -> &[PropertyWrite] {
        &self.writes
    }

    pub fn exits(&self) -> &[ExitEdge] {
        &self.exits
    }

    pub fn jumps(&self) -> &[JumpEdge] {
        &self.jumps
    }

    pub fn choices(&self) -> &[ChoiceFact] {
        &self.choices
    }

    pub fn rules(&self) -> &[RuleFact] {
        &self.rules
    }

    // Lookup helpers.

    pub fn choice_by_id(&self, id: &str) -> Option<&ChoiceFact> {
        self.choices.iter().find(|c| c.choice_id == id)
    }

    pub fn exit_by_id(&self, exit_id: &str) -> Option<&ExitEdge> {
        self.exits.iter().find(|e| e.exit_id() == exit_id)
    }

    pub fn exit_by_location_and_name(
        &self,
        location: &str,
        exit_name: &str,
    ) -> Option<&ExitEdge> {
        self.exits
            .iter()
            .find(|e| e.from_location == location && e.exit_name == exit_name)
    }

    pub fn rule_by_id(&self, id: &str) -> Option<&RuleFact> {
        self.rules.iter().find(|r| r.rule_id == id)
    }

    // Property-level queries.

    pub fn reads_by_key<'a>(
        &'a self,
        key: &'a PropertyKey,
    ) -> impl Iterator<Item = &'a PropertyRead> {
        self.reads
            .iter()
            .filter(move |r| r.entity_type == key.entity_type && r.property == key.property)
    }

    pub fn writes_by_key<'a>(
        &'a self,
        key: &'a PropertyKey,
    ) -> impl Iterator<Item = &'a PropertyWrite> {
        self.writes
            .iter()
            .filter(move |w| w.entity_type == key.entity_type && w.property == key.property)
    }

    pub fn sites_reading<'a>(
        &'a self,
        key: &'a PropertyKey,
    ) -> impl Iterator<Item = &'a FactSite> {
        self.reads_by_key(key).map(|r| &r.site)
    }

    pub fn sites_writing<'a>(
        &'a self,
        key: &'a PropertyKey,
    ) -> impl Iterator<Item = &'a FactSite> {
        self.writes_by_key(key).map(|w| &w.site)
    }

    // Site adjacency.

    pub fn read_indices_for_site(&self, site: &FactSite) -> &[usize] {
        match site {
            FactSite::Choice(id) => self
                .choice_by_id(id)
                .map(|c| c.condition_reads.as_slice())
                .unwrap_or(&[]),
            FactSite::Exit(id) => self
                .exit_by_id(id)
                .map(|e| e.guard_reads.as_slice())
                .unwrap_or(&[]),
            FactSite::Rule(id) => self
                .rule_by_id(id)
                .map(|r| r.condition_reads.as_slice())
                .unwrap_or(&[]),
        }
    }

    pub fn write_indices_for_site(&self, site: &FactSite) -> &[usize] {
        match site {
            FactSite::Choice(id) => self
                .choice_by_id(id)
                .map(|c| c.effect_writes.as_slice())
                .unwrap_or(&[]),
            FactSite::Rule(id) => self
                .rule_by_id(id)
                .map(|r| r.effect_writes.as_slice())
                .unwrap_or(&[]),
            FactSite::Exit(_) => &[],
        }
    }

    pub fn resolve_site<'a>(&'a self, site: &FactSite) -> Option<SiteOwner<'a>> {
        match site {
            FactSite::Choice(id) => self.choice_by_id(id).map(SiteOwner::Choice),
            FactSite::Exit(id) => self.exit_by_id(id).map(SiteOwner::Exit),
            FactSite::Rule(id) => self.rule_by_id(id).map(SiteOwner::Rule),
        }
    }
}

// ── FactSetBuilder (private) ──

/// Private builder for constructing a FactSet. Enforces immutability at the
/// type level — only `extract_facts()` can produce a FactSet.
struct FactSetBuilder {
    reads: Vec<PropertyRead>,
    writes: Vec<PropertyWrite>,
    exits: Vec<ExitEdge>,
    jumps: Vec<JumpEdge>,
    choices: Vec<ChoiceFact>,
    rules: Vec<RuleFact>,
}

impl FactSetBuilder {
    fn new() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
            exits: Vec::new(),
            jumps: Vec::new(),
            choices: Vec::new(),
            rules: Vec::new(),
        }
    }

    fn push_read(&mut self, read: PropertyRead) -> usize {
        let idx = self.reads.len();
        self.reads.push(read);
        idx
    }

    fn push_write(&mut self, write: PropertyWrite) -> usize {
        let idx = self.writes.len();
        self.writes.push(write);
        idx
    }

    fn push_exit(&mut self, edge: ExitEdge) {
        self.exits.push(edge);
    }

    fn push_jump(&mut self, edge: JumpEdge) {
        self.jumps.push(edge);
    }

    fn push_choice(&mut self, choice: ChoiceFact) {
        self.choices.push(choice);
    }

    fn push_rule(&mut self, rule: RuleFact) {
        self.rules.push(rule);
    }

    /// Set guard reads on an exit identified by location and exit name.
    fn set_exit_guard_reads(&mut self, from_loc: &str, exit_name: &str, reads: Vec<usize>) {
        if let Some(edge) = self
            .exits
            .iter_mut()
            .find(|e| e.from_location == from_loc && e.exit_name == exit_name)
        {
            edge.guard_reads = reads;
        }
    }

    fn finish(self) -> FactSet {
        FactSet {
            reads: self.reads,
            writes: self.writes,
            exits: self.exits,
            jumps: self.jumps,
            choices: self.choices,
            rules: self.rules,
        }
    }
}

// ── Extraction ──

/// Classify a literal value's kind based on the resolved property type.
fn classify_literal(property_type: &PropertyType) -> LiteralKind {
    match property_type {
        PropertyType::Boolean => LiteralKind::Bool,
        PropertyType::Integer | PropertyType::Number => LiteralKind::Int,
        PropertyType::Enum => LiteralKind::Ident,
        PropertyType::String => LiteralKind::Str,
        _ => LiteralKind::Str, // Ref, List — fallback
    }
}

/// Classify the value_kind for a PropertyWrite.
fn classify_write_value_kind(
    operator: &WriteOp,
    value_expr: &str,
    property_type: &PropertyType,
) -> Option<LiteralKind> {
    match operator {
        WriteOp::Set => Some(classify_literal(property_type)),
        WriteOp::Add | WriteOp::Sub => {
            if value_expr.parse::<i64>().is_ok() || value_expr.parse::<f64>().is_ok() {
                Some(LiteralKind::Int)
            } else {
                None
            }
        }
    }
}

/// Look up a property's type from the symbol table.
fn lookup_property_type<'a>(
    entity_type: &str,
    property: &str,
    symbol_table: &'a SymbolTable,
) -> Option<&'a PropertyType> {
    symbol_table
        .types
        .get(entity_type)
        .and_then(|t| t.properties.get(property))
        .map(|p| &p.property_type)
}

/// Extract normalized analysis facts from the resolved world.
/// Called after LINK, before or during VALIDATE.
/// Read-only — does not modify the graph or symbol table.
/// Deterministic — same input always produces same output.
pub fn extract_facts(graph: &DependencyGraph, symbol_table: &SymbolTable) -> FactSet {
    let mut builder = FactSetBuilder::new();
    let ordered: Vec<String> = graph.topological_order().into_iter().cloned().collect();

    // Phase A: Extract exits from symbol table.
    for (loc_id, loc_sym) in &symbol_table.locations {
        for (exit_name, exit_sym) in &loc_sym.exits {
            if let Some(dest) = &exit_sym.resolved_destination {
                builder.push_exit(ExitEdge {
                    from_location: loc_id.clone(),
                    to_location: dest.clone(),
                    exit_name: exit_name.clone(),
                    is_conditional: exit_sym.condition_node.is_some(),
                    guard_reads: Vec::new(), // populated in Phase B
                    span: exit_sym.declared_in.clone(),
                });
            }
        }
    }

    // Phase B: Walk AST content in topological file order.
    for file_path in &ordered {
        let file_node = match graph.nodes.get(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };
        let file_stem = crate::graph::file_stem(file_path);
        let mut current_location_id: Option<String> = None;
        let mut current_section_id: Option<String> = None;

        for node in &file_node.ast.content {
            extract_top_level_node(
                node,
                &mut builder,
                symbol_table,
                &file_stem,
                &mut current_location_id,
                &mut current_section_id,
            );
        }
    }

    builder.finish()
}

/// Extract facts from a top-level content node (not inside a choice).
fn extract_top_level_node(
    node: &ContentNode,
    builder: &mut FactSetBuilder,
    symbol_table: &SymbolTable,
    file_stem: &str,
    current_location_id: &mut Option<String>,
    current_section_id: &mut Option<String>,
) {
    match node {
        ContentNode::LocationHeading(lh) => {
            let slug = slugify(&lh.display_name);
            if symbol_table.locations.contains_key(&slug) {
                *current_location_id = Some(slug);
            } else {
                *current_location_id = None;
            }
        }

        ContentNode::SectionLabel(sl) => {
            // Use compiled_id from symbol table — never recompute from file_stem/name.
            let lookup_key = format!("{}/{}", file_stem, sl.name);
            if let Some(section_sym) = symbol_table.sections.get(&lookup_key) {
                *current_section_id = Some(section_sym.compiled_id.clone());
            } else {
                *current_section_id = None;
            }
        }

        ContentNode::Choice(choice) => {
            extract_choice(
                choice,
                builder,
                symbol_table,
                file_stem,
                current_location_id,
                current_section_id,
            );
        }

        ContentNode::Jump(jump) => {
            extract_jump(jump, builder, current_section_id);
        }

        ContentNode::ExitDeclaration(exit_decl) => {
            extract_exit_guards(exit_decl, builder, symbol_table, current_location_id);
        }

        ContentNode::RuleBlock(rule_block) => {
            extract_rule(rule_block, builder, symbol_table);
        }

        // Other top-level nodes (prose, speech, conditions outside choices) are not facts.
        _ => {}
    }
}

/// Extract a choice and all its children (conditions, effects, nested choices).
fn extract_choice(
    choice: &crate::ast::Choice,
    builder: &mut FactSetBuilder,
    symbol_table: &SymbolTable,
    file_stem: &str,
    current_location_id: &mut Option<String>,
    current_section_id: &mut Option<String>,
) {
    let section_id = match current_section_id {
        Some(ref id) => id.clone(),
        None => return,
    };

    // Look up the choice in the symbol table to get its compiled_id.
    let choice_sym = symbol_table
        .sections
        .get(&section_id)
        .and_then(|s| {
            s.choices
                .iter()
                .find(|c| c.label == choice.label && c.sticky == choice.sticky)
        });

    let choice_id = match choice_sym {
        Some(cs) => cs.compiled_id.clone(),
        None => return,
    };

    let site = FactSite::Choice(choice_id.clone());
    let mut condition_reads: Vec<usize> = Vec::new();
    let mut effect_writes: Vec<usize> = Vec::new();

    // Walk choice children.
    for child in &choice.content {
        match child {
            ContentNode::Condition(cond) => {
                if let ConditionExpr::PropertyComparison(pc) = &cond.expr {
                    if let Some(idx) =
                        extract_property_read(pc, &site, symbol_table, builder)
                    {
                        condition_reads.push(idx);
                    }
                }
            }

            ContentNode::OrConditionBlock(or_block) => {
                for expr in &or_block.conditions {
                    if let ConditionExpr::PropertyComparison(pc) = expr {
                        if let Some(idx) =
                            extract_property_read(pc, &site, symbol_table, builder)
                        {
                            condition_reads.push(idx);
                        }
                    }
                }
            }

            ContentNode::Effect(effect) => {
                if let Some(idx) =
                    extract_property_write(effect, &site, symbol_table, builder)
                {
                    effect_writes.push(idx);
                }
            }

            ContentNode::Choice(nested) => {
                // Nested choices get their own ChoiceFact.
                extract_choice(
                    nested,
                    builder,
                    symbol_table,
                    file_stem,
                    current_location_id,
                    current_section_id,
                );
            }

            ContentNode::Jump(jump) => {
                extract_jump(jump, builder, current_section_id);
            }

            _ => {}
        }
    }

    builder.push_choice(ChoiceFact {
        section: section_id,
        choice_id,
        label: choice.label.clone(),
        sticky: choice.sticky,
        condition_reads,
        effect_writes,
        span: choice.span.clone(),
    });
}

/// Extract a JumpEdge from a Jump node.
fn extract_jump(
    jump: &crate::ast::Jump,
    builder: &mut FactSetBuilder,
    current_section_id: &mut Option<String>,
) {
    let ann = match &jump.annotation {
        Some(a) => a,
        None => return,
    };

    let section_id = match current_section_id {
        Some(ref id) => id.clone(),
        None => return,
    };

    // Determine JumpTarget from annotation.
    let target = if let Some(ref section) = ann.resolved_section {
        JumpTarget::Section(section.clone())
    } else if let Some(ref loc_id) = ann.resolved_location {
        // Exit jump — verify the exit exists in the builder.
        let exit_id = make_exit_id(loc_id, &jump.target);
        if builder.exits.iter().any(|e| e.exit_id() == exit_id) {
            JumpTarget::Exit(exit_id)
        } else {
            return; // Exit destination unresolved, no JumpEdge
        }
    } else if jump.target == "end" {
        JumpTarget::End
    } else {
        return; // Unresolvable
    };

    builder.push_jump(JumpEdge {
        from_section: section_id,
        target,
        span: jump.span.clone(),
    });
}

/// Extract guard condition reads from an ExitDeclaration's children.
fn extract_exit_guards(
    exit_decl: &crate::ast::ExitDeclaration,
    builder: &mut FactSetBuilder,
    symbol_table: &SymbolTable,
    current_location_id: &mut Option<String>,
) {
    let loc_id = match current_location_id {
        Some(ref id) => id.clone(),
        None => return,
    };

    let mut guard_read_indices: Vec<usize> = Vec::new();
    let exit_id = make_exit_id(&loc_id, &exit_decl.direction);

    for child in &exit_decl.children {
        if let ContentNode::Condition(cond) = child {
            if let ConditionExpr::PropertyComparison(pc) = &cond.expr {
                let site = FactSite::Exit(exit_id.clone());
                if let Some(idx) = extract_property_read(pc, &site, symbol_table, builder) {
                    guard_read_indices.push(idx);
                }
            }
        }
    }

    if !guard_read_indices.is_empty() {
        builder.set_exit_guard_reads(&loc_id, &exit_decl.direction, guard_read_indices);
    }
}

/// Extract a RuleFact from a RuleBlock node.
fn extract_rule(
    rule_block: &crate::ast::RuleBlock,
    builder: &mut FactSetBuilder,
    symbol_table: &SymbolTable,
) {
    let rule_id = rule_block.name.clone();
    let site = FactSite::Rule(rule_id.clone());
    let mut condition_reads: Vec<usize> = Vec::new();
    let mut effect_writes: Vec<usize> = Vec::new();

    // Walk rule where_clauses for PropertyReads.
    for expr in &rule_block.where_clauses {
        if let ConditionExpr::PropertyComparison(pc) = expr {
            if let Some(idx) = extract_property_read(pc, &site, symbol_table, builder) {
                condition_reads.push(idx);
            }
        }
    }

    // Walk select clause where_clauses if present.
    if let Some(ref select) = rule_block.select {
        for expr in &select.where_clauses {
            if let ConditionExpr::PropertyComparison(pc) = expr {
                if let Some(idx) = extract_property_read(pc, &site, symbol_table, builder) {
                    condition_reads.push(idx);
                }
            }
        }
    }

    // Walk rule effects for PropertyWrites.
    for effect in &rule_block.effects {
        if let Some(idx) = extract_property_write(effect, &site, symbol_table, builder) {
            effect_writes.push(idx);
        }
    }

    builder.push_rule(RuleFact {
        rule_id,
        condition_reads,
        effect_writes,
        span: rule_block.span.clone(),
    });
}

/// Extract a PropertyRead from a PropertyComparison with a known FactSite.
/// Returns the index into builder.reads if successful.
fn extract_property_read(
    pc: &crate::ast::PropertyComparison,
    site: &FactSite,
    symbol_table: &SymbolTable,
    builder: &mut FactSetBuilder,
) -> Option<usize> {
    let ann = pc.annotation.as_ref()?;
    let resolved_type = ann.resolved_type.as_ref()?;
    let resolved_property = ann.resolved_property.as_ref()?;

    let compare_op = CompareOp::from_token(&pc.operator)?;

    let prop_type = lookup_property_type(resolved_type, resolved_property, symbol_table);
    let value_kind = prop_type
        .map(|pt| classify_literal(pt))
        .unwrap_or(LiteralKind::Str);

    let idx = builder.push_read(PropertyRead {
        site: site.clone(),
        entity_type: resolved_type.clone(),
        property: resolved_property.clone(),
        operator: compare_op,
        value_literal: pc.value.clone(),
        value_kind,
        span: pc.span.clone(),
    });

    Some(idx)
}

/// Extract a PropertyWrite from an Effect node with a known FactSite.
/// Only handles EffectType::Set — lifecycle effects are out of scope.
/// Returns the index into builder.writes if successful.
fn extract_property_write(
    effect: &crate::ast::Effect,
    site: &FactSite,
    symbol_table: &SymbolTable,
    builder: &mut FactSetBuilder,
) -> Option<usize> {
    if let EffectType::Set {
        target_prop: _,
        operator,
        value_expr,
    } = &effect.effect_type
    {
        let ann = effect.annotation.as_ref()?;
        let resolved_type = ann.resolved_type.as_ref()?;
        let resolved_property = ann.resolved_property.as_ref()?;

        let write_op = WriteOp::from_token(operator)?;

        let prop_type = lookup_property_type(resolved_type, resolved_property, symbol_table);
        let value_kind = prop_type
            .map(|pt| classify_write_value_kind(&write_op, value_expr, pt))
            .flatten();

        let idx = builder.push_write(PropertyWrite {
            site: site.clone(),
            entity_type: resolved_type.clone(),
            property: resolved_property.clone(),
            operator: write_op,
            value_expr: value_expr.clone(),
            value_kind,
            span: effect.span.clone(),
        });

        return Some(idx);
    }

    None
}

// ── PropertyDependencyIndex ──

/// Index mapping (type, property) pairs to their read and write sites.
/// Built from the FactSet as a derived secondary index.
pub struct PropertyDependencyIndex {
    readers: IndexMap<PropertyKey, Vec<usize>>,
    writers: IndexMap<PropertyKey, Vec<usize>>,
}

impl PropertyDependencyIndex {
    /// Build the index from a FactSet. Single pass over reads and writes.
    pub fn build(fact_set: &FactSet) -> Self {
        let mut readers: IndexMap<PropertyKey, Vec<usize>> = IndexMap::new();
        let mut writers: IndexMap<PropertyKey, Vec<usize>> = IndexMap::new();

        for (i, read) in fact_set.reads().iter().enumerate() {
            readers.entry(read.key()).or_default().push(i);
        }
        for (i, write) in fact_set.writes().iter().enumerate() {
            writers.entry(write.key()).or_default().push(i);
        }

        Self { readers, writers }
    }

    /// All read indices for a given (type, property) pair.
    pub fn reads_of(&self, key: &PropertyKey) -> &[usize] {
        self.readers
            .get(key)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// All write indices for a given property key.
    pub fn writes_of(&self, key: &PropertyKey) -> &[usize] {
        self.writers
            .get(key)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// All property keys that are read anywhere.
    pub fn read_properties(&self) -> impl Iterator<Item = &PropertyKey> {
        self.readers.keys()
    }

    /// All property keys that are written anywhere.
    pub fn written_properties(&self) -> impl Iterator<Item = &PropertyKey> {
        self.writers.keys()
    }
}

// ── JSON serialisation ──

fn span_to_json(span: &Span) -> serde_json::Value {
    serde_json::json!({
        "file": span.file,
        "start_line": span.start_line,
        "start_col": span.start_col,
        "end_line": span.end_line,
        "end_col": span.end_col,
    })
}

fn site_to_json(site: &FactSite) -> serde_json::Value {
    match site {
        FactSite::Choice(id) => serde_json::json!({ "kind": "choice", "id": id }),
        FactSite::Exit(id) => serde_json::json!({ "kind": "exit", "id": id }),
        FactSite::Rule(id) => serde_json::json!({ "kind": "rule", "id": id }),
    }
}

fn compare_op_str(op: &CompareOp) -> &'static str {
    match op {
        CompareOp::Eq => "==",
        CompareOp::Ne => "!=",
        CompareOp::Lt => "<",
        CompareOp::Gt => ">",
        CompareOp::Le => "<=",
        CompareOp::Ge => ">=",
    }
}

fn write_op_str(op: &WriteOp) -> &'static str {
    match op {
        WriteOp::Set => "=",
        WriteOp::Add => "+",
        WriteOp::Sub => "-",
    }
}

fn literal_kind_str(kind: &LiteralKind) -> &'static str {
    match kind {
        LiteralKind::Bool => "bool",
        LiteralKind::Int => "int",
        LiteralKind::Str => "str",
        LiteralKind::Ident => "ident",
    }
}

fn jump_target_to_json(target: &JumpTarget) -> serde_json::Value {
    match target {
        JumpTarget::Section(id) => serde_json::json!({ "kind": "section", "id": id }),
        JumpTarget::Exit(id) => serde_json::json!({ "kind": "exit", "id": id }),
        JumpTarget::End => serde_json::json!({ "kind": "end" }),
    }
}

impl FactSet {
    /// Serialise the FactSet to a JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "reads": self.reads.iter().map(|r| serde_json::json!({
                "site": site_to_json(&r.site),
                "entity_type": r.entity_type,
                "property": r.property,
                "operator": compare_op_str(&r.operator),
                "value_literal": r.value_literal,
                "value_kind": literal_kind_str(&r.value_kind),
                "span": span_to_json(&r.span),
            })).collect::<Vec<_>>(),
            "writes": self.writes.iter().map(|w| serde_json::json!({
                "site": site_to_json(&w.site),
                "entity_type": w.entity_type,
                "property": w.property,
                "operator": write_op_str(&w.operator),
                "value_expr": w.value_expr,
                "value_kind": w.value_kind.as_ref().map(literal_kind_str),
                "span": span_to_json(&w.span),
            })).collect::<Vec<_>>(),
            "exits": self.exits.iter().map(|e| serde_json::json!({
                "from_location": e.from_location,
                "to_location": e.to_location,
                "exit_name": e.exit_name,
                "is_conditional": e.is_conditional,
                "guard_reads": e.guard_reads,
                "span": span_to_json(&e.span),
            })).collect::<Vec<_>>(),
            "jumps": self.jumps.iter().map(|j| serde_json::json!({
                "from_section": j.from_section,
                "target": jump_target_to_json(&j.target),
                "span": span_to_json(&j.span),
            })).collect::<Vec<_>>(),
            "choices": self.choices.iter().map(|c| serde_json::json!({
                "section": c.section,
                "choice_id": c.choice_id,
                "label": c.label,
                "sticky": c.sticky,
                "condition_reads": c.condition_reads,
                "effect_writes": c.effect_writes,
                "span": span_to_json(&c.span),
            })).collect::<Vec<_>>(),
            "rules": self.rules.iter().map(|r| serde_json::json!({
                "rule_id": r.rule_id,
                "condition_reads": r.condition_reads,
                "effect_writes": r.effect_writes,
                "span": span_to_json(&r.span),
            })).collect::<Vec<_>>(),
        })
    }
}

// ── Test helpers ──

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpanKey {
    pub file: crate::span::FilePath,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[cfg(test)]
impl From<&Span> for SpanKey {
    fn from(span: &Span) -> Self {
        SpanKey {
            file: span.file.clone(),
            start_line: span.start_line,
            start_col: span.start_col,
            end_line: span.end_line,
            end_col: span.end_col,
        }
    }
}
