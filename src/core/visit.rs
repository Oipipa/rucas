use super::{Expr, ExprKind};

pub trait Visitor {
    fn enter(&mut self, _expr: &Expr) {}

    fn exit(&mut self, _expr: &Expr) {}
}

pub fn walk(expr: &Expr, visitor: &mut impl Visitor) {
    visitor.enter(expr);

    match expr.kind() {
        ExprKind::Number(_) | ExprKind::Symbol(_) => {}
        ExprKind::Add(terms) | ExprKind::Mul(terms) => {
            for term in terms {
                walk(term, visitor);
            }
        }
        ExprKind::Pow { base, exp } => {
            walk(base, visitor);
            walk(exp, visitor);
        }
        ExprKind::Call { args, .. } => {
            for arg in args {
                walk(arg, visitor);
            }
        }
        ExprKind::Derivative(derivative) => walk(&derivative.expr, visitor),
        ExprKind::Integral(integral) => walk(&integral.expr, visitor),
    }

    visitor.exit(expr);
}

pub fn any(expr: &Expr, predicate: &mut impl FnMut(&Expr) -> bool) -> bool {
    if predicate(expr) {
        return true;
    }

    match expr.kind() {
        ExprKind::Number(_) | ExprKind::Symbol(_) => false,
        ExprKind::Add(terms) | ExprKind::Mul(terms) => {
            terms.iter().any(|term| any(term, predicate))
        }
        ExprKind::Pow { base, exp } => any(base, predicate) || any(exp, predicate),
        ExprKind::Call { args, .. } => args.iter().any(|arg| any(arg, predicate)),
        ExprKind::Derivative(derivative) => any(&derivative.expr, predicate),
        ExprKind::Integral(integral) => any(&integral.expr, predicate),
    }
}

pub fn contains(expr: &Expr, needle: &Expr) -> bool {
    any(expr, &mut |candidate| candidate == needle)
}

pub trait Folder {
    fn fold(&mut self, expr: &Expr) -> Expr
    where
        Self: Sized,
    {
        fold_expr(self, expr)
    }

    fn rewrite(&mut self, expr: Expr) -> Expr {
        expr
    }
}

pub fn fold_expr(folder: &mut impl Folder, expr: &Expr) -> Expr {
    let rebuilt = match expr.kind() {
        ExprKind::Number(_) | ExprKind::Symbol(_) => expr.clone(),
        ExprKind::Add(terms) => Expr::sum(terms.iter().map(|term| folder.fold(term))),
        ExprKind::Mul(factors) => Expr::product(factors.iter().map(|factor| folder.fold(factor))),
        ExprKind::Pow { base, exp } => Expr::pow(folder.fold(base), folder.fold(exp)),
        ExprKind::Call { function, args } => {
            Expr::call(function.clone(), args.iter().map(|arg| folder.fold(arg)))
        }
        ExprKind::Derivative(derivative) => {
            Expr::derivative(folder.fold(&derivative.expr), derivative.variable.clone())
        }
        ExprKind::Integral(integral) => {
            Expr::integral(folder.fold(&integral.expr), integral.variable.clone())
        }
    };

    folder.rewrite(rebuilt)
}

pub fn rewrite_bottom_up(expr: &Expr, rewrite: &mut impl FnMut(Expr) -> Expr) -> Expr {
    struct ClosureFolder<'a, F> {
        rewrite: &'a mut F,
    }

    impl<F> Folder for ClosureFolder<'_, F>
    where
        F: FnMut(Expr) -> Expr,
    {
        fn rewrite(&mut self, expr: Expr) -> Expr {
            (self.rewrite)(expr)
        }
    }

    let mut folder = ClosureFolder { rewrite };
    fold_expr(&mut folder, expr)
}

pub fn replace(expr: &Expr, needle: &Expr, replacement: &Expr) -> Expr {
    rewrite_bottom_up(expr, &mut |candidate| {
        if candidate == *needle {
            replacement.clone()
        } else {
            candidate
        }
    })
}
