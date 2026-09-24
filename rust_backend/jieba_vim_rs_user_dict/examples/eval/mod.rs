fn get_boundaries(segmented_sent: &str) -> (usize, Vec<usize>) {
    let mut tokens = segmented_sent.trim_ascii().split_ascii_whitespace();
    let mut char_count = 0;
    let mut boundaries = Vec::new();
    if let Some(token1) = tokens.next() {
        char_count += token1.chars().count();
        let mut offset = token1.len();
        for token in tokens {
            char_count += token.chars().count();
            boundaries.push(offset);
            offset += token.len();
        }
    }
    (char_count, boundaries)
}

fn evaluate_boundaries(
    prediction: &[usize],
    groundtruth: &[usize],
    tpos: &mut u64,
    gpos: &mut u64,
    ppos: &mut u64,
) {
    let mut pbounds = prediction.into_iter().peekable();
    let mut gbounds = groundtruth.into_iter().peekable();
    while pbounds.peek().is_some() && gbounds.peek().is_some() {
        let pb = pbounds.peek().unwrap();
        let gb = gbounds.peek().unwrap();
        if pb < gb {
            pbounds.next().unwrap();
            *ppos += 1;
        } else if pb > gb {
            gbounds.next().unwrap();
            *gpos += 1;
        } else {
            pbounds.next().unwrap();
            gbounds.next().unwrap();
            *ppos += 1;
            *gpos += 1;
            *tpos += 1;
        }
    }
    while pbounds.next().is_some() {
        *ppos += 1;
    }
    while gbounds.next().is_some() {
        *gpos += 1;
    }
}

/// `prediction` and `groundtruth` are space-separated list of tokens of the
/// same number of total characters. `tpos` is the true positive count. `gpos`
/// is the groundtruth positive count. `ppos` is the predicted positive count.
pub fn evaluate_sentence(
    prediction: &str,
    groundtruth: &str,
    tpos: &mut u64,
    gpos: &mut u64,
    ppos: &mut u64,
) {
    let (pc, pbounds) = get_boundaries(prediction);
    let (gc, gbounds) = get_boundaries(groundtruth);
    if pc != gc {
        panic!(
            "prediction sent chars ({}) != groundtruth sent chars ({})",
            pc, gc
        );
    }
    evaluate_boundaries(&pbounds, &gbounds, tpos, gpos, ppos);
}

pub fn f_score(precision: f64, recall: f64, beta: f64) -> f64 {
    let beta2 = beta * beta;
    (1.0 + beta2) * precision * recall / (beta2 * precision + recall)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_boundaries() {
        assert_eq!(get_boundaries("foo"), (3, vec![]));
        assert_eq!(get_boundaries("foo bar"), (6, vec![3]));
        assert_eq!(get_boundaries("foo baar baz"), (10, vec![3, 7]));
    }

    #[test]
    fn test_evaluate_boundaries() {
        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(&[], &[], &mut tpos, &mut gpos, &mut ppos);
        assert_eq!((tpos, gpos, ppos), (0, 0, 0));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(
            &[],
            &[1, 2, 4, 5],
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
        assert_eq!((tpos, gpos, ppos), (0, 4, 0));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(&[1, 3, 4], &[], &mut tpos, &mut gpos, &mut ppos);
        assert_eq!((tpos, gpos, ppos), (0, 0, 3));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(
            &[1, 3, 4],
            &[1, 2, 4, 5],
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
        assert_eq!((tpos, gpos, ppos), (2, 4, 3));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(
            &[1, 3, 4, 7],
            &[1, 2, 4, 5, 7],
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
        assert_eq!((tpos, gpos, ppos), (3, 5, 4));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(
            &[1, 3, 4, 7],
            &[1, 2, 4, 5],
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
        assert_eq!((tpos, gpos, ppos), (2, 4, 4));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(
            &[1, 3, 4, 7, 8],
            &[1, 2, 4, 5],
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
        assert_eq!((tpos, gpos, ppos), (2, 4, 5));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(
            &[1, 3, 4, 7],
            &[2, 5],
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
        assert_eq!((tpos, gpos, ppos), (0, 2, 4));

        let (mut tpos, mut gpos, mut ppos) = (0, 0, 0);
        evaluate_boundaries(
            &[1, 2, 3, 4, 5, 6, 7],
            &[2, 5],
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
        assert_eq!((tpos, gpos, ppos), (2, 2, 7));
    }
}
