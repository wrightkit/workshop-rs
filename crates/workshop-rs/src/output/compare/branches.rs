use super::*;

pub(super) fn branch_difference(
    a: &wir::Program,
    b: &wir::Program,
    first_branches: &[wir::IfBranch],
    second_branches: &[wir::IfBranch],
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let first_conditional = first_branches
        .iter()
        .filter(|branch| branch.condition.is_some())
        .count();
    let second_conditional = second_branches
        .iter()
        .filter(|branch| branch.condition.is_some())
        .count();
    if first_conditional != second_conditional {
        return Some(DifferenceKind::Count {
            element: "branches",
            first: first_conditional,
            second: second_conditional,
        });
    }
    let first_elses = first_branches.len() - first_conditional;
    let second_elses = second_branches.len() - second_conditional;
    let multiple = first_elses > 1 || second_elses > 1;
    let ordinary = |branches: &[wir::IfBranch]| {
        branches
            .iter()
            .take(branches.len().saturating_sub(1))
            .all(|branch| branch.condition.is_some())
    };
    if multiple || !ordinary(first_branches) || !ordinary(second_branches) {
        if first_branches.len() != second_branches.len() {
            return Some(DifferenceKind::Count {
                element: "branches",
                first: first_branches.len(),
                second: second_branches.len(),
            });
        }
        if let Some((first, second)) = first_branches
            .iter()
            .zip(second_branches)
            .find(|(a, b)| a.condition.is_some() != b.condition.is_some())
        {
            return Some(DifferenceKind::Shape {
                first: if first.condition.is_some() {
                    "conditional branch"
                } else {
                    "else branch"
                },
                second: if second.condition.is_some() {
                    "conditional branch"
                } else {
                    "else branch"
                },
            });
        }
    }
    let mut conditional_index = 0;
    let mut else_index = 0;
    for (first, second) in first_branches.iter().zip(second_branches) {
        match (first.condition, second.condition) {
            (Some(first_condition), Some(second_condition)) => {
                path.push(PathStep::Value(conditional_index));
                if let Some(kind) = value_difference(a, b, first_condition, second_condition, path)
                {
                    return Some(kind);
                }
                path.pop();
                path.push(PathStep::Branch(conditional_index));
                conditional_index += 1;
            }
            (None, None) => {
                path.push(if multiple {
                    PathStep::ElseBranch(else_index)
                } else {
                    PathStep::ElseBody
                });
                else_index += 1;
            }
            _ => unreachable!("branch shape checked above"),
        }
        if let Some(kind) = action_list_difference(a, b, &first.body, &second.body, path) {
            return Some(kind);
        }
        path.pop();
    }
    if first_elses != second_elses {
        return Some(DifferenceKind::Shape {
            first: if first_elses > 0 { "if/else" } else { "if" },
            second: if second_elses > 0 { "if/else" } else { "if" },
        });
    }
    None
}
