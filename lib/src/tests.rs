use clingo::{Model, ModelType, Part, ShowType, SolveMode, Symbol, ToSymbol};

/// Just some symbols for testing purposes
mod symb {
    use clingo::{ClingoError, Symbol, ToSymbol};

    #[derive(ToSymbol)]
    pub struct A(pub i32);
    #[derive(ToSymbol)]
    pub struct B(pub i32);
    #[derive(ToSymbol)]
    pub struct C;
}

// This is just copy-pasted from clingo's examples to have some human feedback
fn print_model(model: &Model) {
    // get model type
    let model_type = model.model_type().unwrap();

    let type_string = match model_type {
        ModelType::StableModel => "Stable model",
        ModelType::BraveConsequences => "Brave consequences",
        ModelType::CautiousConsequences => "Cautious consequences",
    };

    // get running number of model
    let number = model.number().unwrap();

    println!("{}: {}", type_string, number);

    fn print(model: &Model, label: &str, show: ShowType) {
        print!("{}:", label);

        // retrieve the symbols in the model
        let atoms = model
            .symbols(show)
            .expect("Failed to retrieve symbols in the model.");

        for symbol in atoms {
            print!(" {}", symbol);
        }
        println!();
    }

    print(model, "  shown", ShowType::SHOWN);
    print(model, "  atoms", ShowType::ATOMS);
    print(model, "  terms", ShowType::TERMS);
    print(model, " ~atoms", ShowType::COMPLEMENT | ShowType::ATOMS);
}

#[test]
fn i_correctly_understand_clingos_symbolic_atoms() {
    let mut ctl = clingo::control(vec![]).expect("Init control");
    ctl.add(
        "base",
        &[],
        r#"
            a(1).
            a(2).
            b(X): a(X).
            c.
        "#,
    )
    .expect("Adding program to base");
    let base = Part::new("base", vec![]).unwrap();
    ctl.ground(&[base]).expect("Grounding");
    let symbol = symb::B(1).symbol().unwrap();
    let mut found_it = false;
    ctl.symbolic_atoms()
        .expect("Getting symbolic atoms")
        .iter()
        .expect("Iter over symbolic atoms")
        .for_each(|atom| {
            let s = atom.symbol().unwrap();
            eprintln!("{:?} ? {}", s, s == symbol);
            if s == symbol {
                eprintln!("{:?}", atom.literal().unwrap());
                found_it = true;
            }
        });
    assert!(found_it)
}

#[test]
fn i_can_actually_repeat_the_grounding_to_add_facts() {
    let mut ctl = clingo::control(vec![]).expect("Init control");
    ctl.add(
        "base",
        &[],
        r#"
            a(1).
            b(X):- a(X).
        "#,
    )
    .expect("Adding program to base");
    let base = Part::new("base", vec![]).unwrap();
    ctl.ground(&[base.clone()]).expect("Grounding");
    // Get the first model
    let mut solve_handle = ctl.solve(SolveMode::YIELD, &[]).expect("Solving");
    solve_handle.get().unwrap();
    let model = solve_handle
        .model()
        .expect("Getting Model")
        .expect("Model should exist");
    print_model(&model);
    let a_1 = symb::A(1).symbol().unwrap();
    let b_1 = symb::A(1).symbol().unwrap();
    let a_2 = symb::A(2).symbol().unwrap();
    let b_2 = symb::A(2).symbol().unwrap();
    assert!(model.contains(a_1).expect("Checking model for a(1)"));
    assert!(model.contains(b_1).expect("Checking model for b(1)"));
    assert!(!model.contains(a_2).expect("Checking model for a(2)"));
    assert!(!model.contains(b_2).expect("Checking model for b(2)"));
    ctl = solve_handle.close().expect("Returning solve handle");
    // == SECOND GROUNDING ==
    ctl.add(
        "base",
        &[],
        r#"
            a(2).
        "#,
    )
    .expect("Adding second fact");
    ctl.ground(&[base]).expect("Grounding");
    // Get the first model
    let mut solve_handle = ctl.solve(SolveMode::YIELD, &[]).expect("Solving");
    solve_handle.get().unwrap();
    let model = solve_handle
        .model()
        .expect("Getting Model")
        .expect("Model should exist");
    print_model(&model);
    assert!(model.contains(a_1).expect("Checking model for a(1)"));
    assert!(model.contains(b_1).expect("Checking model for b(1)"));
    assert!(model.contains(a_2).expect("Checking model for a(2)"));
    assert!(model.contains(b_2).expect("Checking model for b(2)"));
}

#[test]
fn i_understand_grounding_now() {
    let mut ctl = clingo::control(vec![]).expect("Init control");
    ctl.add(
        "theory",
        &[],
        r#"
            #external arg(X) : arg_(X).
            att(X, X):- arg(X).
        "#,
    )
    .expect("Adding theory program");
    ctl.add(
        "base",
        &[],
        r#"
            arg_("1";"2";"3";"4").
            #show arg/1.
            #show att/2.
        "#,
    )
    .expect("Adding program to base");
    let base = Part::new("base", vec![]).unwrap();
    let theory = Part::new("theory", vec![]).unwrap();
    // Grounding both `base` and `theory` will add the external toggles
    // and attacks for the `arg_` in the base
    ctl.ground(&[base.clone(), theory.clone()])
        .expect("Grounding");
    // Adding additional `arg_`s is now possible
    ctl.add(
        "update",
        &[],
        r#"
            arg_("5";"6";"7";"8").
        "#,
    )
    .unwrap();
    let update = Part::new("update", vec![]).unwrap();
    // We have to reground theory aswell, to find the required
    // atom below. Try removing `theory` from the list of programs to make this test fail
    ctl.ground(&[update, theory]).expect("Grounding");
    let arg7 =
        Symbol::create_function("arg", &[Symbol::create_string("7").unwrap()], true).unwrap();
    let atom = ctl
        .symbolic_atoms()
        .expect("Getting symbolic atoms")
        .iter()
        .expect("Iter over symbolic atoms")
        .find(|atom| {
            let s = atom.symbol().unwrap();
            eprintln!("atom {s} {arg7}");
            s == arg7
        })
        .expect("Literal exists");
    // We can now assign a truth-value to the atom we've selected above
    // Removing this makes this test fail aswell
    ctl.assign_external(atom.literal().unwrap(), clingo::TruthValue::True)
        .unwrap();
    // Solving will now yield `arg(7)` and `att(7,7)` as defined by our theory
    let mut solve_handle = ctl.solve(SolveMode::YIELD, &[]).unwrap();
    solve_handle.get().unwrap();
    let model = solve_handle
        .model()
        .expect("Getting Model")
        .expect("Model should exist");
    print_model(&model);
    assert!(model.contains(arg7).expect("Checking model for arg(7)"));
    let att77 = Symbol::create_function(
        "att",
        &[
            Symbol::create_string("7").unwrap(),
            Symbol::create_string("7").unwrap(),
        ],
        true,
    )
    .unwrap();
    assert!(model.contains(att77).expect("Checking model for att(7,7)"));
}
