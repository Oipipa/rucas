use rucas::{BuiltinFunction, Differentiator, EngineContext, Expr, Function, Integrator, Symbol};

fn main() {
    let ctx = EngineContext::default();
    let x = Symbol::new("x");
    let expr = Expr::sum([
        Expr::call(
            Function::Builtin(BuiltinFunction::Sin),
            [Expr::from_symbol(x.clone())],
        ),
        Expr::integer(2),
    ]);

    let derivative = Differentiator::default().differentiate(&expr, &x, &ctx);
    let integral = Integrator::default().integrate(&expr, &x, &ctx);

    println!("expr         = {expr}");
    println!("d/d{} expr   = {}", x.name(), derivative);
    println!("int expr d{} = {}", x.name(), integral.expr);
}
