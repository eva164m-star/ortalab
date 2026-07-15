use ortalib::{Chips, Mult, Round};

use crate::poker;

pub fn score_round(round: &Round) -> (Chips, Mult) {
    let evaluated = poker::evaluate(&round.cards_played);
    let (mut chips, mult) = evaluated.hand.hand_value();

    chips += evaluated
        .scoring_indices
        .iter()
        .map(|&index| round.cards_played[index].rank.rank_value())
        .sum::<Chips>();

    (chips, mult)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ortalib::{Card, Rank::*, Suit::*};

    #[test]
    fn only_pair_cards_score() {
        let round = Round {
            cards_played: vec![
                Card::new(Ace, Hearts, None, None),
                Card::new(Ace, Spades, None, None),
                Card::new(King, Diamonds, None, None),
                Card::new(Nine, Clubs, None, None),
                Card::new(Five, Hearts, None, None),
            ],
            cards_held_in_hand: vec![],
            jokers: vec![],
        };

        assert_eq!(score_round(&round), (32.0, 2.0));
    }
}
