use clingo::{Model, ModelType, Part, ShowType, SolveMode, Symbol};

macro_rules! symb {
    ($num:literal) => {
        Ok(Symbol::create_number($num))
    };
    ($name:ident ( $( $( $arg:tt )* ),* )) => {
        {
            use fallible_iterator::FallibleIterator;
            let name: &str = stringify!($name);
            let args: Result<Vec<Symbol>, _> = ::fallible_iterator::convert([ $( symb!($( $arg )*) ),* ].into_iter()).collect();
            match args {
                Ok(args) => Symbol::create_function(name, &args, true),
                Err(why) => Err(why)
            }
        }
    };
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
    let symbol = symb!(b(1)).unwrap();
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
    let a_1 = symb!(a(1)).unwrap();
    let b_1 = symb!(b(1)).unwrap();
    let a_2 = symb!(a(2)).unwrap();
    let b_2 = symb!(b(2)).unwrap();
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
        "base",
        &[],
        r#"
            arg_(1..4).
            #external arg(X) : arg_(X).
            att(X, X):- arg(X).
            #show arg/1.
            #show att/2.
        "#,
    )
    .expect("Adding program to base");
    let base = Part::new("base", vec![]).unwrap();
    ctl.ground(&[base.clone()]).expect("Grounding");
    ctl.add(
        "facts",
        &[],
        r#"
            arg_(5..8).
        "#,
    )
    .unwrap();
    let facts = Part::new("facts", vec![]).unwrap();
    ctl.ground(&[base, facts]).expect("Grounding");
    let arg7 = symb!(arg(7)).unwrap();
    let atom = ctl
        .symbolic_atoms()
        .expect("Getting symbolic atoms")
        .iter()
        .expect("Iter over symbolic atoms")
        .find(|atom| {
            let s = atom.symbol().unwrap();
            s == arg7
        })
        .expect("Literal exists");
    ctl.assign_external(atom.literal().unwrap(), clingo::TruthValue::True)
        .unwrap();
    let mut solve_handle = ctl.solve(SolveMode::YIELD, &[]).unwrap();

    solve_handle.get().unwrap();
    let model = solve_handle
        .model()
        .expect("Getting Model")
        .expect("Model should exist");
    print_model(&model);
    assert!(model.contains(arg7).expect("Checking model for arg(7)"));
    let att77 = symb!(att(7, 7)).unwrap();
    let att77 = Symbol::create_function(
        "att",
        &[Symbol::create_number(7), Symbol::create_number(7)],
        true,
    )
    .unwrap();
    assert!(model.contains(att77).expect("Checking model for att(7,7)"));
}

#[test]
fn symb_macro_works() {
    let symbol = symb!(a(1));
    let symbol = symb!(a(a(1)));
}
