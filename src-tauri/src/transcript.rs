use serde::Serialize;

pub const CAPACITY: usize = 30;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    pub speaker: String,
    pub text: String,
    pub is_user_turn: bool,
    pub ts: i64,
}

#[derive(Debug, Default)]
pub struct TranscriptBuffer {
    turns: std::collections::VecDeque<Turn>,
}

impl TranscriptBuffer {
    pub fn push(&mut self, t: Turn) {
        if self.turns.len() >= CAPACITY {
            self.turns.pop_front();
        }
        self.turns.push_back(t);
    }
    pub fn last(&self, n: usize) -> Vec<&Turn> {
        self.turns.iter().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect()
    }
    pub fn len(&self) -> usize { self.turns.len() }
    pub fn is_empty(&self) -> bool { self.turns.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(i: usize) -> Turn {
        Turn { speaker: "s".into(), text: format!("t{i}"), is_user_turn: false, ts: i as i64 }
    }

    #[test]
    fn buffer_bounded_to_capacity() {
        let mut b = TranscriptBuffer::default();
        for i in 0..35 { b.push(turn(i)); }
        assert_eq!(b.len(), CAPACITY);
        assert_eq!(b.last(1)[0].text, "t34");
        assert_eq!(b.last(30)[0].text, "t5"); // oldest retained
    }
}
