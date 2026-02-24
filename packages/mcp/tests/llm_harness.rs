/// LLM validation harness for MCP semantic interface (SF-6.4).
///
/// Five structural questions about the Locked Garden world, each with a
/// known correct answer derived from the FactSet. Verifiers are
/// substring/pattern matchers, not exact comparisons.
///
/// NOT a CI test. Requires an LLM with MCP tool access and API credentials.
/// Run manually: `cargo test --test llm_harness -- --ignored --nocapture`
///
/// Gate: minimum 4/5 pass with at least one LLM model.

#[allow(dead_code)]
struct Question {
    prompt: &'static str,
    verifier: fn(llm_response: &str) -> bool,
}

fn questions() -> Vec<Question> {
    vec![
        // Q1: Exit condition from Gatehouse to Walled Garden
        Question {
            prompt: "What condition must be met to travel from the Gatehouse to the Walled Garden?",
            verifier: |response| {
                let r = response.to_lowercase();
                (r.contains("locked") && r.contains("false"))
                    || r.contains("garden_gate.locked")
                    || (r.contains("lock") && r.contains("unlocked"))
            },
        },
        // Q2: Character entities
        Question {
            prompt: "Which entities are of type Character?",
            verifier: |response| {
                let r = response.to_lowercase();
                r.contains("warden") && r.contains("ghost")
            },
        },
        // Q3: Ways @warden.trust can change in the gatehouse
        Question {
            prompt: "What are all the ways @warden.trust can change in the gatehouse?",
            verifier: |response| {
                let r = response.to_lowercase();
                // +1 (State your purpose), +5 (Offer the journal), -2 (Force the gate)
                (r.contains("+1") || r.contains("+ 1") || r.contains("plus 1"))
                    && (r.contains("+5") || r.contains("+ 5") || r.contains("plus 5"))
                    && (r.contains("-2") || r.contains("- 2") || r.contains("minus 2"))
            },
        },
        // Q4: Effects of choosing "Offer the journal"
        Question {
            prompt: "What effects does choosing 'Offer the journal' have?",
            verifier: |response| {
                let r = response.to_lowercase();
                // trust +5, mood = friendly
                (r.contains("trust") && (r.contains("+5") || r.contains("+ 5") || r.contains("5")))
                    && (r.contains("mood") && r.contains("friendly"))
            },
        },
        // Q5: How can the garden gate be unlocked
        Question {
            prompt: "How can the garden gate be unlocked?",
            verifier: |response| {
                let r = response.to_lowercase();
                // Trust threshold (3) and locked = false both present
                r.contains("trust") && r.contains("3") && r.contains("locked")
            },
        },
    ]
}

/// Validate verifiers against known-correct reference answers.
///
/// This test runs without an LLM — it checks that the verifier functions
/// accept the expected reference answers. Useful for ensuring the verifiers
/// don't accidentally reject correct responses.
#[test]
fn verifiers_accept_reference_answers() {
    let reference_answers = vec![
        "The exit from Gatehouse to the Walled Garden requires @garden_gate.locked == false.",
        "The entities of type Character are @warden and @ghost.",
        "@warden.trust changes: +1 (State your purpose), +5 (Offer the journal), -2 (Force the gate).",
        "Choosing 'Offer the journal' sets @warden.trust += 5 and @warden.mood = friendly.",
        "Choose 'Ask about the garden' when @warden.trust >= 3 to set @garden_gate.locked = false.",
    ];

    let qs = questions();
    for (i, (q, answer)) in qs.iter().zip(reference_answers.iter()).enumerate() {
        assert!(
            (q.verifier)(answer),
            "Verifier {} rejected reference answer: {}",
            i + 1,
            answer
        );
    }
}

/// Placeholder for the full LLM validation run.
///
/// To implement:
/// 1. Start `urd-mcp locked-garden.urd.md` as a subprocess
/// 2. Connect an LLM client with MCP tool access
/// 3. For each question, send to LLM, collect response, run verifier
/// 4. Assert >= 4/5 pass
#[test]
#[ignore]
fn llm_validation_run() {
    let qs = questions();
    let passed = 0;
    let total = qs.len();

    // TODO: Replace with actual LLM integration
    // For each question:
    //   let response = llm_with_mcp_tools(q.prompt);
    //   if (q.verifier)(&response) { passed += 1; }

    eprintln!(
        "LLM validation: {}/{} passed (gate: {}/{})",
        passed,
        total,
        4,
        total
    );
    assert!(
        passed >= 4,
        "LLM validation gate failed: {}/{} (need 4/{})",
        passed,
        total,
        total
    );
}
