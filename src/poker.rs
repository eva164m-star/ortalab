use ortalib::{Card, Enhancement, PokerHand, Rank, Suit};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct EvaluatedHand {
    pub hand: PokerHand,
    pub scoring_indices: Vec<usize>,
}

#[derive(Clone, Copy, Default)]
pub struct Rules {
    pub four_fingers: bool,
    pub shortcut: bool,
    pub smeared: bool,
    pub splash: bool,
}

pub fn evaluate(cards: &[Card], rules: Rules) -> EvaluatedHand {
    assert!(!cards.is_empty());
    let regular: Vec<usize> = cards
        .iter()
        .enumerate()
        .filter_map(|(i, c)| (!is_stone(c)).then_some(i))
        .collect();
    let rcards: Vec<Card> = regular.iter().map(|&i| cards[i]).collect();
    let mut e = evaluate_regular(&rcards, rules);
    e.scoring_indices = e.scoring_indices.into_iter().map(|i| regular[i]).collect();
    e.scoring_indices.extend(
        cards
            .iter()
            .enumerate()
            .filter_map(|(i, c)| is_stone(c).then_some(i)),
    );
    if rules.splash {
        e.scoring_indices = (0..cards.len()).collect();
    }
    e.scoring_indices.sort_unstable();
    e.scoring_indices.dedup();
    e
}

