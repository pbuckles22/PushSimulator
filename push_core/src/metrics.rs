//! Win rates, average turns, and score variance for a batch of finished games.
//!
//! The lowest total score wins a game. A shared lowest score is a tie, and a
//! tie is not a win for any seat. Win rate is that seat's wins divided by the
//! number of games. Score variance is the population variance of that seat's
//! totals (the sum of squared differences from the mean, divided by the number
//! of games).

/// One finished match: how many turns it took, and each seat's total score.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameSample {
    pub turns: u32,
    pub scores: Vec<u32>,
}

/// One seat's wins and score spread across a batch.
#[derive(Clone, Debug, PartialEq)]
pub struct SeatMetrics {
    pub seat: u32,
    pub wins: u64,
    pub win_rate: f64,
    pub mean_score: f64,
    pub score_variance: f64,
}

/// Batch totals. `average_turns` is the mean turn count. `ties` counts games
/// whose lowest score was shared.
#[derive(Clone, Debug, PartialEq)]
pub struct BatchMetrics {
    pub games: u64,
    pub ties: u64,
    pub average_turns: f64,
    pub seats: Vec<SeatMetrics>,
}

/// Summarize `samples`.
///
/// An empty slice has zero games, zero ties, an average of 0, and no seats.
/// Every sample must list at least one score, and every sample must list the
/// same number of seats.
pub fn batch_metrics(samples: &[GameSample]) -> BatchMetrics {
    if samples.is_empty() {
        return BatchMetrics {
            games: 0,
            ties: 0,
            average_turns: 0.0,
            seats: Vec::new(),
        };
    }

    let seats = samples[0].scores.len();
    assert!(seats > 0, "a game records at least one seat");
    for sample in samples {
        assert_eq!(
            sample.scores.len(),
            seats,
            "every game records the same seats"
        );
    }

    let games = samples.len() as u64;
    let mut wins = vec![0u64; seats];
    let mut ties = 0u64;
    let mut turn_sum = 0u64;
    let mut score_sum = vec![0u64; seats];
    for sample in samples {
        turn_sum += u64::from(sample.turns);
        let best = sample
            .scores
            .iter()
            .copied()
            .min()
            .expect("a game records at least one seat");
        let mut leaders = 0usize;
        let mut leader = 0usize;
        for (seat, score) in sample.scores.iter().copied().enumerate() {
            score_sum[seat] += u64::from(score);
            if score == best {
                leaders += 1;
                leader = seat;
            }
        }
        if leaders == 1 {
            wins[leader] += 1;
        } else {
            ties += 1;
        }
    }

    let games_f = games as f64;
    let means: Vec<f64> = score_sum.iter().map(|sum| *sum as f64 / games_f).collect();
    let mut square_sum = vec![0.0; seats];
    for sample in samples {
        for seat in 0..seats {
            let delta = sample.scores[seat] as f64 - means[seat];
            square_sum[seat] += delta * delta;
        }
    }

    let seat_metrics = (0..seats)
        .map(|seat| SeatMetrics {
            seat: seat as u32,
            wins: wins[seat],
            win_rate: wins[seat] as f64 / games_f,
            mean_score: means[seat],
            score_variance: square_sum[seat] / games_f,
        })
        .collect();

    BatchMetrics {
        games,
        ties,
        average_turns: turn_sum as f64 / games_f,
        seats: seat_metrics,
    }
}
