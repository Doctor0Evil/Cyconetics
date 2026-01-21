pub fn bayes_posterior(
    prior: f32,          // P(H)
    likelihood: f32,     // P(D|H)
    alt_prior: f32,      // P(~H)
    alt_likelihood: f32, // P(D|~H)
) -> f32 {
    // returns P(H|D)
    // P(H|D) = P(D|H) P(H) / [P(D|H)P(H) + P(D|~H)P(~H)]
    unimplemented!()
}
