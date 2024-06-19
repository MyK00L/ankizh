use std::cmp::Ordering;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Definition {
    pub pinyin: Option<String>,
    pub english: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Priority {
    pub val: f32,
    pub max: f32,
}
impl Eq for Priority {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharWriting {
    Strokes(Vec<String>),
    Char(char),
}

#[derive(Clone, Debug, Default, Eq)]
pub struct Entry {
    pub id: String,
    pub pinyin: Vec<String>,
    pub definitions: Vec<Definition>,
    pub priority: Vec<Priority>,
    pub dependencies: Vec<String>,
    pub writing: Vec<CharWriting>,
    pub traditional: Option<String>,
}
impl Entry {
    pub fn total_priority(&self) -> f32 {
        if self.priority.is_empty() {
            0f32
        } else {
            self.priority.iter().map(|x| x.val).sum::<f32>()
                / self.priority.iter().map(|x| x.max).sum::<f32>()
        }
    }
    pub fn merge_add(&mut self, mut o: Entry) {
        if !(self.writing.is_empty() || o.writing.is_empty()) {
            eprintln!("{:?} {:?}", self, o.clone());
        }
        assert_eq!(self.id, o.id);
        for py in o.pinyin {
            if !self.pinyin.contains(&py) {
                self.pinyin.push(py);
            }
        }
        for dp in o.dependencies {
            if !self.dependencies.contains(&dp) {
                self.dependencies.push(dp);
            }
        }
        self.definitions.append(&mut o.definitions);
        self.priority.append(&mut o.priority);
        assert!(self.writing.is_empty() || o.writing.is_empty());
        self.writing.append(&mut o.writing);

        self.traditional = self.traditional.take().or(o.traditional);
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.total_priority().partial_cmp(&other.total_priority())
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.total_priority().eq(&other.total_priority())
    }
}