fn evaluate_regular(cards: &[Card], rules: Rules) -> EvaluatedHand {
    if cards.is_empty() {
        return EvaluatedHand {
            hand: PokerHand::HighCard,
            scoring_indices: vec![],
        };
    }
    let groups = group_by_rank(cards);
    let mut gs: Vec<_> = groups.into_iter().collect();
    gs.sort_by(|(ra, ia), (rb, ib)| ib.len().cmp(&ia.len()).then_with(|| rb.cmp(ra)));
    let sizes: Vec<_> = gs.iter().map(|x| x.1.len()).collect();
    if cards.len() == 5
        && sizes.first() == Some(&5)
        && flush_subset(cards, 5, rules.smeared).is_some()
    {
        return all(PokerHand::FlushFive, cards.len());
    }
    if cards.len() == 5
        && sizes.as_slice() == [3, 2]
        && flush_subset(cards, 5, rules.smeared).is_some()
    {
        return all(PokerHand::FlushHouse, cards.len());
    }
    if cards.len() == 5 && sizes.first() == Some(&5) {
        return all(PokerHand::FiveOfAKind, cards.len());
    }
    let min = if rules.four_fingers { 4 } else { 5 };
    let straight = largest_straight_subset(cards, min, rules.shortcut);
    let flush = largest_flush_subset(cards, min, rules.smeared);
    if let (Some(s), Some(f)) = (straight.as_ref(), flush.as_ref())
        && (rules.four_fingers || s == f)
    {
        let mut u = s.clone();
        for &i in f {
            if !u.contains(&i) {
                u.push(i)
            }
        }
        u.sort();
        return EvaluatedHand {
            hand: PokerHand::StraightFlush,
            scoring_indices: u,
        };
    }
    if let Some((_, v)) = gs.iter().find(|x| x.1.len() == 4) {
        return EvaluatedHand {
            hand: PokerHand::FourOfAKind,
            scoring_indices: v.clone(),
        };
    }
    if cards.len() == 5 && sizes.as_slice() == [3, 2] {
        return all(PokerHand::FullHouse, 5);
    }
    if let Some(v) = flush {
        return EvaluatedHand {
            hand: PokerHand::Flush,
            scoring_indices: v,
        };
    }
    if let Some(v) = straight {
        return EvaluatedHand {
            hand: PokerHand::Straight,
            scoring_indices: v,
        };
    }
    if let Some((_, v)) = gs.iter().find(|x| x.1.len() == 3) {
        return EvaluatedHand {
            hand: PokerHand::ThreeOfAKind,
            scoring_indices: v.clone(),
        };
    }
    let pairs: Vec<_> = gs.iter().filter(|x| x.1.len() == 2).collect();
    if pairs.len() >= 2 {
        let mut v = vec![];
        v.extend(&pairs[0].1);
        v.extend(&pairs[1].1);
        v.sort();
        return EvaluatedHand {
            hand: PokerHand::TwoPair,
            scoring_indices: v,
        };
    }
    if let Some(x) = pairs.first() {
        return EvaluatedHand {
            hand: PokerHand::Pair,
            scoring_indices: x.1.clone(),
        };
    }
    let i = cards.iter().enumerate().max_by_key(|x| x.1.rank).unwrap().0;
    EvaluatedHand {
        hand: PokerHand::HighCard,
        scoring_indices: vec![i],
    }
}
fn all(h: PokerHand, n: usize) -> EvaluatedHand {
    EvaluatedHand {
        hand: h,
        scoring_indices: (0..n).collect(),
    }
}
fn group_by_rank(cards: &[Card]) -> BTreeMap<Rank, Vec<usize>> {
    let mut m: BTreeMap<Rank, Vec<usize>> = BTreeMap::new();
    for (i, c) in cards.iter().enumerate() {
        m.entry(c.rank).or_default().push(i)
    }
    m
}
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    fn rec(n: usize, k: usize, s: usize, v: &mut Vec<usize>, o: &mut Vec<Vec<usize>>) {
        if v.len() == k {
            o.push(v.clone());
            return;
        }
        for i in s..n {
            v.push(i);
            rec(n, k, i + 1, v, o);
            v.pop();
        }
    }
    let mut o = vec![];
    rec(n, k, 0, &mut vec![], &mut o);
    o
}
fn flush_subset(c: &[Card], k: usize, smeared: bool) -> Option<Vec<usize>> {
    if c.len() < k {
        return None;
    }
    for ids in combinations(c.len(), k) {
        for suit in [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds] {
            if ids.iter().all(|&i| suit_match(&c[i], suit, smeared)) {
                return Some(ids);
            }
        }
    }
    None
}
fn largest_flush_subset(c: &[Card], min: usize, smeared: bool) -> Option<Vec<usize>> {
    for k in (min..=c.len()).rev() {
        if let Some(ids) = flush_subset(c, k, smeared) {
            return Some(ids);
        }
    }
    None
}
fn straight_subset(c: &[Card], k: usize, shortcut: bool) -> Option<Vec<usize>> {
    if c.len() < k {
        return None;
    }
    for ids in combinations(c.len(), k) {
        let mut r: Vec<u8> = ids.iter().map(|&i| rank_index(c[i].rank)).collect();
        r.sort();
        r.dedup();
        if r.len() != k {
            continue;
        }
        let ok = if shortcut {
            r.windows(2).all(|w| w[1] - w[0] <= 2)
        } else {
            r.windows(2).all(|w| w[1] == w[0] + 1)
        };
        let ace_low = if k == 5 {
            r == [0, 1, 2, 3, 12]
        } else {
            r == [0, 1, 2, 12]
        };
        if ok || ace_low {
            return Some(ids);
        }
    }
    None
}
fn largest_straight_subset(c: &[Card], min: usize, shortcut: bool) -> Option<Vec<usize>> {
    for k in (min..=c.len()).rev() {
        if let Some(ids) = straight_subset(c, k, shortcut) {
            return Some(ids);
        }
    }
    None
}
pub fn contains_pair(c: &[Card]) -> bool {
    counts(c).values().any(|&n| n >= 2)
}
pub fn contains_three(c: &[Card]) -> bool {
    counts(c).values().any(|&n| n >= 3)
}
pub fn contains_two_pair(c: &[Card]) -> bool {
    counts(c).values().filter(|&&n| n >= 2).count() >= 2
}
pub fn contains_straight(c: &[Card], r: Rules) -> bool {
    largest_straight_subset(
        &c.iter()
            .copied()
            .filter(|x| !is_stone(x))
            .collect::<Vec<_>>(),
        if r.four_fingers { 4 } else { 5 },
        r.shortcut,
    )
    .is_some()
}
pub fn contains_flush(c: &[Card], r: Rules) -> bool {
    largest_flush_subset(
        &c.iter()
            .copied()
            .filter(|x| !is_stone(x))
            .collect::<Vec<_>>(),
        if r.four_fingers { 4 } else { 5 },
        r.smeared,
    )
    .is_some()
}
fn counts(c: &[Card]) -> BTreeMap<Rank, usize> {
    let mut m = BTreeMap::new();
    for x in c.iter().filter(|x| !is_stone(x)) {
        *m.entry(x.rank).or_default() += 1
    }
    m
}
pub fn is_face(c: &Card, pareidolia: bool) -> bool {
    !is_stone(c) && (pareidolia || matches!(c.rank, Rank::Jack | Rank::Queen | Rank::King))
}
pub fn suit_match(c: &Card, s: Suit, smeared: bool) -> bool {
    if is_stone(c) {
        return false;
    }
    if c.enhancement == Some(Enhancement::Wild) {
        return true;
    }
    if smeared {
        matches!(
            (c.suit, s),
            (Suit::Hearts | Suit::Diamonds, Suit::Hearts | Suit::Diamonds)
                | (Suit::Spades | Suit::Clubs, Suit::Spades | Suit::Clubs)
        )
    } else {
        c.suit == s
    }
}
fn is_stone(c: &Card) -> bool {
    c.enhancement == Some(Enhancement::Stone)
}
fn rank_index(r: Rank) -> u8 {
    use Rank::*;
    match r {
        Two => 0,
        Three => 1,
        Four => 2,
        Five => 3,
        Six => 4,
        Seven => 5,
        Eight => 6,
        Nine => 7,
        Ten => 8,
        Jack => 9,
        Queen => 10,
        King => 11,
        Ace => 12,
    }
}
