// Semantic inference retains quantified lifetime debts. Check them with each
// body's original binder and CFG before classifying a language outcome.
pub fn prove(source: &str) -> Result<(), String> {
    let (report, mut facts) = super::semantic::analyze(source)?;
    let mut program = rustgm::body::build_concrete(&report, &facts).map_err(|e| e.to_string())?;
    let mut templates =
        rustgm::body::templates::build(&report, &facts).map_err(|e| e.to_string())?;
    rustgm::body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
    rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)
        .map_err(|e| e.to_string())?;
    program
        .bodies
        .extend(templates.bodies.into_iter().map(|template| template.body));
    let contracts = rustgm::body::regions::collect(&facts).map_err(str::to_owned)?;
    rustgm::borrow::regions::check_bodies(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
