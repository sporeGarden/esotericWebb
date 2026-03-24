// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp003: State emergence and predicate evaluation.
//!
//! Validates that `WorldState` predicates, effects, conditions, trust,
//! inventory, and knowledge interact correctly — the combinatorial space
//! that produces emergent paths through authored structure.

fn main() {
    use esoteric_webb::experiment::{check_bool, exit};
    use esoteric_webb::narrative::effect::StateEffect;
    use esoteric_webb::narrative::predicate::StatePredicate;
    use esoteric_webb::state::WorldState;

    println!("exp003: state emergence");

    let mut state = WorldState::new();

    // Knowledge via apply()
    state.apply(&StateEffect::AddKnowledge("ancient_lore".to_owned()));
    check_bool(
        "knowledge added",
        state.evaluate(&StatePredicate::HasKnowledge("ancient_lore".to_owned())),
    );
    check_bool(
        "missing knowledge returns false",
        !state.evaluate(&StatePredicate::HasKnowledge("unknown".to_owned())),
    );

    // Inventory
    state.apply(&StateEffect::AddItem("crystal_shard".to_owned()));
    check_bool(
        "item in inventory",
        state.evaluate(&StatePredicate::HasItem("crystal_shard".to_owned())),
    );
    state.apply(&StateEffect::RemoveItem("crystal_shard".to_owned()));
    check_bool(
        "item removed",
        state.evaluate(&StatePredicate::LacksItem("crystal_shard".to_owned())),
    );

    // Trust
    state.apply(&StateEffect::ModifyTrust("maren".to_owned(), 10));
    check_bool(
        "trust increased",
        state.evaluate(&StatePredicate::TrustAbove("maren".to_owned(), 10)),
    );
    state.apply(&StateEffect::ModifyTrust("maren".to_owned(), -5));
    check_bool(
        "trust decreased",
        state.evaluate(&StatePredicate::TrustAbove("maren".to_owned(), 5)),
    );

    // Flags
    state.apply(&StateEffect::SetFlag("door_opened".to_owned()));
    check_bool(
        "flag set",
        state.evaluate(&StatePredicate::FlagSet("door_opened".to_owned())),
    );
    state.apply(&StateEffect::ClearFlag("door_opened".to_owned()));
    check_bool(
        "flag cleared",
        state.evaluate(&StatePredicate::FlagUnset("door_opened".to_owned())),
    );

    // Conditions with duration
    state.apply(&StateEffect::ApplyCondition("poisoned".to_owned(), 3));
    check_bool(
        "condition active",
        state.evaluate(&StatePredicate::ConditionActive("poisoned".to_owned())),
    );
    state.tick_conditions();
    state.tick_conditions();
    state.tick_conditions();
    check_bool(
        "timed condition expired",
        state.evaluate(&StatePredicate::ConditionInactive("poisoned".to_owned())),
    );

    // Permanent conditions (duration 0)
    state.apply(&StateEffect::ApplyCondition("blessed".to_owned(), 0));
    check_bool(
        "permanent condition active",
        state.evaluate(&StatePredicate::ConditionActive("blessed".to_owned())),
    );
    for _ in 0..10 {
        state.tick_conditions();
    }
    check_bool(
        "permanent condition persists",
        state.evaluate(&StatePredicate::ConditionActive("blessed".to_owned())),
    );

    // Arc phases
    state.apply(&StateEffect::AdvanceArc(
        "main_quest".to_owned(),
        "act_2".to_owned(),
    ));
    check_bool(
        "arc phase advanced",
        state.evaluate(&StatePredicate::ArcPhaseIs(
            "main_quest".to_owned(),
            "act_2".to_owned(),
        )),
    );

    // Compound predicates
    let all = StatePredicate::All(vec![
        StatePredicate::HasKnowledge("ancient_lore".to_owned()),
        StatePredicate::ConditionActive("blessed".to_owned()),
    ]);
    check_bool("compound All predicate", state.evaluate(&all));

    let not = StatePredicate::Not(Box::new(StatePredicate::HasItem("nonexistent".to_owned())));
    check_bool("compound Not predicate", state.evaluate(&not));

    exit("exp003_state_emergence");
}
